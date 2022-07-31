use core::slice;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::rc::Rc;
use riscv::register::sepc;
use riscv::register::scause;
use riscv::register::scause::Trap;
use riscv::register::scause::Exception;
use riscv::register::scause::Interrupt;
use riscv::register::stval;
use riscv::register::sstatus;

use crate::memory::page_table::PageMappingManager;
use crate::memory::addr::VirtAddr;
use crate::memory::addr::PhysAddr;
use crate::interrupt::{Context, timer};
use crate::fs::filetree::INode;
use crate::task::task_scheduler::kill_task;
use crate::sys_call::consts::EBADF;

use crate::fs::file::FileType;
use crate::interrupt::timer::set_last_ticks;
use crate::runtime_err::RuntimeError;
use crate::task::signal::SignalUserContext;
use crate::task::task::Task;

pub mod fd;
pub mod task;
pub mod time;
pub mod mm;
pub mod consts;
pub mod signal;

// 中断调用列表
pub const SYS_GETCWD:usize  = 17;
pub const SYS_DUP: usize    = 23;
pub const SYS_DUP3: usize   = 24;
pub const SYS_MKDIRAT:usize = 34;
pub const SYS_UNLINKAT:usize= 35;
pub const SYS_UMOUNT2: usize= 39;
pub const SYS_MOUNT: usize  = 40;
pub const SYS_STATFS: usize = 43;
pub const SYS_CHDIR: usize  = 49;
pub const SYS_OPENAT:usize  = 56;
pub const SYS_CLOSE: usize  = 57;
pub const SYS_PIPE2: usize  = 59;
pub const SYS_GETDENTS:usize= 61;
pub const SYS_LSEEK: usize  = 62;
pub const SYS_READ:  usize  = 63;
pub const SYS_WRITE: usize  = 64;
pub const SYS_READV:  usize  = 65;
pub const SYS_WRITEV: usize = 66;
pub const SYS_PREAD: usize  = 67;
pub const SYS_FSTATAT: usize= 79;
pub const SYS_FSTAT: usize  = 80;
pub const SYS_UTIMEAT:usize = 88;
pub const SYS_EXIT:  usize  = 93;
pub const SYS_EXIT_GROUP: usize = 94;
pub const SYS_SET_TID_ADDRESS: usize = 96;
pub const SYS_FUTEX: usize  = 98;
pub const SYS_NANOSLEEP: usize = 101;
pub const SYS_GETTIME: usize = 113;
pub const SYS_SCHED_YIELD: usize = 124;
pub const SYS_KILL: usize = 129;
pub const SYS_TKILL: usize = 130;
pub const SYS_SIGACTION: usize = 134;
pub const SYS_SIGPROCMASK: usize = 135;
pub const SYS_SIGTIMEDWAIT: usize = 137;
pub const SYS_SIGRETURN: usize = 139;
pub const SYS_TIMES: usize  = 153;
pub const SYS_UNAME: usize  = 160;
pub const SYS_GETTIMEOFDAY: usize= 169;
pub const SYS_GETPID:usize  = 172;
pub const SYS_GETPPID:usize = 173;
pub const SYS_GETTID: usize = 178;
pub const SYS_BRK:   usize  = 214;
pub const SYS_CLONE: usize  = 220;
pub const SYS_EXECVE:usize  = 221;
pub const SYS_MMAP: usize   = 222;
pub const SYS_MPROTECT:usize= 226;
pub const SYS_MUNMAP:usize  = 215;
pub const SYS_WAIT4: usize  = 260;

// 系统调用错误码
pub const SYS_CALL_ERR: usize = -1 as isize as usize;


