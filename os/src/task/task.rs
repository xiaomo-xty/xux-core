use core::{cell::UnsafeCell, fmt::{self, Display}, ptr, sync::atomic::{AtomicBool, AtomicPtr, Ordering}, usize};

use alloc::{boxed::Box, format, string::String, sync::{Arc, Weak}, vec::Vec};
use bitflags::bitflags;


use crate::{mm::{address::{PhysPageNum, VirtPageNum}, memory_set::MemorySet, KERNEL_SPACE}, println, processor::get_current_processor, sync::spin::mutex::{IRQSpinLock,IRQSpinLockGuard}, trap::{trap_handler, TrapContext}};

use super::{allocator::{KernelStackALlocator, KernelStackGuard, RecycleAllocator, TaskHandle, TaskHandleAllocator, TaskID, TrapContextPageAllocator, TrapContextPageGuard, UserStackAlloctor, UserStackGuard}, signal::Signal, yield_current, TaskContext};

type Mutex<T> = IRQSpinLock<T>;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskState {
    // UnInitialized,
    Ready,
    Running,
    Blocking,
    Zombie(i32),
    Dead,
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskState::Ready => f.pad("Ready"),
            TaskState::Running => f.pad("Running"),
            TaskState::Blocking => f.pad("Blocking"),
            TaskState::Zombie(exit_code) => write!(f, "Zombie(exit_code: {})", exit_code),
            TaskState::Dead => f.pad("Dead"),
        }
    }
}


bitflags! {
    pub struct CloneFlags: u32 {
        const CLONE_VM        = 0x00000100; // 共享地址空间
        // const CLONE_FS        = 0x00000200; // 共享文件系统信息
        // const CLONE_FILES     = 0x00000400; // 共享文件描述符
        // const CLONE_SIGHAND   = 0x00000800; // 共享信号处理器
        const CLONE_THREAD    = 0x00010000; // 同线程组
        const CLONE_PARENT    = 0x00008000; // 共享父进程
        const CLONE_CHILD_CLEARTID = 0x00200000; // 清除子线程TID
    }
}


pub struct PendingTaskLockGuard {
    slot: UnsafeCell<Option<IRQSpinLockGuard<'static, TaskControlBlockInner>>>,
    occupied: AtomicBool,
}


impl PendingTaskLockGuard {
    pub const fn new() -> Self {
        Self { slot: UnsafeCell::new(None), occupied: AtomicBool::new(false) }
    }

    pub unsafe fn store_lock(&self, guard: IRQSpinLockGuard<'_, TaskControlBlockInner>) {
        if self.occupied.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            panic!("PendingTaskLockGuard already occupied");
        }

        // 将guard的生命周期延长到'static（需确保安全）
        unsafe { 
            self.slot.get().write(
                Some( core::mem::transmute(guard) )
            );
        }
    }
    
    /// 取出锁守卫
    /// 安全要求：必须确保之前已经调用了store_lock
    pub unsafe fn take_lock(&self) -> IRQSpinLockGuard<'_, TaskControlBlockInner> {
        if !self.occupied.swap(false, Ordering::Release) {
            panic!("No lock stored in PendingTaskLockGuard");
        }

        (*self.slot.get()).take().expect("PendingTaskLockGuard was empty")
    }
}

unsafe impl Sync for PendingTaskLockGuard {}




pub struct TaskControlBlock { 
    task_handle: TaskHandle,        // 进程ID
    name: String,                   // name
    is_leader: bool,
    kernel_stack_guard: KernelStackGuard,
    
    inner: Mutex<TaskControlBlockInner>,
    lock_guard: PendingTaskLockGuard,
}

/// Task's Control information used by kernel
// LWP Light ...
pub struct TaskControlBlockInner {
    pub state: TaskState,              // 运行状态（就绪/阻塞等）
    pub context: TaskContext,          // 寄存器等硬件上下文
    
    user_res: Option<TaskUserResource>,
      
}



/// UserResource
/// It's not necessary for a Task
pub struct TaskUserResource {
    pub parent_group_id: Option<TaskID>,
    pub parent: Option<Weak<TaskControlBlock>>, // parent leader


    pub group_leader: Weak<TaskControlBlock>,
    // pub fs: Arc<FileSystem>,           // 文件系统上下文
    // pub files: Arc<Mutex<FileTable>>,  // 文件描述符表
    // pub signal: Arc<SignalHandler>,    // 信号处理
    pub memory_set: Arc<Mutex<MemorySet>>, // 内存管理（用户空间）



