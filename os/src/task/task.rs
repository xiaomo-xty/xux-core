use core::{fmt, usize};

use alloc::{format, string::String, sync::{Arc, Weak}, vec::Vec};
use bitflags::bitflags;


use crate::{mm::{address::{PhysPageNum, VirtPageNum}, memory_set::MemorySet, KERNEL_SPACE}, sync::spin::mutex::{SpinMutex, SpinMutexGuard}, trap::{trap_handler, TrapContext}};

use super::{allocator::{KernelStackALlocator, KernelStackGuard, RecycleAllocator, TaskHandle, TaskHandleAllocator, TaskID, TrapContextPageAllocator, TrapContextPageGuard, UserStackAlloctor, UserStackGuard}, TaskContext};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskState {
    // UnInitialized,
    Ready,
    Running,
    Exited,
    Blocking,
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

impl fmt::Debug for TaskControlBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner.lock();
        f.debug_struct("TaskControlBlock")
                    .field("task_handle", &inner.task_handle)
                    .field("name", &inner.name)
                    .field("state", &inner.state)
                    .field("has_user_res", &inner.user_res.is_some())
                    .finish_non_exhaustive() // 避免打印所有字段
    }
}

impl fmt::Debug for TaskUserResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let children_len = self.children.lock().len();
        let task_group_len = self.task_group.lock().len();
        
        f.debug_struct("TaskUserResource")
            .field("task_group_id", &self.task_group_id)
            .field("children_count", &children_len)
            .field("task_group_count", &task_group_len)
            .field("entry_point", &self.entry_point)
            .finish_non_exhaustive()
    }
}



/// Task's Control information used by kernel
// LWP Light ...
pub struct TaskControlBlockInner {
    pub task_handle: TaskHandle,                      // 进程ID
    pub name: String,                   // name
    pub state: TaskState,              // 运行状态（就绪/阻塞等）
    pub kernel_stack_guard: KernelStackGuard,
    pub context: TaskContext,          // 寄存器等硬件上下文
    
    user_res: Option<TaskUserResource>,
      
}


pub struct TaskControlBlock { 
    inner: SpinMutex<TaskControlBlockInner>,
}

/// UserResource
/// It's not necessary for a Task
pub struct TaskUserResource {
    pub parent_group_id: Option<TaskID>,
    pub task_group_id: TaskID, 
    pub group_leader: Weak<TaskControlBlock>,
    // pub fs: Arc<FileSystem>,           // 文件系统上下文
    // pub files: Arc<Mutex<FileTable>>,  // 文件描述符表
    // pub signal: Arc<SignalHandler>,    // 信号处理
    pub memory_set: Arc<SpinMutex<MemorySet>>, // 内存管理（用户空间）

    pub parent: Option<Weak<TaskControlBlock>>, // parent leader


    pub children: Arc<SpinMutex<Vec<Arc<TaskControlBlock>>>>,   // the leader of child task group
    pub task_group: Arc<SpinMutex<Vec<Arc<TaskControlBlock>>>>, // task_group

    user_stack_id_allocator: Arc<SpinMutex<RecycleAllocator>>,
    pub user_stack_guard: UserStackGuard,
    pub entry_point: usize,

    pub trap_context_guard: TrapContextPageGuard,
}



impl TaskControlBlock {
    pub fn lock(&self) -> SpinMutexGuard<TaskControlBlockInner> {
        self.inner.lock()
    }

    pub fn new_from_elf(elf_data: &[u8], app_id: usize) -> Arc<Self> {
        let name = format!("app[{}]", app_id);
        let inner = TaskControlBlockInner::new(name);

        let kerbel_stack_top = inner.kernel_stack_guard.get_top();

        let task_control_block = Arc::new(
            TaskControlBlock 
            { 
                inner: SpinMutex::new(inner) 
            }
        );


        task_control_block.inner.lock().allocate_user_resource(
            task_control_block.clone(), 
            elf_data,
            kerbel_stack_top
        );
        task_control_block
    }

    pub fn set_state(&self, target_state: TaskState){
        self.lock().state = target_state;
    }

    pub fn get_state(&self) -> TaskState {
        self.lock().state
    }

    // Disjoint set
    pub fn is_leader(&self) -> bool {
        let inner = self.inner.lock();
        if let Some(user_res) = &inner.user_res {
            user_res.task_group_id == inner.task_handle.id()
        }
        else {
            true
        }
    }

    pub fn with_user_res<R>(&self, f: impl FnOnce(Option<&mut TaskUserResource>) -> R) -> R {
        let mut inner = self.inner.lock();
        f(inner.user_res.as_mut())
    }

    
}



impl TaskControlBlockInner {

    pub fn new(name: String) -> Self {
        let kernel_stack_guard = KernelStackALlocator::alloc();
        let kernel_stack_top = kernel_stack_guard.get_top();


        Self {
            task_handle: TaskHandleAllocator::allocate(),
            name,
            state: TaskState::Ready,
            kernel_stack_guard,
            context: TaskContext::goto_trap_return(kernel_stack_top),
            user_res: None,
        }
    }

    fn allocate_user_resource(&mut self, 
            task_control_block: Arc<TaskControlBlock>, 
            elf_data: &[u8],
            kernel_stack_top: usize
        ) {

        let (memory_set, user_stack_base, entry_point) = 
        MemorySet::from_elf(elf_data);


        self.user_res = Some(TaskUserResource::new(
            self.task_handle.id(),
            self.task_handle.id(),
            Arc::downgrade(&task_control_block), // 创建弱引用
            user_stack_base,
            kernel_stack_top,
            Arc::new(SpinMutex::new(memory_set)),
            entry_point
        ));
    }
        
}


impl TaskUserResource {

    pub fn new(
        tid: TaskID, 
        leader_id: TaskID,
        leader: Weak<TaskControlBlock>, 
        user_stack_base: usize,
        kernel_stack_top: usize,
        memory_set: Arc<SpinMutex<MemorySet>>,
        entry_point: usize,
    ) -> Self {

        log::debug!("new TaskUserResource");

        let user_stack_id_allocator = Arc::new(SpinMutex::new(
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



        Self { 
            task_group_id: leader_id, 
            parent_group_id: None,
            group_leader: leader,
            memory_set, 
            parent: None, 
            children: Arc::new(SpinMutex::new(Vec::new())), 
            task_group: Arc::new(SpinMutex::new(Vec::new())), 
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


}