// Open标志
bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 6;
        const TRUNC = 1 << 10;
        const O_DIRECTORY = 1 << 21;
    }

    pub struct SignalFlag: usize {
        const SA_NOCLDSTOP = 0x1;
        const SA_NOCLDWAIT = 0x2;
        const SA_SIGINFO   = 0x4;
        const SA_RESTART   = 0x10000000;
        const SA_NODEFER   = 0x40000000;
        const SA_RESETHAND = 0x80000000;
        const SA_RESTORER  = 0x04000000;
    }

    pub struct CloneFlags: usize {
        const CSIGNAL		= 0x000000ff;
        const CLONE_VM	= 0x00000100;
        const CLONE_FS	= 0x00000200;
        const CLONE_FILES	= 0x00000400;
        const CLONE_SIGHAND	= 0x00000800;
        const CLONE_PIDFD	= 0x00001000;
        const CLONE_PTRACE	= 0x00002000;
        const CLONE_VFORK	= 0x00004000;
        const CLONE_PARENT	= 0x00008000;
        const CLONE_THREAD	= 0x00010000;
        const CLONE_NEWNS	= 0x00020000;
        const CLONE_SYSVSEM	= 0x00040000;
        const CLONE_SETTLS	= 0x00080000;
        const CLONE_PARENT_SETTID	= 0x00100000;
        const CLONE_CHILD_CLEARTID	= 0x00200000;
        const CLONE_DETACHED	= 0x00400000;
        const CLONE_UNTRACED	= 0x00800000;
        const CLONE_CHILD_SETTID	= 0x01000000;
        const CLONE_NEWCGROUP	= 0x02000000;
        const CLONE_NEWUTS	= 0x04000000;
        const CLONE_NEWIPC	= 0x08000000;
        const CLONE_NEWUSER	= 0x10000000;
        const CLONE_NEWPID	= 0x20000000;
        const CLONE_NEWNET	= 0x40000000;
        const CLONE_IO	= 0x80000000;
    }
}

// 系统信息结构
pub struct UTSname  {
    sysname: [u8;65],
    nodename: [u8;65],
    release: [u8;65],
    version: [u8;65],
    machine: [u8;65],
    domainname: [u8;65],
}

// 文件Dirent结构
#[repr(C)]
struct Dirent {
    d_ino: u64,	        // 索引结点号
    d_off: u64,	        // 到下一个dirent的偏移
    d_reclen: u16,	    // 当前dirent的长度
    d_type: u8,	        // 文件类型
    d_name_start: u8	//文件名
}

// sys_write调用
pub fn sys_write_wrap(pmm: Rc<PageMappingManager>, fd: Rc<INode>, buf: usize, count: usize) -> usize {
    // 根据satp中的地址构建PageMapping 获取当前的映射方式
    let buf = pmm.get_phys_addr(VirtAddr::from(buf)).unwrap();

    // 寻找物理地址
    let buf = unsafe {slice::from_raw_parts_mut(usize::from(buf) as *mut u8, count)};
    
    // 匹配文件类型
    match fd.get_file_type() {
        FileType::VirtFile => {
            fd.write(buf);
        }
        _ => {warn!("SYS_WRITE暂未找到设备");}
    }
    count
}

// 从内存中获取字符串 目前仅支持ascii码
pub fn get_string_from_raw(addr: PhysAddr) -> String {

    let mut ptr = addr.as_ptr();
    let mut str: String = String::new();
    loop {
        let ch = unsafe { ptr.read() };
        if ch == 0 {
            break;
        }
        str.push(ch as char);
        unsafe { ptr = ptr.add(1) };
    }
    str
}

// 从内存中获取数字直到0
pub fn get_usize_vec_from_raw(addr: PhysAddr) -> Vec<usize> {
    let mut usize_vec = vec![];
    let mut usize_vec_ptr = addr.0 as *const usize;
    loop {
        let value = unsafe { usize_vec_ptr.read() };
        if value == 0 {break;}
        usize_vec.push(value);
        usize_vec_ptr = unsafe { usize_vec_ptr.add(1) };
    }
    usize_vec
}

