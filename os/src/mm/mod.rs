pub mod memory_set;
pub mod heap_allocator;
pub mod address;
pub mod page_table;
pub mod frame_allocator;
pub mod map_area;
pub mod user_ptr;
mod error;
// pub mod user;
// mod buffer;


pub use memory_set::KERNEL_SPACE;

pub use user_ptr::UserBuffer;



pub fn init() {
    log::info!("Memory manager initializing.");
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.lock().activate();
    log::info!("Memory manager initialized successfully.");
}

