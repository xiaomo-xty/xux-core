use crate::{exit, println, syscall::sys_exit};

#[panic_handler]
fn panic_handler(panic_info : &core::panic::PanicInfo) -> ! {
    let err = panic_info.message();
    if let Some(location) = panic_info.location() {
        println!(
            "Panic at {}:{}, {}",
            location.file(),
            location.line(),
            err
        );
    }
    else {
        println!("Paniced {}", err);
    }
    exit(-1)
}