// 将字符串写入内存 目前仅支持ascii码
pub fn write_string_to_raw(target: &mut [u8], str: &str) {
    let mut index = 0;
    for c in str.chars() {
        target[index] = c as u8;
        index = index + 1;
    }
    target[index] = 0;
}

impl Task {
    // 系统调用
    pub fn sys_call(&self, call_type: usize, args: [usize; 7]) -> Result<(), RuntimeError> {
        // 匹配系统调用 a7(x17) 作为调用号
        match call_type {
            // 获取文件路径
            SYS_GETCWD => self.get_cwd(args[0], args[1]),
            // 复制文件描述符
            SYS_DUP => self.sys_dup(args[0]),
            // 复制文件描述符
            SYS_DUP3 => self.sys_dup3(args[0], args[1]),
            // 创建文件夹
            SYS_MKDIRAT => self.sys_mkdirat(args[0], args[1], args[2]),
            // 取消link
            SYS_UNLINKAT => self.sys_unlinkat(args[0], args[1], args[2]),
            // umount设备
            SYS_UMOUNT2 => Ok(()),
            // mount设备
            SYS_MOUNT => Ok(()),
            // 获取文件系统信息
            SYS_STATFS => self.sys_statfs(args[0], args[1].into()),
            // 改变文件信息
            SYS_CHDIR => self.sys_chdir(args[0]),
            // 打开文件地址
            SYS_OPENAT => self.sys_openat(args[0], args[1], args[2], args[3]),
            // 关闭文件描述符
            SYS_CLOSE => self.sys_close(args[0]),
            // 进行PIPE
            SYS_PIPE2 => self.sys_pipe2(args[0]),
            // 获取文件节点
            SYS_GETDENTS => self.sys_getdents(args[0], args[1], args[2]),
            // 移动读取位置
            SYS_LSEEK => self.sys_lseek(args[0], args[1], args[2]),
            // 读取文件描述符
            SYS_READ => self.sys_read(args[0], args[1], args[2]),
            // 写入文件数据
            SYS_WRITE => self.sys_write(args[0], args[1], args[2]),
            // 读取数据
            SYS_READV => self.sys_readv(args[0], args[1].into(), args[2]),
            // 写入数据
            SYS_WRITEV => self.sys_writev(args[0], args[1].into(), args[2]),
            // 读取数据
            SYS_PREAD => self.sys_pread(args[0], args[1].into(), args[2], args[3]),
            // 获取文件数据信息
            SYS_FSTATAT => self.sys_fstatat(args[0], args[1].into(), args[2], args[3]),
            // 获取文件数据信息
            SYS_FSTAT => self.sys_fstat(args[0], args[1]),
            // 改变文件时间
            SYS_UTIMEAT => self.sys_utimeat(args[0], args[1].into(), args[2].into(), args[3]),
            // 退出文件信息
            SYS_EXIT => self.sys_exit(args[0]),
            // 退出组
            SYS_EXIT_GROUP => self.sys_exit_group(args[0]),
            // 设置tid
            SYS_SET_TID_ADDRESS => self.sys_set_tid_address(args[0].into()),
            // 互斥锁
            SYS_FUTEX => self.sys_futex(args[0].into(), args[1] as u32, args[2] as _, args[3], args[4]),
            // 文件休眠
            SYS_NANOSLEEP => self.sys_nanosleep(args[0].into(), args[1].into()),
            // 获取系统时间
            SYS_GETTIME => self.sys_gettime(args[0], args[1].into()),
            // 转移文件权限
            SYS_SCHED_YIELD => self.sys_sched_yield(),
            // 结束进程
            SYS_KILL => self.sys_kill(args[0], args[1]),
            // 结束任务进程
            SYS_TKILL => self.sys_tkill(args[0], args[1]),
            // 释放sigacrtion
            SYS_SIGACTION => self.sys_sigaction(args[0], args[1].into(),args[2].into(), args[3]),
            // 遮盖信号
            SYS_SIGPROCMASK => self.sys_sigprocmask(args[0] as _, args[1].into(),args[2].into(), args[3] as _),
            //
            // SYS_SIGTIMEDWAIT => {
            //     let mut inner = self.inner.borrow_mut();
            //     inner.context.x[10] = 0;
            //     Ok(())
            // }
            // 信号返回程序
            SYS_SIGRETURN => self.sys_sigreturn(),
            // 获取文件时间
            SYS_TIMES => self.sys_times(args[0]),
            // 获取系统信息
            SYS_UNAME => self.sys_uname(args[0]),
            // 获取时间信息
            SYS_GETTIMEOFDAY => self.sys_gettimeofday(args[0]),
            // 获取进程信息
            SYS_GETPID => self.sys_getpid(),
            // 获取进程父进程
            SYS_GETPPID => self.sys_getppid(),
            // 获取tid
            SYS_GETTID => self.sys_gettid(),
            // 申请堆空间
            SYS_BRK => self.sys_brk(args[0]),
            // 复制进程信息
            SYS_CLONE => self.sys_clone(args[0], args[1], args[2].into(), args[3], args[4].into()),
            // 执行文件
            SYS_EXECVE => self.sys_execve(args[0].into(), args[1].into(), args[2].into()),
            // 进行文件映射
            SYS_MMAP => self.sys_mmap(args[0], args[1], args[2], args[3], args[4], args[5]),
            // 页面保护
            SYS_MPROTECT => self.sys_mprotect(args[0], args[1], args[2]),
            // 取消文件映射
            SYS_MUNMAP => self.sys_munmap(args[0], args[1]),
            // 等待进程
            SYS_WAIT4 => self.sys_wait4(args[0], args[1].into(), args[2]),
            _ => {
                warn!("未识别调用号 {}", call_type);
                Ok(())
            }
        }
    }

