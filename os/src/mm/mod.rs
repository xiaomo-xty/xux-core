pub mod memory_set;
pub mod heap_allocator;
pub mod address;
pub mod page_table;
pub mod frame_allocator;
pub mod map_area;
// pub mod error;
// pub mod user;
// mod buffer;


pub use memory_set::KERNEL_SPACE;



pub fn init() {
    log::info!("Memory manager initializing.");
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
    log::info!("Memory manager initialized successfully.");
}

