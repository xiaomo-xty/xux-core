//! 在Unix-like系统中，系统调用通常返回-1，并将错误码放在errno中

#![allow(missing_docs)] 

use strum_macros::{Display, EnumString, FromRepr};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, FromRepr)]
#[repr(i32)]
#[strum(serialize_all = "snake_case")]
pub enum Errno {
    #[strum(serialize = "Operation not permitted")]
    EPERM = 1,
    #[strum(serialize = "No such file or directory")]
    ENOENT = 2,
    #[strum(serialize = "Function not implemented")]
    ENOSYS = 38,
    // ...
}

// 自动实现 i32 -> Errno
impl TryFrom<i32> for Errno {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Errno::from_repr(value).ok_or(())
    }
}