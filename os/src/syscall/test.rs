use alloc::string::String;

use crate::{mm::user_ptr::UserBuffer, println, task::current_user_token};

pub fn sys_test(
    great_cross_page_ptr: usize,
    great_len: usize, 
    arg2: usize, 
    arg3: usize, 
    arg4: usize, 
    arg5: usize
) {
    let great_cross_page_ptr = great_cross_page_ptr as *const u8;
    let string_buffer = UserBuffer::new(current_user_token(), great_cross_page_ptr, great_len);
    let great_str:String = string_buffer.into();

    println!("{}", great_str);

    println!("arg2: {}, arg3: {}, arg4: {}, arg5: {}",
        arg2, arg3, arg4, arg5
    );
}