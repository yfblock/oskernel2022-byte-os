use alloc::rc::Rc;

use crate::{memory::{page_table::{PageMappingManager, PTEFlags}, mem_set::MemSet, addr::{PhysAddr, PAGE_SIZE, get_buf_from_phys_page, PhysPageNum}, mem_map::MemMap}, runtime_err::RuntimeError};

pub const DEFAULT_HEAP_BOTTOM: usize = 0x10f000;
// pub const DEFAULT_HEAP_BOTTOM: usize = 0x10c000;
// pub const DEFAULT_HEAP_BOTTOM: usize = 0x0020_0000;
pub const DEFAULT_HEAP_PAGE_NUM: usize = 5;

#[allow(dead_code)]
// 用户heap
pub struct UserHeap {
    start: usize,
    pointer: usize,
    end: usize,
    temp: usize,
    pmm: Rc<PageMappingManager>,
    mem_set: MemSet
}

impl UserHeap {
    // 创建heap
    pub fn new(pmm: Rc<PageMappingManager>) -> Result<Self, RuntimeError> {
        let mem_set = MemSet::new();
        // 申请页表作为heap
        Ok(UserHeap {
            start: 0,
            pointer: 0,
            end: 0,
            temp: 0,
            pmm,
            mem_set
        })
    }

    // 获取堆开始的地址
    pub fn get_addr(&self) -> PhysAddr {
        self.start.into()
    }

    pub fn get_heap_size(&self) -> usize {
        self.end - self.start
    }

    pub fn get_heap_top(&self) -> usize {
        self.pointer
    }

    pub fn set_heap_top(&mut self, top: usize) -> Result<usize, RuntimeError>{
        if self.start == 0 {
            debug!("设置heap: {:#x}", top);
            self.start = top;
            self.pointer = top;
            let mem_map = MemMap::new((top / PAGE_SIZE).into(), DEFAULT_HEAP_PAGE_NUM, PTEFlags::VRWX | PTEFlags::U)?;
            self.pmm.add_mapping_by_map(&mem_map)?;
            self.mem_set.0.push(mem_map);
            self.end = top + DEFAULT_HEAP_PAGE_NUM * PAGE_SIZE;
            return Ok(top);

            // debug!("设置heap: {:#x}", DEFAULT_HEAP_BOTTOM);
            // self.start = DEFAULT_HEAP_BOTTOM;
            // self.pointer = DEFAULT_HEAP_BOTTOM;
            // let mem_map = MemMap::new((DEFAULT_HEAP_BOTTOM / PAGE_SIZE).into(), DEFAULT_HEAP_PAGE_NUM, PTEFlags::VRWX | PTEFlags::U)?;
            // self.pmm.add_mapping_by_map(&mem_map)?;
            // self.mem_set.0.push(mem_map);
            // self.end = DEFAULT_HEAP_BOTTOM + DEFAULT_HEAP_PAGE_NUM * PAGE_SIZE;
            // return Ok(DEFAULT_HEAP_BOTTOM);
        }

        let _origin_top = self.pointer;
        self.pointer = top;
        // origin_top
        if self.pointer < self.end {
            Ok(top)
        } else {
            Ok(-1 as isize as usize)
        }
    }

    // 获取临时页表
    pub fn get_temp(&mut self, pmm: Rc<PageMappingManager>) -> Result<PhysAddr, RuntimeError>{
        if self.temp == 0 {
            let mem_map = MemMap::new(0xe0000usize.into(), 1, PTEFlags::UVRWX).unwrap();
            self.temp = mem_map.ppn.into();
            pmm.add_mapping(mem_map.ppn, mem_map.vpn, PTEFlags::UVRWX)?;
            // self.pmm.add_mapping_by_map(&mem_map).expect("临时页表申请内存不足");
            self.mem_set.0.push(mem_map);
        }
        Ok(PhysPageNum::from(self.temp).into())
    }

    pub fn release_temp(&self) {
        get_buf_from_phys_page(self.temp.into(), 1).fill(0)
    }

    pub fn clone(&self, pmm: Rc<PageMappingManager>) -> Result<Self, RuntimeError> {
        let mem_set = self.mem_set.clone_with_data()?;
        pmm.add_mapping_by_set(&mem_set)?;
        Ok(Self {
            start: self.start,
            pointer: self.pointer,
            end: self.end,
            temp: self.temp,
            pmm,
            mem_set
        })
    }
}