use core::arch::asm;

use alloc::{sync::Arc, vec::Vec};

use lazy_static::lazy_static;
use riscv::register::satp;

use crate::{
    board::MMIO, 
    config::{PAGE_SIZE, PHYSTOP, TRAMPOLINE}, 
    mm::map_area::{MapArea, MapPermission, MapType}, 
    sync::spin::mutex::SpinMutex, 
};

use super::{
    address::{PhysAddr, VPNRange, VirtAddr, VirtPageNum}, page_table::{PTEFlags, PageTable, PageTableEntry}
};

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc< SpinMutex<MemorySet> > =
        Arc::new(
            SpinMutex::new(
                MemorySet::new_kernel()
            )
        );
}

struct UserMemorySetInfo {
    stack_range: VPNRange,
    // stack: Arc<MapArea>,
    // heap: VPNRange,
    // task_size: usize,
    // total_virtual_memory: usize,
    // data_virtual_memory: usize,
    // exec_virtual_memory: usize,

    // pub mapped_files: Vec<Arc<dyn File>>,   // 已映射的文件（类比 exe_file）
    // exec_file: Arc<dyn File>,
    // binfmt: BinaryFmt,
}


pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
    user_info: Option<UserMemorySetInfo>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        log::debug!("new bare");
        let a = Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
            user_info: None,
        };
        log::debug!("new bare end");
        a
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    // push something data or not to the map area
    // if the data is none, just mapping
    // or copy the data to the virtual memory.
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&self.page_table, data)
        }

        self.areas.push(map_area);
    }

    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        log::debug!("insert {:?}~{:?}", start_va, end_va);
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }


    pub fn remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum) {
        if let Some((idx, area)) = self
            .areas
            .iter_mut()
            .enumerate()
            .find(|(_, area)| area.get_vpn_range().get_start() == start_vpn)
        {
            area.unmap(&mut self.page_table);
            self.areas.remove(idx);
        }
    }

    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            // sync
            asm!("sfence.vma");
        }
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.find_pte_by_vpn(vpn)
    }

    pub fn new_kernel() -> Self {
        log::info!("New kernel starting.");
        let mut memory_set = Self::new_bare();

        log::info!("Map trampoline.");
        memory_set.map_trampoline();

        log::info!(".text   [{:#x}, {:#x}]", stext as usize, etext as usize);
        log::info!(".rodata [{:#x}, {:#x}]", srodata as usize, erodata as usize);
        log::info!(".data   [{:#x}, {:#x}]", sdata as usize, edata as usize);
        log::info!(
            ".bsss   [{:#x}, {:#x}]",
            sbss_with_stack as usize,
            ebss as usize
        );

        log::info!("Mapping .text section with identity mapping...");
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );

        log::info!("Mapping .rodata section with identity mapping...");
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );

        log::info!("Mapping .data section with identity mapping...");
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );

        log::info!("Mapping .bss section with identity mapping...");
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        log::info!("Mapping .physical section with identity mapping...");
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                PHYSTOP.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        log::info!("mapping memory-mapped registers");
        for pair in MMIO {
            memory_set.push(
                MapArea::new(
                    (*pair).0.into(),
                    ((*pair).0 + (*pair).1).into(),
                    MapType::Identical,
                    MapPermission::R | MapPermission::W,
                ),
                None,
            );
        }
        log::info!("New kernel sucessfully.");
        memory_set
    }
    
}

impl MemorySet {
    
    pub fn new_user() -> Self {
        let mut memory_set = Self::new_bare();

        memory_set.map_trampoline();


        memory_set


    }

    
    /// Maps the user-space trampoline page to the kernel's trampoline code.
    ///
    /// # Design Rationale
    /// This exists to enable safe privilege escalation from user-space to kernel-space:
    /// 1. ​**Identity Mapping in Kernel**:
    ///    - The kernel uses 1:1 virtual-to-physical address mapping during early boot.
    ///    - Thus, the symbol `strampoline` (a kernel virtual address) directly represents its physical
    ///      location, allowing direct conversion via `PhysAddr::from()`.
    ///
    /// 2. ​**Trampoline Contract**:
    ///    - User-space expects system call/exception entry points at a fixed virtual address `TRAMPOLINE`
    ///      (e.g., 0xFFFF_FFFF_FFFF_F000), as defined by the ABI.
    ///    - By mapping `TRAMPOLINE` (user VA) → `strampoline` (kernel PA), we create a controlled gateway
    ///      for user-space to trigger kernel handlers without exposing full kernel memory.
    ///
    /// 3. ​**Security Through Permissions**:
    ///    - `R | X` flags allow execution but prevent modification: user-space can jump to this page
    ///      but cannot alter its content (critical for preventing code injection).
    ///
    /// # Notes for Future Maintenance
    /// - This depends on the kernel's identity mapping remaining valid. If the kernel later switches to
    ///   a non-identity-mapped page table, `strampoline` must be converted to its true physical address.
    /// - `TRAMPOLINE` must match the address expected by user-space binaries (defined in ABI constants).
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }

}

impl MemorySet {
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_user();

        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        /// for the string <kbd>DEL</kbd>`ELF`.
        static MAGIC: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];
        assert_eq!(magic, MAGIC, "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                };
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                };
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                };
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);

                max_end_vpn = map_area.get_vpn_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        let max_end_va: VirtAddr = max_end_vpn.into();
        // Div by guard page
        let user_stack_base: usize = usize::from(max_end_va) + PAGE_SIZE;
        

        (
            memory_set,
            user_stack_base,
            elf.header.pt2.entry_point() as usize,
        )
    }


    pub fn from_other_user(user_space: &MemorySet) -> MemorySet {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // copy data sections/trap_context/user_stack
        for area in user_space.areas.iter() {
            let new_area = MapArea::from_other(area);
            memory_set.push(new_area, None);
            // copy data from another space
            for vpn in area.get_vpn_range() {
                let src_ppn = user_space.translate(vpn).unwrap().ppn();
                let dst_ppn = memory_set.translate(vpn).unwrap().ppn();
                dst_ppn
                    .get_bytes_array_slice()
                    .copy_from_slice(src_ppn.get_bytes_array_slice());
            }
        }
        memory_set
    }
}

pub fn remap_test() {
    log::info!("Remap test starting");
    let kernel_space = KERNEL_SPACE.lock();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert_eq!(
        kernel_space
            .page_table
            .find_pte_by_vpn(mid_text.down_to_vpn())
            .unwrap()
            .writable(),
        false
    );

    assert_eq!(
        kernel_space
            .page_table
            .find_pte_by_vpn(mid_rodata.down_to_vpn())
            .unwrap()
            .writable(),
        false
    );

    assert_eq!(
        kernel_space
            .page_table
            .find_pte_by_vpn(mid_data.down_to_vpn())
            .unwrap()
            .executable(),
        false
    );

    log::info!("Remap test passed!");
}
