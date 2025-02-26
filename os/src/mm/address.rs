use core::fmt::{self, Debug};

use crate::config::*;

use super::page_table::PageTableEntry;

#[derive(Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub struct VirtPageNum(pub usize);

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#X}", self.0))
    }
}


impl Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
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

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self { v.0 }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self { v.0 }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self { v.0 }
}

impl PhysAddr {
    pub fn page_offset(&self) -> usize {
        self.0 & OFFSET_MASK
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

impl PhysAddr {
    pub fn down_to_ppn(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    pub fn up_to_ppn(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
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

    pub fn get_pte_slice(&self) -> &'static mut [PageTableEntry] {

        // `into` ensure align
        let pa: PhysAddr = self.clone().into();
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
        let pa: PhysAddr = self.clone().into();

        // Ensure that the physical address is aligned to the page size.
        assert!(
            pa.0 % PAGE_SIZE == 0,
            "Unaligned physical address: {:#x}",
            pa.0
        );

        unsafe {
            // Create a mutable byte slice from the physical address
            core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE)
        }
    }


    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = self.clone().into();
        unsafe {
            (pa.0 as *mut T).as_mut().unwrap()
        }
    }
}


impl VirtAddr {
    pub fn down_to_vpn(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    pub fn up_to_vpn(&self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
}

impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & VPN_MASK;
            vpn >>= VPN_WIDTH;
        }
        idx
    }
}


/// a simple range structure for virtual page number
pub type VPNRange = SimpleRange<VirtPageNum>;

