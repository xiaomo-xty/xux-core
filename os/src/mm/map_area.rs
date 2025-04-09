use alloc::collections::btree_map::BTreeMap;
use bitflags::bitflags;

use crate::{config::PAGE_SIZE, mm::address::StepByOne};

use super::{
    address::{
        PhysPageNum, 
        VPNRange, 
        VirtAddr, 
        VirtPageNum
    }, 
    frame_allocator::{
        frame_alloc, 
        FrameTracker
    }, 
    page_table::{
        PTEFlags, 
        PageTable
    }
};

pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

impl MapArea {
    pub fn new (
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.down_to_vpn();
        let end_vpn: VirtPageNum = end_va.up_to_vpn();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }

    #[allow(unused)]
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn)
        }
    }


    pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();

        loop {
            let src = &data[start .. len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .find_pte_by_vpn(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array_slice()[..src.len()];

            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }

    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0)
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }

        let pte_flags = PTEFlags::from_bits(self.map_perm.bits.into()).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            MapType::Framed => {
                self.data_frames.remove(&vpn);
            }
            _ => {}
        }
        page_table.unmap(vpn);
    }

    pub fn get_vpn_end(&self) -> VirtPageNum {
        self.vpn_range.get_end()
    }

    // pub fn get_vpn_start(&self) -> VirtPageNum {
    //     self.vpn_range.get_start()
    // }
}