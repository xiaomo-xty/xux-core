use core::arch::asm;

use alloc::{sync::Arc, vec::Vec};

use lazy_static::lazy_static;
use riscv::register::satp;

use crate::{
    board::MMIO,
    config::{MEMORY_END, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE},
    mm::map_area::{MapArea, MapPermission, MapType},
    sync::UPSafeCell,
};

use super::{
    address::{PhysAddr, VirtAddr, VirtPageNum},
    page_table::{PTEFlags, PageTable, PageTableEntry},
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
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(unsafe { UPSafeCell::new(MemorySet::new_kernel()) });
}

pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
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
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }

    pub fn activate(&self) {
        log::info!("KERNEL SPACE activing.");
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            // sync
            asm!("sfence.vma");
        }
        log::info!("KERNEL SPACE actived successfully.");
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }
}

impl MemorySet {
    pub fn new_kernel() -> Self {
        log::info!("New kernel starting.");
        let mut memory_set = Self::new_bare();
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
                MEMORY_END.into(),
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
        let mut memory_set = Self::new_bare();

        memory_set.map_trampoline();

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
        //  Map user stack wityh U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();

        // Guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        let guard_page_map = MapArea::new(
            user_stack_bottom.into(),
            user_stack_top.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        );
        memory_set.push(guard_page_map, None);

        let map_trap_ctx = MapArea::new(
            TRAP_CONTEXT.into(),
            TRAMPOLINE.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W,
        );

        memory_set.push(map_trap_ctx, None);
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }
}

pub fn remap_test() {
    log::info!("Remap test starting");
    let kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_text.down_to_vpn())
            .unwrap()
            .writable(),
        false
    );

    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_rodata.down_to_vpn())
            .unwrap()
            .writable(),
        false
    );

    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_data.down_to_vpn())
            .unwrap()
            .executable(),
        false
    );

    log::info!("Remap test passed!");
}
