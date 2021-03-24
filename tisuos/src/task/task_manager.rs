//! # 任务管理
//! 管理任务系统调度，包含调度算法
//! 2021年3月23日 zg

use crate::interrupt::trap::Environment;

pub enum ScheduleMethod{
    Rotation,
}

pub enum TaskState {
    Running,
    Waiting,
    Sleeping,
}

/// ## 任务管理器
pub struct TaskManager<T1, T2> {
    pub scheduler : T1,
    pub task_pool : T2,
}

/// ## 调度器功能实现
/// 仅在机器模式下进行调用，以获得最高权限
/// 原则上只允许通过任务 ID 进行交互，不得引用、复制以节省处理时间防止同步错误
impl<T1 : SchedulerOp, T2 : TaskPoolOp> TaskManager<T1, T2> {
    pub const fn new(sche : T1, pool : T2)->Self {
        Self {
            scheduler : sche,
            task_pool : pool,
        }
    }
    
    pub fn schedule(&mut self, hartid : usize) {
        self.scheduler.schedule(&mut self.task_pool, hartid);
    }
    
    pub fn task_exit(&mut self, hartid : usize) {
        self.task_pool.delete_current(hartid);
    }
    /// ### 创建任务并返还任务 ID
    pub fn create_task(&mut self, func : usize, is_kernel : bool)->Option<usize>{
        self.task_pool.create(func, is_kernel)
    }
}


/// ## 线程池操作要求
/// 与线程池的操作根据线程号进行，不获取引用，避免同步问题，但提供复制功能
/// 因为这是个多核系统，所以与某个任务交互时必须提供足够的条件（id 或 hartid，state）
pub trait TaskPoolOp {
    fn create(&mut self, func : usize, is_kernel : bool)->Option<usize>;
    fn save_current(&mut self, env : &Environment, hartid : usize);
    fn get_task_with_state(&self, hartid : usize, state : TaskState)->usize;
    fn set_state(&mut self, id : usize, state : TaskState);
    fn switch_to(&self, id : usize);
    fn delete_current(&mut self, hartid : usize);
}

/// ## 调度器操作要求
/// 算法实现由调度器自身决定
pub trait SchedulerOp{
    fn schedule<T:TaskPoolOp>(&mut self, task_pool : &mut T, hartid : usize);
    fn switch_method(&mut self, method : ScheduleMethod);
}

