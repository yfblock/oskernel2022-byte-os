use _core::{arch::asm, ops::Add};
use bitflags::*;
use riscv::register::satp;

use crate::{memory::{addr::PhysAddr}, sync::mutex::Mutex};

use super::{addr::{PhysPageNum,  VirtAddr, PAGE_PTE_NUM}, page::PAGE_ALLOCATOR};

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;       // 是否合法 为1合法
        const R = 1 << 1;       // 可读
        const W = 1 << 2;       // 可写
        const X = 1 << 3;       // 可执行
        const U = 1 << 4;       // 处于U特权级下是否允许被访问
        const G = 1 << 5;       // 
        const A = 1 << 6;       // 是否被访问过
        const D = 1 << 7;       // 是否被修改过
        const None = 0;
        const VRWX = 0xf;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry {
            bits: 0,
        }
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    // 判断是否为页表
    pub fn is_valid_pte(&self) -> bool {
        self.flags().contains(PTEFlags::V) && self.flags() & PTEFlags::VRWX != PTEFlags::V
    }

    // 判断是否为页目录
    pub fn is_valid_pd(&self) -> bool {
        self.flags().contains(PTEFlags::V) && self.flags() & PTEFlags::VRWX == PTEFlags::V
    }

    pub unsafe fn get_mut_ptr_from_Phys(addr:PhysAddr) -> *mut Self {
        usize::from(addr) as *mut Self
    }
}

#[derive(Clone)]
pub enum PagingMode {
    Bare = 0,
    Sv39 = 8,
    Sv48 = 9
}

pub struct PageMappingManager {
    paging_mode: PagingMode,
    pte: PhysAddr
}

impl PageMappingManager {

    pub fn new() -> Self {
        PageMappingManager { 
            paging_mode: PagingMode::Sv39, 
            pte: PhysAddr::from(0)
        }
    }

    // 获取pte
    pub fn get_pte(&self) -> usize {
        self.pte.into()
    }

    // 初始化页表
    pub fn alloc_pte(&self, level: usize) -> Option<PhysPageNum> {
        match PAGE_ALLOCATOR.lock().alloc() {
            Some(page) => {
                let pte = unsafe {
                    &mut *((usize::from(PhysAddr::from(page)))as *mut [PageTableEntry; PAGE_PTE_NUM])
                };
                Some(page)
            }
            None=>None
        }
        
    }

    // 添加mapping
    pub fn add_mapping(&mut self, phy_addr: PhysAddr, virt_addr: VirtAddr, flags:PTEFlags) {
        // 如果没有pte则申请pte
        if usize::from(self.pte) == 0 {
            info!("申请pte");
            self.pte = PhysAddr::from(self.alloc_pte(2).unwrap());
        }

        // 得到 列表中的项
        let l2_pte_ptr = unsafe {
            PageTableEntry::get_mut_ptr_from_Phys(self.pte).add(virt_addr.l2())
        };
        let mut l2_pte = unsafe { l2_pte_ptr.read() };

        // 判断 是否是页表项 如果是则申请一个页防止其内容
        if !l2_pte.is_valid_pd() {
            info!("申请二级页表");
            // 创建一个页表放置二级页目录 并写入一级页目录的项中
            l2_pte = PageTableEntry::new(PhysPageNum::from(PhysAddr::from(self.alloc_pte(1).unwrap())), PTEFlags::V);
            // 写入列表
            unsafe {l2_pte_ptr.write(l2_pte)};
        }

        let l1_pte_ptr = unsafe {
            PageTableEntry::get_mut_ptr_from_Phys(PhysAddr::from(l2_pte.ppn())).add(virt_addr.l1())
        };
        let mut l1_pte = unsafe {l1_pte_ptr.read()};

        // 判断 是否有指向下一级的页表
        if !l1_pte.is_valid_pd(){
            l1_pte = PageTableEntry::new(PhysPageNum::from(PhysAddr::from(self.alloc_pte(0).unwrap())), PTEFlags::V);
            unsafe{l1_pte_ptr.write(l1_pte)};
        }
        
        // 写入映射项
        unsafe {
            PageTableEntry::get_mut_ptr_from_Phys(PhysAddr::from(l1_pte.ppn()))
                .add(virt_addr.l0()).write(PageTableEntry::new(PhysPageNum::from(phy_addr), flags));
        }
    }

