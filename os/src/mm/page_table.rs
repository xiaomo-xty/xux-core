//! This module implements a simple page table structure for managing virtual-to-physical address translations in a virtual memory system.
//! The page table follows a 3-level hierarchical structure (SV39 or similar) and supports basic operations like mapping, unmapping, and translating virtual addresses.
//! It uses a custom `PageTableEntry` structure, which represents the entries in the page table. Each entry contains a physical page number (PPN) and a set of flags.
//! The `PTEFlags` bitflags are used to define various entry attributes, such as validity (`V`), read/write permissions (`R`, `W`), and other control flags.
//! The page table also supports manual creation of page tables based on a provided SATP (Supervisor Address Translation and Protection) token.
//! A custom frame allocator (`frame_alloc`) is used to allocate new frames for page table entries as needed.

use alloc::vec;
use alloc::vec::Vec;

use bitflags::*;

// Constants related to SATP (used to mask the PPN in the SATP register)
use crate::{config::SATP_PPN_MASK, println};

// Related modules for address and frame allocation
use super::{address::{PhysPageNum, VirtPageNum}, frame_allocator::{frame_alloc, FrameTracker}};

// Define the PTEFlags bitflags for page table entry attributes
bitflags!{
    pub struct PTEFlags: u16 {
        const V   = 1 << 0; // Valid flag
        const R   = 1 << 1; // Read flag
        const W   = 1 << 2; // Write flag
        const X   = 1 << 3; // Execute flag
        const U   = 1 << 4; // User flag
        const G   = 1 << 5; // Global flag
        const A   = 1 << 6; // Accessed flag
        const D   = 1 << 7; // Dirty flag
        const RSW0 = 1 << 8; // Reserved flag 0
        const RSW1 = 1 << 9; // Reserved flag 1
        const RSW = Self::RSW0.bits | Self::RSW1.bits; // Reserved flags mask
    }
}

// PageTableEntry structure representing a page table entry
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize, // The raw bits of the page table entry, holding PPN and flags
}

#[allow(dead_code)]
impl PageTableEntry {
    const FLAGS_SIZE: usize = 10; // The size of the flags portion of the entry
    const PPN_SIZE: usize = 44; // The size of the PPN portion of the entry
    const FLAGS_MASK: u16 = 1 << (Self::FLAGS_SIZE - 1); // Mask for extracting flags

    /// Creates a new page table entry with the provided physical page number (PPN) and flags.
    ///
    /// This method combines the given PPN and flags into a single `PageTableEntry` structure.
    /// The PPN is shifted by the size of the flags to create the final bit representation of the entry.
    ///
    /// # Parameters:
    /// - `ppn`: The physical page number to store in the page table entry.
    /// - `flags`: The flags that represent the attributes of the page table entry (e.g., read/write permissions).
    ///
    /// # Returns:
    /// A `PageTableEntry` with the combined PPN and flags.
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << Self::FLAGS_SIZE | flags.bits as usize, 
        }
    }

    /// Creates an empty (zeroed) page table entry.
    ///
    /// This method is used to create a page table entry where all the bits are set to zero,
    /// representing an invalid or uninitialized entry.
    ///
    /// # Returns:
    /// A `PageTableEntry` with all bits set to zero.
    pub fn empty() -> Self {
        PageTableEntry {
            bits: 0,
        }
    }

    /// Extracts the physical page number (PPN) from the page table entry.
    ///
    /// This method retrieves the PPN from the stored bits by shifting and masking to extract
    /// the relevant portion of the entry.
    ///
    /// # Returns:
    /// The `PhysPageNum` that was stored in the entry.
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> Self::FLAGS_SIZE & ((1usize << Self::PPN_SIZE) - 1)).into()
    }

    /// Extracts the flags from the page table entry.
    ///
    /// This method retrieves the flags from the entry by masking and interpreting the relevant bits.
    ///
    /// # Returns:
    /// The `PTEFlags` associated with the page table entry.
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u16 & Self::FLAGS_MASK).unwrap()
    }

    /// Checks if the page table entry is valid.
    ///
    /// This method checks if the entry's validity flag (`V`) is set. If the flag is set, it means
    /// that the page is valid and can be used for address translation.
    ///
    /// # Returns:
    /// `true` if the entry is valid, otherwise `false`.
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
}

// Test function to print the flags of a PTE
#[allow(non_snake_case)]
pub fn test_PTEFlags() {
    let empty_flag = PTEFlags::empty();
    println!("{}", empty_flag.bits());
}

#[derive(Clone, Copy)]
// PageTable structure representing the multi-level page table
pub struct PageTable {
    root_ppn: PhysPageNum, // The root page table's PPN
    frames: Vec<FrameTracker>, // A list of frame trackers for the allocated pages
}

impl PageTable {
    /// Creates a new page table with an allocated root page frame.
    ///
    /// This function allocates a frame for the root page table and initializes the page table.
    /// It also stores the allocated frame in the `frames` vector for future management.
    ///
    /// # Returns:
    /// A new `PageTable` with a valid root PPN and an empty list of frames.
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
}

