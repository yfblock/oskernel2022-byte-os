use core::slice;

use alloc::string::String;
use riscv::register::satp;

use crate::{console::puts, task::{kill_current_task, get_current_task, exec, clone_task, TASK_CONTROLLER_MANAGER, suspend_and_run_next, wait_task}, memory::{page_table::PageMapping, addr::{VirtAddr, PhysPageNum, PhysAddr}}, sbi::shutdown, fs::{filetree::{FILETREE, FileTreeNode}, file, self}, print_file_tree, interrupt::{timer::get_time_ms, TICKS}};

use super::{Context,  timer::TimeSpec};

pub const SYS_GETCWD:usize  = 17;
pub const SYS_DUP: usize    = 23;
pub const SYS_DUP3: usize   = 24;
pub const SYS_MKDIRAT:usize = 34;
pub const SYS_UMOUNT2: usize= 39;
pub const SYS_MOUNT: usize  = 40;
pub const SYS_CHDIR: usize  = 49;
pub const SYS_OPENAT:usize  = 56;
pub const SYS_CLOSE: usize  = 57;
pub const SYS_READ:  usize  = 63;
pub const SYS_WRITE: usize  = 64;
pub const SYS_EXIT:  usize  = 93;
pub const SYS_NANOSLEEP: usize = 101;
pub const SYS_SCHED_YIELD: usize = 124;
pub const SYS_UNAME: usize  = 160;
pub const SYS_GETTIMEOFDAY: usize= 169;
pub const SYS_GETPID:usize  = 172;
pub const SYS_GETPPID:usize = 173;
pub const SYS_BRK:   usize  = 214;
pub const SYS_CLONE: usize  = 220;
pub const SYS_EXECVE:usize  = 221;
pub const SYS_WAIT4: usize  = 260;

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 6;
        const TRUNC = 1 << 10;
        const O_DIRECTORY = 1 << 21;
    }
}

pub struct UTSname  {
    sysname: [u8;65],
    nodename: [u8;65],
    release: [u8;65],
    version: [u8;65],
    machine: [u8;65],
    domainname: [u8;65],
}

pub fn sys_write(fd: FileTreeNode, buf: usize, count: usize) -> usize {
    // 根据satp中的地址构建PageMapping 获取当前的映射方式
    let pmm = PageMapping::from(PhysPageNum(satp::read().bits()).to_addr());
    let buf = pmm.get_phys_addr(VirtAddr::from(buf)).unwrap();

    // 寻找物理地址
    let buf = unsafe {slice::from_raw_parts_mut(usize::from(buf) as *mut u8, count)};
    
    if fd.is_device() {
        let device_name = fd.get_filename();
        if device_name == "STDIN" {

        } else if device_name == "STDOUT" {
            puts(buf);
        } else if device_name == "STDERR" {

        } else {
            info!("未找到设备!");
        }
    } else {
        info!("暂未找到中断地址");
    }
    count
}

pub fn get_string_from_raw(addr: PhysAddr) -> String {
    let mut ptr = addr.as_ptr();
    let mut str: String = String::new();
    loop {
        let ch = unsafe { *ptr };
        if ch == 0 {
            break;
        }
        str.push(ch as char);
        unsafe { ptr = ptr.add(1) };
    }
    str
}

pub fn write_string_to_raw(target: &mut [u8], str: &str) {
    let mut index = 0;
    for c in str.chars() {
        target[index] = c as u8;
        index = index + 1;
    }
    target[index] = 0;
}

