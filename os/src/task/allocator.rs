use core::sync::atomic::{ AtomicUsize, Ordering};

use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;

use crate::{config::{KERNEL_STACK_BASE, KERNEL_STACK_SIZE, PAGE_SIZE, TRAP_CONTEXT_START, USER_STACK_SIZE}, mm::{address::{PhysPageNum, VirtAddr, VirtPageNum}, map_area::MapPermission, memory_set::MemorySet, KERNEL_SPACE}, sync::spin::mutex::IRQSpinLock, trap::TrapContext};




lazy_static! {
    static ref TID_ALLOCATOR: RecycleAllocator = RecycleAllocator::new();
    static ref KERNEL_STACK_ID_ALLOCATOR: RecycleAllocator = RecycleAllocator::new();
}



#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TaskID(usize);


pub struct TaskHandleAllocator;
impl TaskHandleAllocator {
    pub fn allocate() -> TaskHandle{
        TaskHandle::new()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TaskHandle(TaskID);


impl TaskHandle {

    fn new() -> Self {
        Self ( 
            TaskID(TID_ALLOCATOR.alloc())
        )
    }

    #[inline(always)]
    /// generate Task Group ID
    pub fn id(&self) -> TaskID {
        self.0
    }
}

impl From<TaskHandle> for usize {
    fn from(value: TaskHandle) -> Self {
        value.0.0
    }
}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        TID_ALLOCATOR.dealloc(self.id().0);
    }
}

pub struct KernelStackALlocator;

impl KernelStackALlocator {
    pub fn alloc() -> KernelStackGuard{
        KernelStackGuard::new()
    }
}


pub struct KernelStackGuard {
    id: usize,
    top: usize,
    bottom: usize,
}



impl KernelStackGuard {
    fn new() -> Self {
        
        let kernel_stack_id =  KERNEL_STACK_ID_ALLOCATOR.alloc();

        let (bottom, top) = Self::get_position(kernel_stack_id);

        KERNEL_SPACE.lock().insert_framed_area(
            bottom.into(),
            top.into(), 
            MapPermission::W | MapPermission::R
        );

        Self{ id: kernel_stack_id, bottom, top }
    }

    // Return (bootom, top) of a kernel stack in kernel space.
    fn get_position(kernel_stack_id: usize) -> (usize, usize) {
        // |   Trampoline   | 
        // |      ...       |
        // |   Guard Page   | 
        // | Current KStack | -new allocate
        let top = KERNEL_STACK_BASE - kernel_stack_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
        let bottom = top - KERNEL_STACK_SIZE;
        (bottom, top)
    }


    fn get_id(&self) -> usize{
        self.id
    }

    pub fn get_top(&self) -> usize {
        self.top
    }
}

impl Drop for KernelStackGuard {
    fn drop(&mut self) {
        let start_va: VirtAddr = self.bottom.into();
        KERNEL_SPACE.lock().remove_area_with_start_vpn(start_va.into());
        KERNEL_STACK_ID_ALLOCATOR.dealloc(self.id);

    }
}

/// Allocator trap frame
pub struct TrapContextPageAllocator;

impl TrapContextPageAllocator {
    pub fn alloc(tid: TaskID, memory_set: Arc<IRQSpinLock<MemorySet>>) -> TrapContextPageGuard{
        TrapContextPageGuard::new(tid, memory_set)
    }
}

pub struct TrapContextPageGuard {
    tid: TaskID,
    vpn: VirtPageNum,
    ppn: PhysPageNum,
    memory_set: Arc<IRQSpinLock<MemorySet>>,
}

impl TrapContextPageGuard {

    #[inline(always)]
    fn trap_context_bottom(tid: TaskID) -> usize {
        TRAP_CONTEXT_START + (tid.0 * PAGE_SIZE)
    }

