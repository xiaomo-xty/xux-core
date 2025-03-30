use super::{address::VirtAddr, error::MemoryError};

/// 统一内存缓冲区抽象
pub trait Buffer {
    /// 获取缓冲区起始虚拟地址
    fn start_va(&self) -> VirtAddr;
    
    /// 获取缓冲区长度（字节数）
    fn len(&self) -> usize;
    
    /// 检查缓冲区是否为空
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// 转换为只读字节切片（需权限验证）
    /// 注意：必须确保映射在生命周期内有效
    unsafe fn as_bytes(&self) -> Result<&[u8], MemoryError>;
    
    /// 转换为可写字节切片（需额外权限检查）
    unsafe fn as_bytes_mut(&mut self) -> Result<&mut [u8], MemoryError>;
    
    // /// 获取物理页列表（用于 DMA 等操作）
    // fn phys_pages(&self) -> Result<Vec<PhysAddr>, MemoryError>;
    
    // /// 检查是否连续物理内存（优化批处理操作）
    // fn is_phys_contiguous(&self) -> bool;
}


pub struct UserBuffer {
    start_va: usize,
    length: usize,
}

impl UserBuffer {
    fn new(start_va: usize, length: usize) -> Result<Self, MemoryError> {
        let start_va: VirtAddr = start_va.into();
        if length == 0 {
            return Err(MemoryError::EmptyBuffer)
        }

        if !start_va.is_user() {
            return Err(MemoryError::AddressOutOfRange {
                address: start_va.into(),
                max_valid: VirtAddr::USER_MAX,
            });
        }


        let end_va = VirtAddr(start_va.0 + length);

        if !end_va.is_user() {
            log::error!("end_va is not a user address");
            return Err(MemoryError::AddressOutOfRange {
                address: start_va.into(),
                max_valid: VirtAddr::USER_MAX,
            });
        }

        



        UserBuffer { start_va, length}
    }
}