    pub fn catch(&self) {
        let result = self.interrupt();
        debug!("catch");
        if let Err(err) = result {
            match err {
                RuntimeError::KillSelfTask => {
                    kill_task(self.pid, self.tid);
                }
                RuntimeError::NoEnoughPage => {
                    panic!("No Enough Page");
                }
                RuntimeError::NoMatchedFileDesc => {
                    let mut inner = self.inner.borrow_mut();
                    warn!("文件未找到");
                    inner.context.x[10] = SYS_CALL_ERR;
                }
                RuntimeError::FileNotFound => {
                    let mut inner = self.inner.borrow_mut();
                    warn!("文件未找到");
                    inner.context.x[10] = SYS_CALL_ERR;
                }
                RuntimeError::EBADF => {
                    let mut inner = self.inner.borrow_mut();
                    warn!("文件未找到  EBADF");
                    inner.context.x[10] = EBADF;
                }
                _ => {
                    warn!("异常: {:?}", err);
                }
            }
        }
    }

    pub fn signal(&self, signal: usize) -> Result<(), RuntimeError> {
        let mut inner = self.inner.borrow_mut();
        let mut process = inner.process.borrow_mut();
        let handler = process.signal.handler;
        debug!("signal handler: {:#x}  pid: {}  tid: {}", handler, self.pid, self.tid);
        // 保存上下文
        let mut temp_context = inner.context.clone();
        let pmm = process.pmm.clone();
        let ucontext = process.heap.get_temp(pmm).tranfer::<SignalUserContext>();
        // 中断正在处理中
        if ucontext.context.x[0] != 0 {
            return Ok(());
        }
        let restorer = process.signal.restorer;
        let flags = SignalFlag::from_bits_truncate(process.signal.flags);
        debug!("signal flags: {:?}", flags);
        
        drop(process);
        inner.context.sepc = handler;
        inner.context.x[1] = restorer;
        inner.context.x[10] = signal;
        inner.context.x[11] = 0;
        inner.context.x[12] = 0xe0000000;
        ucontext.context.clone_from(&temp_context);
        ucontext.context.x[0] = ucontext.context.sepc;
        debug!("回调地址: {:#x}", ucontext.context.sepc);
        drop(inner);

        loop {
            self.run();
            if let Err(RuntimeError::SigReturn) = self.interrupt() {
                break;
                debug!("切换任务");
            }
        }
        debug!("信号处理完毕");
        // 修改回调地址
        debug!("恢复回调地址: {:#x}", ucontext.context.x[0]);
        temp_context.sepc = ucontext.context.x[0];
        // panic!("signal exit");
        // loop {
        //     self.run();
        //     let result = self.interrupt();
        //     if let Err(RuntimeError::ChangeTask ) = result {
        //         break;
        //     }
        // }

        // 恢复上下文 并 移除临时页
        let mut inner = self.inner.borrow_mut();
        let process = inner.process.borrow_mut();
        process.heap.release_temp();
        drop(process);
        inner.context.clone_from(&temp_context);
        Ok(())
    }

