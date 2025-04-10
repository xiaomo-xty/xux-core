use core::{fmt::{self, Debug}, ops::Add};

use os_macros::kernel_test;

use crate::config::*;

use super::page_table::{PageTableEntry, PageTableLevel, PageTableLevelIterator};

#[derive(Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialEq, PartialOrd, Eq, Debug)]
pub struct VirtPageNum(pub usize);


impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtAddr({:#x})", self.0)
    }
}


/// T: {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum}
/// T -> usize: T.0
/// usize -> T: usize.into()

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        Self (value & PPN_MASK)
    }
}
                                                                                                                                                                                                                                                                                                                                                                   

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self {
        Self (value & PA_MASK)
    }
}

impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self {
        Self(value & VA_MASK)
    }
}


impl From<usize> for VirtPageNum {
    fn from(value: usize) -> Self {
        Self (value & VPN_MASK)
    }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self { 
        v.0 
    }
}


impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self { 
        v.0 
    }
}


impl From<VirtAddr> for usize {
    /// Converts an Sv39 virtual address to a canonical 64-bit usize representation
    /// 
    /// # Safety
    /// 
    /// ## Input Requirements
    /// - The input address must comply with RISC-V Sv39 virtual memory conventions
    /// - Bits 39-63 (25 MSBs) must either be:
    ///   - All zeros (user-space canonical form), OR  
    ///   - All ones (kernel-space canonical form)
    ///
    /// ## Behavior Guarantees
    /// - Invalid upper bits (39-63) are truncated via `& VA_MASK` before processing
    /// - Preserves Sv39 sign-extension semantics required by hardware page table walkers
    /// - Returns architecturally valid 64-bit addresses as defined in §4.3.1 of RISC-V Privileged Spec
    fn from(value: VirtAddr) -> Self { 
        const SIGN_BIT_MASK: usize = 1 << (VA_WIDTH - 1);
         // Defense-in-depth: Strip non-address bits before processing
         let sanitized = value.0 & VA_MASK;

         // Sv39 sign-extension rules (§4.3.1)
         if sanitized & SIGN_BIT_MASK != 0 {
             // Kernel-space: Propagate sign bit to upper 25 bits
             sanitized | !((1 << VA_WIDTH) - 1)
         } else {
             // User-space: Upper bits remain zero
             sanitized
         }
    }
}

impl From<VirtPageNum> for usize {
    fn from(value: VirtPageNum) -> Self {
        value.0
    }
}


// const PAGE_SIZE_MASK: usize = PAGE_SIZE - 1;
impl VirtAddr {
    /// Maximum allowable virtual address (architecture-dependent)
    pub const MAX: VirtAddr = VirtAddr(MAX_VA);

    /// Maximum user-space virtual address (bits 37:0 all 1 for Sv39)
    /// This represents the highest address accessible in user mode
    pub const USER_MAX: VirtAddr = VirtAddr((1<< (VA_WIDTH - 1)) - 1);

    /// Creates a new virtual address from raw usize value
    /// 
    /// # Arguments
    /// * `addr` - Raw virtual address value
    /// 
    /// # Note
    /// Does not perform any validity checks on the address
    pub fn new(addr: usize) -> Self{
        Self(addr)
    }


    /// Rounds down to the nearest page-aligned address
    /// 
    /// # Example
    /// ```
    /// let addr = VirtAddr::new(0x1234_5678);
    /// assert_eq!(addr.round_down(), VirtAddr::new(0x1234_5000));
    pub fn round_down(&self) -> Self {
        Self ( self.0 & !(OFFSET_MASK) ) 
    }

