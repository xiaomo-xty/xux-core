// #![no_std]
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Expr, ItemFn, FnArg, Ident, ReturnType};
use syn::spanned::Spanned;



/// A procedural macro attribute for registering system call handlers.
///
/// This attribute transforms a function into a system call handler by:
/// 1. Preserving the original function
/// 2. Generating a wrapper function that handles argument conversion
/// 3. Automatically registering the handler in the system call table
///
/// # Usage
/// ```rust,ignore
/// #[syscall_register(42)] // 42 is the system call number
/// fn my_syscall(arg1: usize, arg2: usize) -> isize {
///     // Implementation
/// }
/// ```
///
/// # Safety
/// - The macro generates unsafe code for system call argument conversion
/// - The target function must only take types that implement `From<usize>`
/// - The system call table (`SYSCALL_TABLE`) must be declared elsewhere
///
/// # Generated Code
/// For each annotated function, the macro generates:
/// 1. The original function
/// 2. A wrapper function with ABI-compatible signature
/// 3. Automatic registration in the system call table


#[proc_macro_attribute]
pub fn syscall_register(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析属性参数和函数定义
    let expr = parse_macro_input!(attr as Expr);
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    // 处理系统调用号表达式
    let syscall_num = match parse_syscall_num(&expr) {
        Ok(num) => num,
        Err(err) => return err.to_compile_error().into(),
    };

    // 提取参数信息
    let params = input_fn.sig.inputs.iter().enumerate().map(|(i, arg)| {
        match arg {
            FnArg::Typed(pat) => (i, &pat.pat, &pat.ty),
            _ => panic!("Receiver arguments not supported in syscall handlers"),
        }
    });

    // 生成参数转换代码
    let arg_conversions = params.clone().map(|(i, arg_name, arg_type)| {
        quote! {
            let #arg_name = unsafe { 
                 args[#i] as #arg_type
            };
        }
    });

    // 生成参数名列表用于调用原函数
    let arg_names = params.map(|(_, arg_name, _)| arg_name);

    let wrapper_name = format_ident!("{}_wrapper", fn_name);
    let register_name = format_ident!("REGISTER_{}", fn_name.to_string().to_uppercase());


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

    // 生成最终代码
    let expanded = quote! {
        // reserve original function
        #input_fn

        #[allow(unreachable_code)]
        #[doc(hidden)]
        #[inline(never)]

        // pub fn #wrapper_name (args: [usize; 6]) -> isize {
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
    // eprintln!("{}\n\n\n", token_stream);
    token_stream
}

/// 解析系统调用号表达式，支持常量和字面量
fn parse_syscall_num(expr: &Expr) -> Result<proc_macro2::TokenStream, syn::Error> {
    match expr {
        // 处理字面量情况 (如 #[syscall_register(1)])
        Expr::Lit(lit) => Ok(quote! { #lit }),
        
        // 处理路径表达式 (如 #[syscall_register(SYSCALL_EXIT)])
        Expr::Path(path) => {
            let ident = path.path.get_ident().ok_or_else(|| {
                syn::Error::new(path.span(), "Expected identifier for syscall number")
            })?;
            
            // 这里我们假设调用者已经正确引入了常量
            // 实际使用时可能需要更复杂的解析
            Ok(quote! { crate::syscall::syscall_num:: #ident })
        },
        
        // 处理其他表达式 (如 #[syscall_register(SYSCALL_BASE + 1)])
        _ => Ok(quote! { #expr }),
    }
}

/// 检查是否为发散函数 (! 类型)
fn is_never_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        type_path.path.segments.last().map_or(false, |seg| seg.ident == "!")
    } else {
        false
    }
}