    // 获取物理地址
    pub fn get_phys_addr(&self, virt_addr: VirtAddr) -> Option<PhysAddr> {
        // 如果没有pte则申请pte
        if usize::from(self.pte) == 0 {
            return None;
        }

        // 得到 列表中的项
        let l2_pte_ptr = unsafe {
            PageTableEntry::get_mut_ptr_from_Phys(self.pte).add(virt_addr.l2())
        };
        let l2_pte = unsafe { l2_pte_ptr.read() };

        // 判断 是否有指向下一级的页表
        if !l2_pte.flags().contains(PTEFlags::V) {
            return None;
        }
        if l2_pte.flags() & PTEFlags::VRWX != PTEFlags::V {
            return Some(PhysAddr::from(virt_addr.page_offset() | (virt_addr.l0() << 12) | (virt_addr
                .l1() << 21) | (usize::from(l2_pte.ppn()) << 12)));
        }

        let l1_pte_ptr = unsafe {
            PageTableEntry::get_mut_ptr_from_Phys(PhysAddr::from(l2_pte.ppn())).add(virt_addr.l1())
        };
        let l1_pte = unsafe { l1_pte_ptr.read() };

        // 判断 是否有指向下一级的页表
        if !l1_pte.flags().contains(PTEFlags::V) {
            return None;
        }
        if l1_pte.flags() & PTEFlags::VRWX != PTEFlags::V {
            info!("页表地址: {:?}", PhysAddr::from(l1_pte.ppn()));
            return Some(PhysAddr::from(virt_addr.page_offset() | (virt_addr.l0() << 12) | (usize::from(l1_pte.ppn()) << 12)));
        }

        // 获取pte项
        let l0_pte_ptr = unsafe {
            PageTableEntry::get_mut_ptr_from_Phys(PhysAddr::from(l1_pte.ppn())).add(virt_addr.l0())
        };
        let l0_pte = unsafe { l0_pte_ptr.read() };
        if !l0_pte.flags().contains(PTEFlags::V) {
            return None;
        }
        Some(PhysAddr::from(usize::from(PhysAddr::from(l0_pte.ppn())) + virt_addr.page_offset()))
    }

    // 更改pte
    pub fn change_satp(&self) {
        let satp_addr = (self.paging_mode.clone() as usize) << 60 | usize::from(PhysPageNum::from(self.pte));
        info!("");
        unsafe {
            asm!("csrw satp, a0",
            "sfence.vma", in("a0") satp_addr)
        }
    }
}

lazy_static! {
    pub static ref KERNEL_PAGE_MAPPING: Mutex<PageMappingManager> = Mutex::new(PageMappingManager::new());
}

// 初始化页面映射
pub fn init() {
    let mut mapping_manager = KERNEL_PAGE_MAPPING.lock();
    for i in (0x80000000..0x80800000).step_by(4096) {
        mapping_manager.add_mapping(PhysAddr::from(i), VirtAddr::from(i), PTEFlags::VRWX);
    }
    mapping_manager.add_mapping(PhysAddr::from(0x10001070), VirtAddr::from(0x10001070), PTEFlags::VRWX);

    // mapping_manager.add_mapping(PhysAddr::from(0x10001070), VirtAddr::from(0x10001070), PTEFlags::VRWX);
    if let Some(end_addr) = mapping_manager.get_phys_addr(VirtAddr::from(0x80000000)) {
        info!("物理地址: {:?} 虚拟地址:{:?}", end_addr, VirtAddr::from(0x80000000 as usize));
    } else {
        info!("未找到物理地址");
    }

    if let Some(end_addr) = mapping_manager.get_phys_addr(VirtAddr::from(0x80212321)) {
        info!("物理地址: {:?} 虚拟地址:{:?}", end_addr, VirtAddr::from(0x80212321 as usize));
    } else {
        info!("未找到物理地址");
    }

    mapping_manager.add_mapping(PhysAddr::from(0x80500000), VirtAddr::from(0x1000), PTEFlags::VRWX);
    let a = 0x80500000 as *mut usize;
    unsafe {a.write(123)};
    if let Some(end_addr) = mapping_manager.get_phys_addr(VirtAddr::from(0x1023)) {
        info!("物理地址: {:?} 虚拟地址:{:?}", end_addr, VirtAddr::from(0x1023));
    } else {
        info!("未找到物理地址");
    }

    mapping_manager.change_satp();
    let b = 0x1000 as *mut usize;
    unsafe{info!("b is {}", b.read())}

}