    /// Converts to virtual page number by truncating lower bits
    /// (Equivalent to floor(address / PAGE_SIZE))
    pub fn down_to_vpn(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    /// Converts to virtual page number by rounding up
    /// (Equivalent to ceil(address / PAGE_SIZE))
    pub fn up_to_vpn(&self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    /// Extracts the page offset portion of the address
    /// (Lower bits not used for page number translation)
    pub fn page_offset(&self)  -> usize{
        self.0 & OFFSET_MASK
    }

    /// Checks if the address is page-aligned
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }

    /// Determines if this is a valid user-space address
    /// 
    /// # Architecture Notes
    /// For Sv39:
    /// - Valid user addresses have bits 63:39 equal to 0
    /// - The user/kernel bit (bit 63 in Sv39) must be 0
    pub fn is_user(&self) -> bool {
        let high_bits_is_valid = (self.0 & HIGH_BITS_MASK) == VALID_USER_HIGH_BITS;
        let is_in_user = (self.0 >> USER_HIGH_BIT) & 1;
        high_bits_is_valid && is_in_user == 0
    }

    /// Determines if this is a valid kernel-space address
    /// 
    /// # Architecture Notes
    /// For Sv39:
    /// - Valid kernel addresses have bits 63:39 equal to 0x1FFFF (sign-extended)
    /// - The user/kernel bit (bit 63 in Sv39) must be 1
    pub fn is_kernel(&self) -> bool {
        let high_bits_is_valid = (self.0 & HIGH_BITS_MASK) == VALID_KERNEL_HIGH_BITS;
        let is_in_kernel = (self.0 >> KERNEL_HIGH_BIT) & 1 != 0;
        high_bits_is_valid && is_in_kernel
    }
}


impl From<VirtAddr> for VirtPageNum {
    fn from(value: VirtAddr) -> Self {
        assert_eq!(value.page_offset(), 0);
        value.down_to_vpn()
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(value: VirtPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BITS)   
    }
}


impl PhysAddr {
    #[inline]
    pub fn page_offset(&self) -> usize {
        self.0 & OFFSET_MASK
    }

    pub fn down_to_ppn(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    pub fn up_to_ppn(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(value: PhysAddr) -> Self {
        assert_eq!(value.page_offset(), 0);
        value.down_to_ppn()
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(value: PhysPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BITS)
    }
}

impl Add<usize> for PhysAddr {
    type Output = Self;

    fn add(self, offset: usize) -> Self::Output {
        Self(self.0 + offset)
    }
}



impl VirtPageNum {
    const LEVEL_MASK: usize = 0x1FF;
    const PPTE_OFFSET: usize = 0;
    const PMD_OFFSET: usize = 9;
    const PGD_OFFSET : usize = 18;

    /// |26~18|17~9|8~0|
    /// |pgd | pmd | ppte |
    pub fn get_pgd(&self) -> PageTableLevel {
        PageTableLevel::Pgd(
            self.extract_level(Self::PGD_OFFSET)
        )
    }

    pub fn get_pmd(&self) -> PageTableLevel {
        PageTableLevel::Pmd(
            self.extract_level(Self::PMD_OFFSET)
        )
    }

    pub fn get_ppte(&self) -> PageTableLevel {
        PageTableLevel::PPte(
            self.extract_level(Self::PPTE_OFFSET)
        )
    }

    /// Extracts an index from the virtual page number (VPN) based on the given offset.
    ///
    /// # Parameters
    /// - `offset`: The offset of the index in the VPN.
    ///
    /// # Returns
    /// - The extracted index.
    #[inline]
    fn extract_level(&self, offset: usize) -> usize {
        (self.0 >> offset) & Self::LEVEL_MASK
    }

    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }


    /// Returns a `PageTableLevelIterator` for traversing the page table hierarchy.
    ///
    /// The iterator follows the order of the page table levels:
    /// 1. Page Global Directory (PGD)
    /// 2. Page Middle Directory (PMD)
    /// 3. Page Table Entry (PTE)
    ///
    /// This iterator is used to traverse the multi-level page table hierarchy
    /// starting from the root level (PGD) down to the leaf level (PTE).
    ///
    /// # Returns
    /// - A `PageTableLevelIterator` that can be used to iterate over the page table levels.
    pub fn get_ptl_iter(&self) -> PageTableLevelIterator {
        PageTableLevelIterator::new(*self)
    }

}



impl PhysPageNum {
    // pub fn get_bytes_array1(&self) -> &'static mut [u8; PAGE_SIZE]{
    //     let base = self.0 << PAGE_SIZE_BITS;
    //     let memory_ptr = base as *mut u8;

