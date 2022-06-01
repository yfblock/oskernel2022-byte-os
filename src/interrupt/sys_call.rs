use core::slice;

use alloc::string::String;
use riscv::register::satp;

use crate::{console::puts, task::{STDOUT, STDIN, STDERR, kill_current_task, get_current_task}, memory::{page_table::PageMapping, addr::{VirtAddr, PhysPageNum, PhysAddr}}, sbi::shutdown, fs::filetree::FILETREE};

use super::Context;

pub const SYS_OPENAT:usize  = 56;
pub const SYS_CLOSE: usize  = 57;
pub const SYS_READ:  usize  = 63;
pub const SYS_WRITE: usize  = 64;
pub const SYS_EXIT:  usize  = 93;
pub const SYS_BRK:   usize  = 214;


pub fn sys_write(fd: usize, buf: usize, count: usize) -> usize {
    // 根据satp中的地址构建PageMapping 获取当前的映射方式
    let pmm = PageMapping::from(PhysPageNum(satp::read().bits()).to_addr());
    let buf = pmm.get_phys_addr(VirtAddr::from(buf)).unwrap();

    // 寻找物理地址
    let buf = unsafe {slice::from_raw_parts_mut(usize::from(buf) as *mut u8, count)};
    match fd {
        STDIN => {

        },
        STDOUT => {
            puts(buf);
        },
        STDERR => {

        },
        _=>{
            info!("暂未找到中断地址");
        }
    };
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

pub fn sys_call(context: &mut Context) {
    // a7(x17) 作为调用号
    match context.x[17] {
        SYS_OPENAT => {
            let current_task_wrap = get_current_task().unwrap();
            let mut current_task = current_task_wrap.force_get();
            let pmm = PageMapping::from(PhysPageNum(satp::read().bits()).to_addr());
            let fd = context.x[10];
            let filename = pmm.get_phys_addr(VirtAddr::from(context.x[11])).unwrap();
            let filename = get_string_from_raw(filename);
            let flags = context.x[12];
            let open_mod = context.x[13];


            // 判断文件描述符是否存在
            if fd == 0xffffffffffffff9c {
                if let Ok(file) = FILETREE.lock().open(&filename) {
                    // let file = file.to_file();
                    // let 
                    let fd = current_task.alloc_fd();
                    current_task.fd_table[fd] = Some(file.clone());
                    context.x[10] = fd;
                } else {
                    let result_code: isize = -1;
                    context.x[10] = result_code as usize;
                }
            } else {
                if let Some(tree_node) = current_task.fd_table[fd].clone() {
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
            sys_write(context.x[10],context.x[11],context.x[12]);
            context.x[10] = context.x[12];
        },
        SYS_EXIT => {
            kill_current_task();
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
        _ => {
            info!("未识别调用号 {}", context.x[17]);
        }
    }
    context.sepc = context.sepc + 4;
}