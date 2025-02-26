use buddy_system_allocator::LockedHeap;
use crate::{config::KERNEL_HEAP_SIZE, println};



#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap =  LockedHeap::empty();

// static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];


#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> !{
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[allow(static_mut_refs)]
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR.
            lock().
            init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}



#[allow(unused)]
pub fn heap_test() {
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
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }

    for i in 0..500 {
        assert_eq!(v[i], i);
    }

    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}