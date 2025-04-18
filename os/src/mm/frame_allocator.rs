use alloc::{fmt, format, vec::Vec};
use lazy_static::lazy_static;
use crate::{config::PHYSTOP, mm::address::PhysAddr, println, sync::spin::mutex::IRQSpinLock};

use super::address::PhysPageNum;

type FrameAllocatorImpl = StackFrameAllocator;


lazy_static! {
    pub static ref FRAME_ALLOCATOR: IRQSpinLock<FrameAllocatorImpl> =
        { 
            log::info!("Initialize FRAME_ALLOCATOR");
            IRQSpinLock::new(FrameAllocatorImpl::new())
        };
}

pub fn init_frame_allocator() {

    log::info!("Frame allocator initializing.");
    extern "C" {
        fn ekernel();
    }

    log::debug!("cao");
    FRAME_ALLOCATOR
        .lock()
        .init(PhysAddr::from(ekernel as usize).up_to_ppn(), PhysAddr::from(PHYSTOP).down_to_ppn());

    log::info!("Frame allocator initialized successfully.");
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .lock()
        .alloc()
        .map(|ppn| FrameTracker::new(ppn))
}

fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR
        .lock()
        .dealloc(ppn);
}


pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        let bytes_array = ppn.get_bytes_array_slice();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}


impl fmt::Debug for FrameTracker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // We can output the page number and a truncated version of the page contents
        let page_data = self.ppn.get_bytes_array_slice();

        // Print the PhysPageNum (page number) and the first few bytes of the page content
        f.debug_struct("FrameTracker")
            .field("ppn", &self.ppn.0) // Display the page number
            .field("page_data", &format!("{:?}", &page_data[..16])) // Display first 16 bytes of the page data
            .finish()
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}


trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}


impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            if self.current == self.end {
                None
            }
            else {
                self.current += 1;
                Some((self.current - 1).into())
            }
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;

        if ppn >= self.current || self.recycled.contains(&ppn) {
                panic!("Frame ppn={:#x} has not been allocated!", ppn)
        }
        self.recycled.push(ppn);
    }

}


#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();

    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }

    v.clear();

    drop(v);
    println!("frame_allocator_test passed!")
}