// Operations for mapping, unmapping, and translating virtual pages
impl PageTable {
    /// Maps a virtual page number (VPN) to a physical page number (PPN) with given flags.
    ///
    /// This method finds or creates a page table entry for the given VPN. If the VPN is not already
    /// mapped, it creates a new entry and sets the appropriate flags for the PTE.
    ///
    /// # Parameters:
    /// - `vpn`: The virtual page number to map.
    /// - `ppn`: The physical page number to map to the VPN.
    /// - `flags`: The flags that represent the attributes of the page table entry (e.g., read/write permissions).
    ///
    /// # Panics:
    /// This function will panic if the VPN is already mapped (i.e., the entry is already valid).
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.fine_pte_or_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    /// Unmaps a virtual page number (VPN).
    ///
    /// This method unmaps the given VPN by setting its page table entry to an empty entry (all bits zero).
    ///
    /// # Parameters:
    /// - `vpn`: The virtual page number to unmap.
    ///
    /// # Panics:
    /// This function will panic if the VPN is invalid (i.e., the entry is not valid before unmapping).
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

// Internal helper functions for managing page table entries (PTEs)
impl PageTable {
    /// Finds an existing page table entry (PTE) or creates a new one if necessary.
    ///
    /// This method traverses the page table structure, following the multi-level page table hierarchy,
    /// and either returns an existing entry or creates new entries as needed. It allocates frames for
    /// new entries when necessary and adds them to the list of frames.
    ///
    /// # Parameters:
    /// - `vpn`: The virtual page number whose page table entry (PTE) is to be found or created.
    ///
    /// # Returns:
    /// - An `Option` containing a mutable reference to the `PageTableEntry` if found or created.
    fn fine_pte_or_create(&mut self, vpn: VirtPageNum)
    -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;

        // Traverse the page table hierarchy, 3 levels deep
        for n in 0..3 {
            let idx = idxs[n]; // Get the index for the current level
            let ptes = ppn.get_pte_slice(); // Get the slice of page table entries at this level
            let pte = &mut ptes[idx]; // Get the specific entry for this index

            if !pte.is_valid() {
                // If the entry is not valid, allocate a new frame and initialize the entry
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }

            if n == 2 {
                // If we are at the last level, return the found PTE
                result = Some(pte);
                break;
            }

            // Move to the next level by setting the physical page number to the next PPN
            ppn = pte.ppn();
        }
        result
    }

    /// Finds the page table entry (PTE) for a given virtual page number (VPN).
    ///
    /// This method traverses the page table hierarchy and returns a reference to the page table entry
    /// for the given VPN. It stops at the last level of the page table hierarchy, which is where the actual
    /// page mapping exists. If an invalid entry is encountered at any level, `None` is returned.
    ///
    /// # Parameters:
    /// - `vpn`: The virtual page number whose page table entry (PTE) is to be found.
    ///
    /// # Returns:
    /// - An `Option` containing a mutable reference to the `PageTableEntry` if found, or `None` if invalid.
    fn find_pte(&self, vpn: VirtPageNum)
    -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes(); // Get the indices for each level of the page table hierarchy
        let mut ppn = self.root_ppn; // Start from the root page table's physical page number
        let mut result: Option<&mut PageTableEntry> = None;

        // Traverse the page table hierarchy, 3 levels deep
        for n in 0..3 {
            let idx = idxs[n]; // Get the index for the current level
            let ptes = ppn.get_pte_slice(); // Get the slice of page table entries at this level
            let pte = &mut ptes[idx]; // Get the specific entry for this index

            if n == 2 {
                // If we are at the last level, return the found PTE
                result = Some(pte);
                break;
            }

            if !pte.is_valid() {
                // If an entry is invalid, return `None` as the mapping doesn't exist
                return None;
            }

            // Move to the next level by setting the physical page number to the next PPN
            ppn = pte.ppn();
        }
        result
    }

    /// Creates a page table from a given SATP token.
    ///
    /// This method creates a `PageTable` structure by extracting the root page table's physical page number
    /// from the provided SATP (Supervisor Address Translation and Protection) token. The `frames` vector is
    /// empty since this page table is used for translation purposes and does not manage actual frames.
    ///
    /// # Parameters:
    /// - `satp`: The SATP token that contains the root page table's physical page number.
    ///
    /// # Returns:
    /// - A `PageTable` initialized with the root page table's PPN extracted from the SATP token.
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & SATP_PPN_MASK), // Extract the PPN from SATP
            frames: Vec::new(), // No frames are allocated for this page table
        }
    }

    /// Translates a virtual page number (VPN) to a page table entry (PTE).
    ///
    /// This method attempts to find the page table entry for a given VPN by traversing the page table hierarchy.
    /// If the mapping exists and is valid, it returns a copy of the corresponding page table entry. Otherwise,
    /// it returns `None`.
    ///
    /// # Parameters:
    /// - `vpn`: The virtual page number to translate.
    ///
    /// # Returns:
    /// - An `Option` containing a `PageTableEntry` if found and valid, otherwise `None`.
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn)
            .map(|pte| pte.clone()) // If found, return a copy of the PTE
    }
}


