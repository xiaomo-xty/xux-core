use core::{marker::PhantomData, mem::{self, MaybeUninit}};
use alloc::{boxed::Box, string::String, vec::Vec};
use super::{error::MemoryError, page_table::{copy_from_user, translated_str}};

/// A zero-cost safe wrapper around user-space memory pointers.
///
/// This provides safe access to memory in user-space from kernel-space,
/// handling potential page faults and invalid addresses.
pub struct UserPtr<T> {
    token: usize,
    addr: *const T,
    _phantom: PhantomData<*mut [T]>,
}

impl<T> UserPtr<T> 
where
    T: Sized 
{
    /// Creates a new UserPtr from a raw pointer and a token.
    ///
    /// # Arguments
    /// * `token` - An identifier for the address space
    /// * `addr` - The raw pointer in user-space
    pub fn new(token: usize, addr: *const T) -> Self {
        Self {
            token,
            addr,
            _phantom: PhantomData,
        }
    }

    /// Reads a single value of type T from user-space.
    ///
    /// # Returns
    /// The value read from user-space or a MemoryError if the operation fails.
    
    #[allow(unused)]
    pub fn read(&self) -> Result<T, MemoryError> 
    where
        T: Default + Copy,
    {
        let mut buffer = MaybeUninit::<T>::uninit();
        let buffer_ptr = buffer.as_mut_ptr();
        let elem_size = mem::size_of::<T>();
        copy_from_user(
            self.token, 
            buffer_ptr as *mut u8, 
            self.addr as *const u8, 
            elem_size
        )?;

        unsafe {
            Ok(buffer.assume_init())
        }
    }

    /// Reads a slice of values from user-space, handling cross-page access automatically.
    ///
    /// # Arguments
    /// * `len` - Number of elements to read
    ///
    /// # Returns
    /// A boxed slice containing the values or a MemoryError if the operation fails.
    pub fn read_slice(&self, len: usize) -> Result<Box<[T]>, MemoryError>
    where
        T: Default + Copy,
    {
        if len == 0 {
            return Ok(Box::new([]));
        }
    
        let elem_size = mem::size_of::<T>();
        let total_bytes = elem_size.checked_mul(len).ok_or(MemoryError::OutOfMemory)?;

        let mut buffer: Box<[MaybeUninit<T>]> = Box::new_uninit_slice(len);
        let buffer_ptr = buffer.as_mut_ptr();

        copy_from_user(
            self.token, 
            buffer_ptr as *mut u8, 
            self.addr as *const u8, 
            total_bytes
        )?;

        let init_buffer = unsafe {
            Box::from_raw(Box::into_raw(buffer) as *mut [T])
        };
        
        Ok(init_buffer) 
    }

}

impl UserPtr<u8> {
    pub fn read_to_string(&self) -> String{
        translated_str(self.token, self.addr)
    }
}



pub struct UserBuffer {
    pub buffers: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }
    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }
}



// /// A contiguous sequence of `T` in user-space memory.
// ///
// /// This wrapper guarantees:
// /// 1. All elements are of type `T`
// /// 2. Memory is contiguous (no gaps between elements)
// /// 3. May cross page boundaries but remains logically contiguous
// pub struct UserBuffer<T> {
//     user_ptr: UserPtr<T>,
//     len: usize,
//     _phantom: PhantomData<[T]>,
// }



// impl<T> UserBuffer<T> 
// where
//     T: Copy + Default,
// {
//     /// Creates a new buffer from user-space memory.
//     ///
//     /// # Safety
//     /// Caller must ensure:
//     /// - `ptr` is a valid user-space address
//     /// - Memory range `[ptr, ptr + len*size_of::<T>()]` is accessible
//     pub fn new(token: usize, ptr: *const T, len: usize) -> Self {
//         Self {
//             user_ptr: UserPtr::new(token, ptr),
//             len,
//             _phantom: PhantomData,
//         }
//     }

//     /// Reads the entire buffer into kernel space.
//     pub fn read_all(&self) -> Result<Box<[T]>, MemoryError> {
//         self.user_ptr.read_slice(self.len)
//     }
// }

// impl From<UserBuffer<u8>> for String {
//     fn from(value: UserBuffer<u8>) -> Self {
//         let bytes = value.read_all()
//         .unwrap_or_else(|_| panic!("Failed to read user buffer"));
    
//         // 2. UTF-8 验证（零拷贝转换）
//         match core::str::from_utf8(&bytes) {
//             Ok(s) => s.into(),
//             Err(e) => {
//                 panic!("Invalid UTF-8 sequence at offset {}: {:?}", e.valid_up_to(), e.error_len());
//             }
//         }
//     }
// }