use core::panic;

use os_macros::syscall_register;

use crate::{mm::user_ptr::UserPtr, print, task::current_user_token};

const FD_STDOUT: usize = 1;

// pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
//     match fd {
//         FD_STDOUT => {
//             let slice = unsafe {core::slice::from_raw_parts(buf, len)};
//             let str = core::str::from_utf8(slice).unwrap();
//             print!("{}", str);
//             len as isize
//         },
//         _ => {
//             panic!("Unsupported fd in sys_write!");
//         }
//     }
// }

/// write buf of length `len`  to a file with `fd`
#[syscall_register(SYSCALL_WRITE)]
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let user_ptr = UserPtr::new(current_user_token(), buf);
            let buffer = user_ptr.read_slice(len);

            match buffer {
                Ok(buf) => {
                    print!("{}", core::str::from_utf8(&buf).unwrap());
                },
                Err(_) => {
                    panic!("memory error")
                },
            }


            // let buffers = translated_byte_buffer(current_user_token(), buf, len);
            // for buffer in buffers {
            //     print!("{}", core::str::from_utf8(buffer).unwrap());
            // }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}