    //     unsafe {
    //         &mut *(memory_ptr as *mut [u8; PAGE_SIZE])
    //     }
    // }

    pub fn get_ptes_slice(&self) -> &'static mut [PageTableEntry] {

        // `into` ensure align
        let pa: PhysAddr = (*self).into();
        let entries_count = PAGE_SIZE / core::mem::size_of::<PageTableEntry>();

        unsafe {
            core::slice::from_raw_parts_mut(
                pa.0 as *mut PageTableEntry,
                entries_count,
            )
        }
    }

    /// Get a mutable reference to the entire physical page as a byte slice.
    ///
    /// # Safety
    /// - The `PhysPageNum` must refer to a valid and allocated physical page.
    /// - The returned slice is mutable, so changes to it will directly affect the memory.
    /// - Ensure that the physical address is properly aligned and within the valid address range.
    pub fn get_bytes_array_slice(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();

        // Ensure that the physical address is aligned to the page size.
        assert!(
            pa.aligned(),
            "Unaligned physical address: {:#x}",
            pa.0
        );

        unsafe {
            // Create a mutable byte slice from the physical address
            core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE)
        }
    }


    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        unsafe {
            (pa.0 as *mut T).as_mut().unwrap()
        }
    }
}

pub trait StepByOne {
    fn step(&mut self);
}

impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Copy, Clone)]
// A simple range structure for type T
pub struct SimpleRange<T>
where 
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug
{
    start: T,
    end: T,
}

impl<T>  SimpleRange<T>
where 
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self {start, end}
    }

    pub fn get_start(&self) -> T {
        self.start
    }

    pub fn get_end(&self) -> T {
        self.end
    }
}


impl<T> IntoIterator for SimpleRange<T> 
where 
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.start, self.end)
    }
}

/// Iterator for the simple range structure
pub struct SimpleRangeIterator<T>
where 
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug 
{
    current: T,
    end: T,
}


impl<T> SimpleRangeIterator<T> 
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug
{
    pub fn new(start: T, end: T) -> Self {
        Self { current: start, end: end}
    }
}


impl<T> Iterator for SimpleRangeIterator<T>
where 
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug 
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}

/// a simple range structure for virtual page number
pub type VPNRange = SimpleRange<VirtPageNum>;



#[kernel_test]
fn test_virt_addr() {
    #[cfg(feature = "sv39")]
    {
        // 合法用户地址（高位全 0，第 38 位为 0）
        let user_va = VirtAddr(0x0000_003F_FFFF_FFFF);
        assert!(user_va.is_user());   // ✅
        assert!(!user_va.is_kernel());

        // 合法内核地址（高位全 1，第 38 位为 1）
        let kernel_va = VirtAddr(0xFFFF_FFC0_1234_5678);
        assert!(kernel_va.is_kernel()); // ✅
        assert!(!kernel_va.is_user());

        // 非法地址（高位不全 0 或 1）
        let invalid_va = VirtAddr(0x0000_0040_0000_0000);
        assert!(!invalid_va.is_user()); // ✅
        assert!(!invalid_va.is_kernel());
    }
    #[cfg(feature = "sv48")]
    {
        // 合法用户地址（高位全 0，第 47 位为 0）
        let user_va = VirtAddr(0x0000_0FFF_FFFF_FFFF);
        assert!(user_va.is_user());   // ✅
        assert!(!user_va.is_kernel());

        // 合法内核地址（高位全 1，第 47 位为 1）
        let kernel_va = VirtAddr(0xFFFF_8000_1234_5678);
        assert!(kernel_va.is_kernel()); // ✅
        assert!(!kernel_va.is_user());

        // 非法地址（高位不全 0 或 1）
        let invalid_va = VirtAddr(0x1000_0040_0000_0000);
        assert!(!invalid_va.is_user()); // ✅
        assert!(!invalid_va.is_kernel());
    }
}