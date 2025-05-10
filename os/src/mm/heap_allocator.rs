use core::{alloc::{GlobalAlloc, Layout}, ops::Deref, ptr::NonNull};



use buddy_system_allocator::Heap;
use os_macros::kernel_test;
use spin::Mutex;
use crate::{config::KERNEL_HEAP_SIZE, println, sync::spin::{mutex::SpinLock, ticket::{IRQTicketMutex, TicketMutex}}};

type HeapLock<T> = IRQTicketMutex<T>;

pub struct LockedHeap(HeapLock<Heap>);

impl LockedHeap {
    /// Creates an empty heap
    pub const fn new() -> LockedHeap {
        LockedHeap(HeapLock::new(Heap::new()))
    }

    /// Creates an empty heap
    pub const fn empty() -> LockedHeap {
        LockedHeap(HeapLock::new(Heap::new()))
    }
}

impl Deref for LockedHeap {
    type Target = HeapLock<Heap>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        log::debug!("alloc");
        self.0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout);
        log::debug!("dealloc");
    }
}

/// I should implement a slab allcator
/// Request space for buddy dynamiclly
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap =  LockedHeap::empty();

// static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];


#[allow(static_mut_refs)]
pub fn init_heap() {
    log::info!("heap allocator initializing.");
    unsafe {
        HEAP_ALLOCATOR.
            lock().
            init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
    log::info!("heap allocator initialized successfully.");
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> !{
    let allocator = HEAP_ALLOCATOR.lock();
    let used_total = allocator.stats_alloc_actual();
    let used_user = allocator.stats_alloc_user();
    let total = allocator.stats_total_bytes();
    let free = total - used_total;
    log::error!(
        "Heap allocation failed:
        [Requested]:
            size:        {:>10.2} bytes
            align:       {:>10.2} 
        [Heap usage]:
            Used (total):{:>10.2} bytes
            Used (user): {:>10.2} bytes
            Free:        {:>10.2} bytes
            Total:       {:>10.2} bytes",
        layout.size(),  // allocating request size
        layout.align(), // align format
        used_total,     // used total
        used_user,
        free,
        total
    );
    panic!("Heap allocation error, layout = {:?}", layout);
}


pub fn heap_test() {
    log::info!("==========heap test start================");
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
    let bss_range = sbss as usize..ebss as usize;
    // log::info!("HEAP_SPACE start from : {:#X}", HEAP_SPACE.as_ptr() as usize);
    log::info!("sbss: {:#X} - ebss: {:#X}", sbss as usize, ebss as usize);
    let a = Box::new(5);
    assert_eq!(*a, 5);
    println!("a at {:#X}",&(a.as_ref() as *const _ as usize));
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);

    log::info!("Vec::new()");
    let mut v: Vec<usize> = Vec::new();
    log::info!("push 500 usize");
    for i in 0..500 {
        v.push(i);
    }

    for i in 0..500 {
        assert_eq!(v[i], i);
    }

    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    log::info!("==============heap_test passed!=========================");
}



#[kernel_test]
pub fn test_dead_lock_in_interrupt() {
    use core::arch::asm;

    let gurad = HEAP_ALLOCATOR.lock();
    let gurad2 = HEAP_ALLOCATOR.lock();
    unsafe {
        core::arch::asm!(
            "ecall",    // 插入 ecall 指令
        );
    }
    println!("pass");
}