    pub children: Arc<Mutex<Vec<Arc<TaskControlBlock>>>>,   // the leader of child task group
    pub task_group: Arc<Mutex<Vec<Arc<TaskControlBlock>>>>, // task_group

    user_stack_id_allocator: Arc<Mutex<RecycleAllocator>>,
    pub user_stack_guard: UserStackGuard,
    pub entry_point: usize,
    pub trap_context_guard: TrapContextPageGuard,
}


impl fmt::Debug for TaskControlBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 原子指针的可读显示

        
        f.debug_struct("TaskControlBlock")
            .field("task_handle", &self.task_handle)
            .field("name", &self.name)
            .field("is_leader", &self.is_leader)
            .field("kernel_stack_top", &self.kernel_stack_guard.get_top())
            .finish()
    }
}

impl fmt::Debug for TaskUserResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 避免直接打印需要锁的字段，而是打印它们的摘要信息
        
        f.debug_struct("TaskUserResource")
            .field("parent_group_id", &self.parent_group_id)
            .field("\ntask_group_id", &self.group_leader.upgrade().unwrap().task_handle)
            .field("\nuser_stack top", &self.user_stack_guard.get_top()) // 假设 UserStackGuard 实现了 Debug
            .field("\nentry_point", &format_args!("{:#x}", self.entry_point))
            .field("\ntrap_context_page vpn:", &self.trap_context_guard.get_trap_vpn()) // 假设 TrapContextPageGuard 实现了 Debug
            .finish()
    }
}



impl TaskControlBlock {
    pub fn lock(&self) -> IRQSpinLockGuard<TaskControlBlockInner> {
        self.inner.lock()
    }

    #[inline]
    pub fn get_name(&self) -> &String {
        &self.name
    }

    #[inline]
    pub fn get_tid(&self) -> TaskID {
        self.task_handle.id()
    }

    #[inline]
    pub fn is_leader(&self) -> bool {
        return self.is_leader;
    }

    pub fn store_lock(&self, guard: IRQSpinLockGuard<'_, TaskControlBlockInner>) {
        unsafe { self.lock_guard.store_lock(guard); }
    }
    
    pub fn take_lock(&self) -> IRQSpinLockGuard<'_, TaskControlBlockInner> {
        unsafe { self.lock_guard.take_lock() }
    }

    pub fn new_from_elf(elf_data: &[u8], app_name: String, parent_task: Option<Arc<TaskControlBlock>>) -> Arc<Self> {
        let task_handle = TaskHandleAllocator::allocate();
        let task_id = task_handle.id();

        let kernel_stack_guard = KernelStackALlocator::alloc();
        let kernel_stack_top = kernel_stack_guard.get_top();

        let inner = TaskControlBlockInner::new(kernel_stack_top);


        let task_control_block = Arc::new(
            TaskControlBlock 
            { 
                task_handle,
                name: app_name,
                kernel_stack_guard,
                is_leader: true,
                inner: Mutex::new(inner),
                lock_guard: PendingTaskLockGuard::new(),
            }
        );

        let group_leader = Arc::downgrade(&task_control_block);


        task_control_block.inner.lock().user_res = Some(
            TaskUserResource::new(
                task_id, 
                elf_data,
                group_leader,
                parent_task,
                kernel_stack_top, 
            )
        );

        task_control_block.lock().with_user_res(|user_res| {
            user_res.add_group_member(task_control_block.clone());
        });


        task_control_block
    }

    
    pub fn prepare_exit(&self) {
        if self.is_leader() {
            self.lock().wait_group_eixt();
        }
        // mound_child_to_init

        // release whole task group resource
        drop(self.lock().user_res.take().unwrap());

    }

}



impl TaskControlBlockInner {

    pub fn new(kernel_stack_top: usize) -> Self {
        Self {
            state: TaskState::Ready,
            context: TaskContext::goto_new_user_task_start(kernel_stack_top),
            user_res: None,
        }
    }

    pub fn set_state(&mut self, target_state: TaskState){
        self.state = target_state;
    }

    pub fn get_state(&self) -> TaskState {
        self.state
    }

