// #![no_std]
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, FnArg, ItemFn, ReturnType};

/// System call registration procedural macro
///
/// Transforms a function into a system call handler with:
/// 1. Original function preservation
/// 2. Wrapper generation for ABI compatibility
/// 3. Automatic registration in system call table
///
/// Usage: #[syscall_register(N)] where N is the syscall number
///
/// Safety requirements:
/// - Only accepts types implementing From<usize>
/// - Requires SYSCALL_TABLE to be defined externally
/// - Generates unsafe argument conversion code
#[proc_macro_attribute]
pub fn syscall_register(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute as expression and input function
    let expr = parse_macro_input!(attr as Expr);
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    // Handle syscall number parsing (supports literals, paths, and expressions)
    let syscall_num = match parse_syscall_num(&expr) {
        Ok(num) => num,
        Err(err) => return err.to_compile_error().into(),
    };

    // Extract parameter information (index, name, type)
    let params = input_fn
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(|(i, arg)| match arg {
            FnArg::Typed(pat) => (i, &pat.pat, &pat.ty),
            _ => panic!("Receiver arguments not supported in syscall handlers"),
        });

    // Generate argument conversion code for wrapper
    let arg_conversions = params.clone().map(|(i, arg_name, arg_type)| {
        quote! {
            let #arg_name = unsafe {
                 args[#i] as #arg_type
            };
        }
    });

    // Collect argument names for function call
    let arg_names = params.map(|(_, arg_name, _)| arg_name);

    let wrapper_name = format_ident!("{}_wrapper", fn_name);
    let register_name = format_ident!("REGISTER_{}", fn_name.to_string().to_uppercase());

    // Handle different return type cases:
    // - Default (no return) -> returns 0
    // - Never type (!) -> unreachable
    // - Normal return -> converted to isize
    let wrapper_return = match &input_fn.sig.output {
        ReturnType::Default => quote! {#fn_name(#(#arg_names),*); 0 },
        ReturnType::Type(_, ty) => {
            if is_never_type(&ty) {
                quote! {
                    #fn_name(#(#arg_names),*);
                    unsafe { core::hint::unreachable_unchecked() }
                }
            } else {
                quote! {
                    #fn_name(#(#arg_names),*) as isize
                }
            }
        }
    };

    // Generate final output containing:
    // 1. Original function
    // 2. Wrapper function
    // 3. Registration static
    let expanded = quote! {
        // reserve original function
        #input_fn

        #[allow(unreachable_code)]
        #[doc(hidden)]
        #[inline(never)]
        pub unsafe extern "C" fn #wrapper_name (args: [usize; 6]) -> isize {
            #(#arg_conversions)*
            #wrapper_return
        }

        #[used]
        #[link_section = ".syscall_registry"]
        static #register_name: crate::syscall::SyscallRegistry = crate::syscall::SyscallRegistry {
            num: #syscall_num,
            handler: #wrapper_name,
        };
    };

    let token_stream = expanded.into();
    token_stream
}

/// Parses syscall number from attribute expression
///
/// Supports:
/// - Literals (e.g., 42)
/// - Paths (e.g., SYSCALL_EXIT)
/// - Complex expressions (e.g., BASE + 1)
fn parse_syscall_num(expr: &Expr) -> Result<proc_macro2::TokenStream, syn::Error> {
    match expr {
        Expr::Lit(lit) => Ok(quote! { #lit }),
        Expr::Path(path) => {
            let ident = path.path.get_ident().ok_or_else(|| {
                syn::Error::new(path.span(), "Expected identifier for syscall number")
            })?;
            Ok(quote! { crate::syscall::syscall_num:: #ident })
        }
        _ => Ok(quote! { #expr }),
    }
}

/// Checks if type is the never type (!)
fn is_never_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .map_or(false, |seg| seg.ident == "!")
    } else {
        false
    }
}

/// Kernel test case procedural macro
///
/// Enhances test cases with:
/// - Automatic test identification
/// - Colored output formatting
/// - Source location reporting
///
/// Generates both original function and test wrapper
#[proc_macro_attribute]
pub fn kernel_test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let wrapper_name = format_ident!("__{}_test_wrapper", fn_name);

    // Generate test wrapper with:
    // 1. Test identification header
    // 2. Original function execution
    // 3. Success/failure reporting
    let output = quote! {
        // Original function (unchanged)
        #[allow(unused)]
        #input_fn

        // Generated test wrapper
        #[doc(hidden)]
        #[test_case]
        fn #wrapper_name () {
            crate::color_println!(crate::io::console::Color::Blue,
                "\nTesting > {} ({}::{}) ...",
                stringify!(#fn_name),
                file!(),
                stringify!(#fn_name)
            );

            #fn_name ();
            crate::color_println!(crate::io::console::Color::Green, 
                "========[Test passed!]========"
            );
        }
    };

    output.into()
}