    pub fn get_mut_ref(&mut self) -> &'static mut TrapContext {
        self.get_trap_ppn().get_mut()
    }

    pub fn update(&mut self, other: TrapContext) {
        *self.get_mut_ref() = other
    }

    #[inline(always)]
    pub fn get_trap_ppn(&self) -> PhysPageNum {
        self.ppn
    }

    #[inline(always)]
    pub fn get_trap_vpn(&self) -> VirtPageNum {
        self.vpn
    }

    fn new(tid: TaskID, memory_set: Arc<IRQSpinLock<MemorySet>>) -> Self {
        let bottom = Self::trap_context_bottom(tid);
        let top = bottom + PAGE_SIZE;

        let mut memory_set_guard = memory_set.lock();

        memory_set_guard.insert_framed_area(
            bottom.into(), 
            top.into(), 
            MapPermission::R | MapPermission::W
        );


        let ppn = memory_set_guard
                .translate(bottom.into())
                .unwrap()
                .ppn();

        drop(memory_set_guard);

        Self {
            tid,
            vpn: VirtAddr::from(bottom).into(),
            ppn,
            memory_set
        }
    }
} 

impl Drop for TrapContextPageGuard {
    fn drop(&mut self) {
        self.memory_set.lock().remove_area_with_start_vpn(self.vpn);
    }
}


pub struct UserStackAlloctor;

impl UserStackAlloctor {
    pub fn alloc(memory_set: Arc<IRQSpinLock<MemorySet>>, base: usize, id: usize) -> UserStackGuard{
        UserStackGuard::new(memory_set, base, id)
    }
}

#[allow(unused)]
pub struct UserStackGuard {
    vpn: VirtPageNum,
    ppn: PhysPageNum,
    size: usize,
    user_stack_id: usize,
    memory_set: Arc<IRQSpinLock<MemorySet>>,
}

impl UserStackGuard {
    pub fn new(memory_set: Arc<IRQSpinLock<MemorySet>>, base: usize, id: usize) ->  Self{
        let top = Self::gen_top(base, id);

        let bottom = top - USER_STACK_SIZE;

        // log::debug!("stack base: {:x}, bottom({:x}) ~ top({:x})",  base, bottom, top);
        let bottom_va = VirtAddr::from(bottom);
        let top_va = VirtAddr::from(top);
        let bottom_vpn: VirtPageNum = bottom_va.into();
        let top_vpn: VirtPageNum = top_va.into();


        let mut memory_set_guard = memory_set.lock();
        memory_set_guard.insert_framed_area(
            bottom_va,
            top_va,
            MapPermission::U | MapPermission::W | MapPermission::R
        );


        // log::debug!("bottom_vpn: {:?}, bottom_va: {:?}", bottom_vpn, bottom_va);

        let ppn = memory_set_guard
                .translate(bottom_vpn)
                .unwrap()
                .ppn();

        drop(memory_set_guard);

        let va = VirtAddr::from(base);
        Self {
            vpn: bottom_vpn,
            ppn,
            size: PAGE_SIZE,
            user_stack_id: id,
            memory_set
        }
    }

    pub fn get_top(&self) -> usize {
        let base_va = VirtAddr::from(self.vpn);
        self.size + usize::from(base_va)
    }

    #[inline(always)]
    fn gen_top(base: usize, id: usize) -> usize {
        base + (id+1)* (PAGE_SIZE + USER_STACK_SIZE)
    }
}

impl Drop for UserStackGuard {
    fn drop(&mut self) {
        self.memory_set.lock().remove_area_with_start_vpn(self.vpn);
    }
}



pub struct RecycleAllocator {
    current: AtomicUsize,
    recycled: IRQSpinLock<Vec<usize>>,
}

impl RecycleAllocator {
    pub fn new() -> Self {
        RecycleAllocator {
            current: AtomicUsize::new(0),
            recycled: IRQSpinLock::new(Vec::new()),
        }
    }
    pub fn alloc(&self) -> usize {
        if let Some(id) = self.recycled.lock().pop() {
            return id;
        } 

        self.current.fetch_add(1, Ordering::AcqRel)
    }
    pub fn dealloc(&self, id: usize) {
        let mut recycled = self.recycled.lock();
        assert!(id < self.current.load(Ordering::Acquire));
        assert!(
            !recycled.iter().any(|i| *i == id),
            "id {} has been deallocated!",
            id
        );
        recycled.push(id);
    }
}