    fn wait_group_eixt(&mut self) {
        // notity all group member
        if let Some(user_res) = self.user_res.as_ref() {
            for task in user_res.task_group.lock().iter() {
                if ! task.is_leader() {
                    let mut task = task.inner.lock();
                    task.signal(Signal::SIGTERM);
                }
            };

            // maybe add timeout
            loop {
                if user_res.task_group.lock().len() == 1 {
                    break;
                }
                
                //sleep
            }

            user_res.task_group.lock().pop();

            assert!(user_res.task_group.lock().is_empty())
        }
    }

    /// Provides controlled access to the task's user resource within a locked context
    ///
    /// # Contract
    /// - ​**Caller must hold outer lock(s)** protecting the task structure
    /// - ​**Resource must be initialized** before invocation
    ///
    /// # Safety
    /// 1. Not thread-safe on its own - external synchronization required
    /// 2. Mutable access (`&mut TaskUserResource`) enables arbitrary modifications
    /// 3. No reentrancy protection - avoid recursive calls
    ///
    /// # Panics
    /// Will panic if either:
    /// - User resource not initialized (`user_res.is_none()`)
    /// - Called without proper outer locking (via UB from data races)
    ///
    /// # Examples
    /// ```rust
    /// // Proper usage sequence:
    /// let mut guard = task_inner.lock();  // Acquire outer lock
    /// let result = guard.with_user_res(|res| {
    ///     res.allocate_buffer(4096)?;
    ///     res.commit_operations()
    /// });
    /// // Lock released when `guard` drops
    /// ```
    ///
    pub fn with_user_res<R>(&mut self, f: impl FnOnce(&mut TaskUserResource) -> R) -> R {
        log::debug!("operator with user res");
        f(self.user_res.as_mut().unwrap())
    }

    fn mount_child_to_init() {
        unimplemented!()
    }
    

    pub fn notify_parent(&self, exit_code: i32) {
        println!("notify parent (faker)");
    }

    pub fn signal(&mut self, signal: Signal) {
        println!("(faker) signal {}", signal.description());
        // self.handler_signal(signal);
        
    }

}


impl TaskUserResource {

    pub fn new(
        tid: TaskID, 
        elf_data: &[u8],
        group_leader: Weak<TaskControlBlock>,
        parent: Option<Arc<TaskControlBlock>>,
        kernel_stack_top: usize,
    ) -> Self {

        log::debug!("new TaskUserResource");

        let (memory_set, user_stack_base, entry_point) = 
        MemorySet::from_elf(elf_data);

        let memory_set = Arc::new(Mutex::new(memory_set));

        let user_stack_id_allocator = Arc::new(Mutex::new(
            RecycleAllocator::new()
        ));

        let user_stack_id = user_stack_id_allocator.lock().alloc();

        let user_stack_guard =  UserStackAlloctor::alloc(
            memory_set.clone(), 
            user_stack_base, 
            user_stack_id
        );

        let mut trap_context_guard = TrapContextPageAllocator::alloc(tid, memory_set.clone());

        let trap_context = TrapContext::app_init_context(
            entry_point, 
            user_stack_guard.get_top(), 
            KERNEL_SPACE.lock().token(), 
            kernel_stack_top, 
            trap_handler as usize
        );
        trap_context_guard.update(trap_context);


        let task_group = Arc::new(Mutex::new(Vec::new()));

        let (parent_group_id, parent) = match parent {
            Some(parent) => {
                ( Some(parent.task_handle.id()),
                Some(Arc::downgrade(&parent)))
            },
            None => {
                (None, None)
            },
        };

        Self { 
            parent_group_id,
            group_leader,
            memory_set, 
            parent, 
            children: Arc::new(Mutex::new(Vec::new())), 
            task_group, 
            user_stack_guard,
            entry_point,
            user_stack_id_allocator,
            trap_context_guard,
        }
    }



    #[inline(always)]
    pub fn trap_context_ppn(&self) -> PhysPageNum {
        self.trap_context_guard.get_trap_ppn()
    }

    #[inline(always)]
    pub fn trap_context_vpn(&self) -> VirtPageNum {
        self.trap_context_guard.get_trap_vpn()
    }


    pub fn add_group_member(&mut self, new_member: Arc<TaskControlBlock>) {
        self.task_group.lock().push(new_member);
    }

    pub fn add_child(&mut self, new_child: Arc<TaskControlBlock>) {
        self.children.lock().push(new_child);
    }

}


impl Drop for TaskControlBlock {
    fn drop(&mut self) {
        log::debug!("drop task {}", self.get_name());
    }
}