    pub fn interrupt(&self) -> Result<(), RuntimeError> {
        unsafe {
            sstatus::set_fs(sstatus::FS::Dirty);
        }
        let scause = scause::read();
        let stval = stval::read();
        let mut task_inner = self.inner.borrow_mut();
        let context = &mut task_inner.context;
        warn!("中断发生: {:#x}, 地址: {:#x}", scause.bits(), context.sepc);
        // 更新TICKS
        set_last_ticks();

        // 匹配中断原因
        match scause.cause(){
            // 断点中断
            Trap::Exception(Exception::Breakpoint) => {
                warn!("break中断产生 中断地址 {:#x}", sepc::read());
                context.sepc = context.sepc + 2;
            },
            // 时钟中断
            Trap::Interrupt(Interrupt::SupervisorTimer) => timer::timer_handler(),
            // 页处理错误
            Trap::Exception(Exception::StorePageFault) | Trap::Exception(Exception::StoreFault) => {
                error!("缺页中断触发 缺页地址: {:#x} 触发地址:{:#x} 已同步映射", stval, context.sepc);
                drop(context);
                if stval > 0xf0000000 && stval < 0xf00010000 {
                    error!("处理缺页中断;");
                    let mut process = task_inner.process.borrow_mut();
                    process.stack.alloc_until(stval)?;
                } else {
                    panic!("无法 恢复的缺页中断");
                }
                // panic!("系统终止");
            },
            // 用户请求
            Trap::Exception(Exception::UserEnvCall) => {
                // 将 恢复地址 + 4 跳过调用地址
                debug!("中断号: {} 调用地址: {:#x}", context.x[17], sepc::read());

                // 对sepc + 4
                context.sepc += 4;
                // 复制参数
                let mut args = [0;7];
                args.clone_from_slice(&context.x[10..17]);
                let call_type = context.x[17];
                drop(context);
                drop(task_inner);
                

                self.sys_call(call_type, args)?;
            },
            // 加载页面错误
            Trap::Exception(Exception::LoadPageFault) => {
                panic!("加载权限异常 地址:{:#x} 调用地址: {:#x}", stval, context.sepc)
            },
            // 页面未对齐错误
            Trap::Exception(Exception::StoreMisaligned) => {
                warn!("页面未对齐");
            }
            Trap::Exception(Exception::IllegalInstruction) => {
                warn!("中断 {:#x} 地址 {:#x} stval: {:#x}", scause.bits(), sepc::read(), stval);
                // panic!("指令页错误");

            }
            Trap::Exception(Exception::InstructionPageFault) => {
                warn!("中断 {:#x} 地址 {:#x} stval: {:#x}", scause.bits(), sepc::read(), stval);
                panic!("指令页错误");
            }
            // 其他情况，终止当前线程
            _ => {
                warn!("未知 中断 {:#x} 地址 {:#x} stval: {:#x}", scause.bits(), sepc::read(), stval);
                return Err(RuntimeError::KillSelfTask);
            },
        }
    
        // 更新TICKS
        set_last_ticks();

        Ok(())
    }
}