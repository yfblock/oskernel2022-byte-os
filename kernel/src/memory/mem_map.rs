use crate::runtime_err::RuntimeError;

use super::{addr::{PhysPageNum, VirtPageNum, VirtAddr, PAGE_SIZE, get_buf_from_phys_page}, page::alloc_more, page_table::PTEFlags};

pub struct MemMap {
    pub ppn: PhysPageNum,
    pub vpn: VirtPageNum,
    pub page_num: usize,
    pub flags: PTEFlags
}

impl MemMap {
    // 申请开始页表和页表数量 申请内存
    pub fn new(vpn: VirtPageNum, page_num: usize, flags: PTEFlags) -> Result<Self, RuntimeError> {
        let phys_num_start = alloc_more(page_num)?;
        Ok(Self {
            ppn: phys_num_start,
            vpn,
            page_num,
            flags
        })
    }

    // 获取pte容器地址
    pub fn pte_container(ppn: PhysPageNum) -> Self {
        Self {
            ppn,
            vpn: VirtPageNum::default(),
            page_num: 1,
            flags: PTEFlags::V
        }
    }

    // 通过虚拟地址申请内存map
    pub fn alloc_range(start_va: VirtAddr, end_va: VirtAddr, flags: PTEFlags) -> Result<Self, RuntimeError> {
        let start_page: usize = start_va.0 / PAGE_SIZE * PAGE_SIZE;   // floor get start_page
        let end_page: usize = (end_va.0 + PAGE_SIZE - 1) / PAGE_SIZE;  
        let page_num = end_page - start_page;
        let phys_num_start = alloc_more(page_num)?;
        Ok(Self {
            ppn: phys_num_start,
            vpn: VirtPageNum::from(start_va),
            page_num,
            flags
        })
    }

    // 添加已经映射的页
    pub fn exists_page(ppn: PhysPageNum, vpn: VirtPageNum, page_num: usize, flags: PTEFlags) -> Self {
        Self { 
            ppn, 
            vpn, 
            page_num, 
            flags
        }
    }

    pub fn clone_with_data(&self) -> Result<Self, RuntimeError> {
        let page_num = self.page_num;
        let phys_num_start = alloc_more(page_num)?;

        // 复制数据
        let new_data = get_buf_from_phys_page(phys_num_start, self.page_num);
        let old_data = get_buf_from_phys_page(self.ppn, self.page_num);
        new_data.clone_from_slice(old_data);


        Ok(Self {
            ppn: phys_num_start,
            vpn: self.vpn,
            page_num,
            flags: self.flags.clone()
        })
    }
}