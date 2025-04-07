use super::address::VirtAddr;

#[allow(unused)]

/// 内存操作错误类型（跨架构通用）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryError {
    InvalidEntry,   // PTE存在但Valid=0
    OutOfMemory,

    PermissionDenied, // 权限检查失败
    /// 地址越界（用户/内核空间）
    /// - `address`: 违规地址
    /// - `max_valid`: 该空间的最大合法地址
    AddressOutOfRange {
        address: VirtAddr,
        max_valid: VirtAddr,
    },
    
    // /// 内存访问权限不足
    // /// - `va`: 目标虚拟地址
    // /// - `required`: 需要的权限（R/W/X）
    // /// - `actual`: 实际页表项权限
    // PermissionDenied {
    //     va: usize,
    //     required: PagePermission,
    //     actual: PagePermission,
    // },
    
    /// 页表项未映射或无效
    /// - `pte_addr`: 页表项物理地址
    PageNotMapped,
    
    /// 地址对齐错误（如非页对齐的 DMA 操作）
    /// - `address`: 未对齐地址
    /// - `alignment`: 要求对齐粒度（如 4096）
    Misaligned {
        address: usize,
        alignment: usize,
    },
    
    /// 物理页不连续（需要连续物理内存的操作）
    /// - `first_bad`: 第一个不连续页的索引
    NonContinuous(usize),
    
    /// 空缓冲区操作（零长度）
    EmptyBuffer,
}