pub fn sys_call(context: &mut Context) {
    let current_task_wrap = get_current_task().unwrap();
    let mut current_task = current_task_wrap.force_get();
    let context: &mut Context = &mut current_task.context;
    let pmm = PageMapping::from(PhysPageNum(satp::read().bits()).to_addr());
    context.sepc = context.sepc + 4;
    // a7(x17) 作为调用号
    match context.x[17] {
        SYS_GETCWD => {
            let current_task_wrap = get_current_task().unwrap();
            let current_task = current_task_wrap.force_get();
            // 内存映射管理器
            let pmm = PageMapping::from(PhysPageNum(satp::read().bits()).to_addr());
            // 获取参数
            let mut buf = pmm.get_phys_addr(VirtAddr::from(context.x[10])).unwrap();
            let size = context.x[11];
            let buf = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr(), size) };
            let pwd = current_task.home_dir.get_pwd();

            let pwd_buf = pwd.as_bytes();
            buf[..pwd_buf.len()].copy_from_slice(pwd_buf);
            context.x[10] = buf.as_ptr() as usize;
        }
        SYS_DUP => {
            let fd = context.x[10];
            let mut current_task = current_task_wrap.force_get();
            let new_fd = current_task.alloc_fd();
            if let Some(tree_node) = current_task.fd_table[fd].clone() {
                current_task.fd_table[new_fd] = Some(tree_node);
                context.x[10] = new_fd;
            } else {
                context.x[10] = -1 as isize as usize;
            }
        }
        SYS_DUP3 => {
            let fd = context.x[10];
            let new_fd = context.x[11];
            let mut current_task = current_task_wrap.force_get();
            if let Some(tree_node) = current_task.fd_table[fd].clone() {
                // 申请空间
                if current_task.alloc_fd_with_size(new_fd) == -1 as isize as usize {
                    context.x[10] = -1 as isize as usize;
                } else {
                    current_task.fd_table[new_fd] = Some(tree_node);
                    context.x[10] = new_fd;
                }
            } else {
                context.x[10] = -1 as isize as usize;
            }
        }
        SYS_MKDIRAT => {
            let current_task = current_task_wrap.force_get();
            let pmm = PageMapping::from(PhysPageNum(satp::read().bits()).to_addr());
            let dirfd = context.x[10];
            let filename = pmm.get_phys_addr(VirtAddr::from(context.x[11])).unwrap();
            let filename = get_string_from_raw(filename);
            let flags = context.x[12];

            // 判断文件描述符是否存在
            if dirfd == 0xffffffffffffff9c {
                FILETREE.lock().open("/").unwrap().mkdir(&filename, flags as u16);
                context.x[10] = 0;
            } else {
                if let Some(mut tree_node) = current_task.fd_table[dirfd].clone() {
                    tree_node.mkdir(&filename, flags as u16);
                } else {
                    let result_code: isize = -1;
                    context.x[10] = result_code as usize;
                }
            };
        },
        SYS_UMOUNT2 => {
            let special_ptr = pmm.get_phys_addr(VirtAddr::from(context.x[10])).unwrap();
            let flag = context.x[11];
            context.x[10] = 0;
        },
        SYS_MOUNT => {
            let special_ptr = pmm.get_phys_addr(VirtAddr::from(context.x[10])).unwrap();
            let dir_ptr = pmm.get_phys_addr(VirtAddr::from(context.x[11])).unwrap();
            let fstype_ptr = pmm.get_phys_addr(VirtAddr::from(context.x[12])).unwrap();
            let flag = context.x[13];
            let data_ptr = context.x[14];
            let special = get_string_from_raw(special_ptr);
            let dir = get_string_from_raw(dir_ptr);
            let fstype = get_string_from_raw(fstype_ptr);
            // info!("special: {}, dir: {}, fstype: {}", special, dir, fstype);
            // let special = get_string_from_raw(fstype_ptr);
            context.x[10] = 0;
        },
        SYS_CHDIR => {
            let current_task_wrap = get_current_task().unwrap();
            let mut current_task = current_task_wrap.force_get();
            let pmm = PageMapping::from(PhysPageNum(satp::read().bits()).to_addr());
            let filename = pmm.get_phys_addr(VirtAddr::from(context.x[10])).unwrap();
            let filename = get_string_from_raw(filename);
            if let Ok(file) = FILETREE.lock().open(&filename) {
                // let file = file.to_file();
                // let 
                current_task.home_dir = file.clone();
                context.x[10] = 0;
            } else {
                let result_code: isize = -1;
                context.x[10] = result_code as usize;
            }
            
        }
        SYS_OPENAT => {
            let current_task_wrap = get_current_task().unwrap();
            let mut current_task = current_task_wrap.force_get();
            let pmm = PageMapping::from(PhysPageNum(satp::read().bits()).to_addr());
            let fd = context.x[10];
            let filename = pmm.get_phys_addr(VirtAddr::from(context.x[11])).unwrap();
            let filename = get_string_from_raw(filename);
            let flags = context.x[12];
            let open_mod = context.x[13];

            let flags = OpenFlags::from_bits(flags as u32).unwrap();

            if let Ok(file) = FILETREE.lock().open(&filename) {
                let fd = current_task.alloc_fd();
                current_task.fd_table[fd] = Some(file.clone());
                context.x[10] = fd;
            } else {
                let result_code: isize = -1;
                context.x[10] = result_code as usize;
            }


            // 判断文件描述符是否存在
            if fd == 0xffffffffffffff9c {
                if let Ok(file) = FILETREE.lock().open(&filename) {
                    let fd = current_task.alloc_fd();
                    current_task.fd_table[fd] = Some(file.clone());
                    context.x[10] = fd;
                } else {
                    let result_code: isize = -1;
                    context.x[10] = result_code as usize;
                }
            } else {
                if let Some(mut tree_node) = current_task.fd_table[fd].clone() {
                    if flags.contains(OpenFlags::CREATE) {
                        tree_node.create(&filename);
                    }
                    if let Ok(file) = tree_node.open(&filename) {
                        let fd = current_task.alloc_fd();
                        current_task.fd_table[fd] = Some(file.clone());
                        context.x[10] = fd;
                    } else {
                        let result_code: isize = -1;
                        context.x[10] = result_code as usize;
                    }
                } else {
                    let result_code: isize = -1;
                    context.x[10] = result_code as usize;
                }
            };
        }
        SYS_CLOSE => {
            let current_task_wrap = get_current_task().unwrap();
            let mut current_task = current_task_wrap.force_get();
            let fd = context.x[10];
            if let Some(_) = current_task.fd_table[fd].clone() {
                current_task.fd_table[fd] = None;
                context.x[10] = 0;
            } else {
                let result_code: isize = -1;
                context.x[10] = result_code as usize;
            }
        }
        SYS_READ => {
            // 当前任务
            let current_task_wrap = get_current_task().unwrap();
            let current_task = current_task_wrap.force_get();
            // 内存映射管理器
            let pmm = PageMapping::from(PhysPageNum(satp::read().bits()).to_addr());
            // 获取参数
            let fd = context.x[10];
            let mut buf = pmm.get_phys_addr(VirtAddr::from(context.x[11])).unwrap();
            let count = context.x[12];
            let buf = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr(), count) };

            if let Some(file_tree_node) = current_task.fd_table[fd].clone() {
                let size = file_tree_node.to_file().read_to(buf);
                // buf[buf.len() - 1] = 0;
                context.x[10] = size as usize;
            } else {
                let result_code: isize = -1;
                context.x[10] = result_code as usize;
            }
            
        }
        SYS_WRITE => {
            let fd = context.x[10];
            let current_task = current_task_wrap.force_get();
            if let Some(file_tree_node) = current_task.fd_table[fd].clone() {
                sys_write(file_tree_node,context.x[11],context.x[12]);
                context.x[10] = context.x[12];
            } else {
                context.x[10] = -1 as isize as usize;
            }
        },
        SYS_EXIT => {
            kill_current_task();
        },
        SYS_NANOSLEEP => {
            let req_ptr = usize::from(pmm.get_phys_addr(VirtAddr::from(context.x[10])).unwrap()) as *mut TimeSpec;
            let req = unsafe { req_ptr.as_mut().unwrap() };
            let rem_ptr = usize::from(pmm.get_phys_addr(VirtAddr::from(context.x[10])).unwrap()) as *mut TimeSpec;
            let rem = unsafe { rem_ptr.as_mut().unwrap() };
            // 如果 nsec < 0则此任务已被处理 nsec = - remain_ticks
            if rem.tv_nsec < 0 {
                let remain_ticks = (-rem.tv_nsec) as usize;
                if remain_ticks <= unsafe {TICKS} {
                    context.x[10] = 0;
                } else {
                    context.sepc = context.sepc - 4;
                    suspend_and_run_next();
                }
            } else {
                // 1秒100个TICKS  1ns = 1/1000ms = 1/10000TICKS
                let wake_ticks = req.tv_sec * 100 + req.tv_nsec as u64 / 10000;
                let remain_ticks = wake_ticks + unsafe {TICKS as u64};

                rem.tv_nsec = - (remain_ticks as i64);
                // 减少spec进行重复请求
                context.sepc = context.sepc - 4;
                suspend_and_run_next();
            }
        },
        SYS_SCHED_YIELD => {
            suspend_and_run_next();
        },
        SYS_UNAME => {
            let sys_info = usize::from(pmm.get_phys_addr(VirtAddr::from(context.x[10])).unwrap()) as *mut UTSname;
            let sys_info = unsafe { sys_info.as_mut().unwrap() };
            write_string_to_raw(&mut sys_info.sysname, "ByteOS");
            write_string_to_raw(&mut sys_info.nodename, "ByteOS");
            write_string_to_raw(&mut sys_info.release, "release");
            write_string_to_raw(&mut sys_info.version, "alpha 1.1");
            write_string_to_raw(&mut sys_info.machine, "riscv k210");
            write_string_to_raw(&mut sys_info.domainname, "alexbd.cn");
            context.x[10] = 0;
        },
        SYS_GETTIMEOFDAY => {
            let timespec = usize::from(pmm.get_phys_addr(VirtAddr::from(context.x[10])).unwrap()) as *mut TimeSpec;
            unsafe { timespec.as_mut().unwrap().get_now() };
            // info!("ms: {}", get_time_ms());
            context.x[10] = 0;
        },
        SYS_GETPID => {
            // 当前任务
            let current_task_wrap = get_current_task().unwrap();
            let current_task = current_task_wrap.force_get();
            context.x[10] = current_task.pid;
        },
        SYS_GETPPID => {
            let current_task_wrap = get_current_task().unwrap();
            let current_task = current_task_wrap.force_get();
            context.x[10] = current_task.ppid;
        },
        SYS_BRK => {
            let top_pos = context.x[10];
            // 如果是0 返回堆顶 否则设置为新的堆顶
            if top_pos == 0 {
                context.x[10] = get_current_task().unwrap().lock().get_heap_size();
            } else {
                let top = get_current_task().unwrap().lock().set_heap_top(top_pos);
                context.x[10] = top;
            }
        }
        SYS_CLONE => {
            let flag = context.x[10];
            let stack_addr = context.x[11];
            let ptid = context.x[12];
            let tls = context.x[13];
            let ctid = context.x[14];


            let mut task = clone_task(&mut current_task_wrap.force_get());
            
            // 如果指定了栈 则设置栈
            if stack_addr > 0 {
                task.context.x[2] = stack_addr;
            }

            task.context.x[10] = 0;
            context.x[10] = task.pid;
            TASK_CONTROLLER_MANAGER.force_get().add(task);
        }
        SYS_EXECVE => {
            let filename = pmm.get_phys_addr(VirtAddr::from(context.x[10])).unwrap();
            let filename = get_string_from_raw(filename);
            exec(&filename);
            kill_current_task();
        }
        SYS_WAIT4 => {
            let pid = context.x[10];
            let ptr = usize::from(pmm.get_phys_addr(VirtAddr::from(context.x[11])).unwrap()) as *mut u16;
            let options = context.x[12];
            wait_task(pid, ptr, options);
        }
        _ => {
            info!("未识别调用号 {}", context.x[17]);
        }
    }
}