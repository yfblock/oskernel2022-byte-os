pub mod timer;

use core::arch::{global_asm, asm};
use riscv::register::{scause::{Trap, Exception, Interrupt,Scause}, sepc};

pub use timer::TICKS;

use crate::memory::{addr::{VirtAddr, PhysAddr},  page_table::{PTEFlags, KERNEL_PAGE_MAPPING}};



#[repr(C)]
#[derive(Debug)]
// 上下文
pub struct Context {
    pub x: [usize; 32],     // 32 个通用寄存器
    pub sstatus: usize,
    pub sepc: usize
}

impl Context {
    // 创建上下文信息
    pub fn new() -> Self {
        Context {
            x: [0usize; 32],
            sstatus: 0,
            sepc: 0
        }
    }
    // 从另一个上下文复制
    pub fn clone_from(&mut self, target: &Self) {
        for i in 0..32 {
            self.x[i] = target.x[i];
        }

        self.sstatus = target.sstatus;
        self.sepc = target.sepc;
    }
}

// break中断
fn breakpoint(context: &mut Context) {
    warn!("break中断产生 中断地址 {:#x}", sepc::read());
    context.sepc = context.sepc + 2;
}

// 中断错误
fn fault(_context: &mut Context, scause: Scause, stval: usize) {
    info!("中断 {:#x} 地址 {:#x} stval: {:#x}", scause.bits(), sepc::read(), stval);
    panic!("未知中断")
}

// 处理缺页异常
fn handle_page_fault(stval: usize) {
    warn!("缺页中断触发 缺页地址: {:#x} 触发地址:{:#x} 已同步映射", stval, sepc::read());
    panic!("end");
    KERNEL_PAGE_MAPPING.lock().add_mapping(PhysAddr::from(stval).into(), 
        VirtAddr::from(stval).into(), PTEFlags::VRWX).expect("缺页处理异常");
    unsafe{
        asm!("sfence.vma {x}", x = in(reg) stval)
    };
}

// 内核中断回调
#[no_mangle]
fn kernel_callback(context: &mut Context, scause: Scause, stval: usize) -> usize {
    warn!("中断发生: {:#x}  stval {:#x}  sepc: {:#x}", scause.bits(), stval,  context.sepc);
    match scause.cause(){
        // 中断异常
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => timer::timer_handler(),
        // 缺页异常
        Trap::Exception(Exception::StorePageFault) => handle_page_fault(stval),
        // 加载页面错误
        Trap::Exception(Exception::LoadPageFault) => {
            panic!("加载权限异常 地址:{:#x}", stval)
        },
        Trap::Exception(Exception::InstructionPageFault) => handle_page_fault(stval),
        // 页面未对齐异常
        Trap::Exception(Exception::StoreMisaligned) => {
            info!("页面未对齐");
        }
        // 其他情况，终止当前线程
        _ => fault(context, scause, stval),
    }
    context as *const Context as usize
}


// 包含中断代码
global_asm!(include_str!("interrupt-kernel.asm"));

// 设置中断
pub fn init() {
    extern "C" {
        fn kernel_callback_entry();
    }

    // 输出内核信息
    info!("kernel_callback_entry addr: {:#x}", kernel_callback_entry as usize);

    unsafe {
        asm!("csrw stvec, a0", in("a0") kernel_callback_entry as usize);
    }

    // 初始化定时器
    timer::init();
}

// 调试代码
pub fn test() {
    unsafe {asm!("ebreak")};
}