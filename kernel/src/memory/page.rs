use alloc::vec::Vec;

use crate::{sync::mutex::Mutex, memory::addr::PAGE_SIZE, runtime_err::RuntimeError};

use super::addr::PhysPageNum;

const ADDR_END: usize = 0x80800000;

// 内存页分配器
pub struct MemoryPageAllocator {
    pub start: usize,
    pub end: usize,
    pub pages: Vec<bool>
}


// 添加内存页分配器方法
impl MemoryPageAllocator {
    // 创建内存分配器结构
    fn new() -> Self {
        MemoryPageAllocator {
            start: 0,
            end: 0,
            pages: vec![]
        }
    }

    // 初始化内存分配器
    fn init(&mut self, start: usize, end: usize) {
        self.start = start;
        self.end = end;
        self.pages = vec![false;(end - start) / PAGE_SIZE];
        info!("初始化页式内存管理, 页表数: {}", self.pages.capacity());
    }

    // 申请内存
    pub fn alloc(&mut self) -> Option<PhysPageNum> {
        for i in 0..self.pages.len() {
            if !self.pages[i] {
                self.pages[i] = true;
                return Some(PhysPageNum::from((self.start >> 12) + i));
            }
        }
        None
    }

    // 取消分配页
    pub fn dealloc(&mut self, page: PhysPageNum) {
        let index = usize::from(page) - (self.start >> 12); 
        if let Some(_) = self.pages.get(index) {
            self.pages[index] = false;
        }
    }

    // 申请多个页
    pub fn alloc_more(&mut self, pages: usize) -> Result<PhysPageNum, RuntimeError> {
        let mut i = 0;
        loop {
            if i >= self.pages.len() {
                break;
            }

            if !self.pages[i] {
                let mut is_ok = true;
                // 判断后面是否连续未被使用
                for j in 1..pages {
                    if self.pages[i+j] {
                        is_ok = false;
                        i=i+j;
                    }
                }
                if is_ok {
                   for j in 0..pages {
                       self.pages[i+j] = true;
                   } 
                   return Ok(PhysPageNum::from((self.start >> 12) + i));
                }
            }

            // 进行下一个计算
            i+=1;
        }
        for i in 0..self.pages.len() {
            if !self.pages[i] {
                self.pages[i] = true;
                return Ok(PhysPageNum::from((self.start >> 12) + i));
            }
        }
        Err(RuntimeError::NoEnoughPage)
    }

    // 释放多个页
    pub fn dealloc_more(&mut self, page: PhysPageNum, pages: usize) {
        let index = usize::from(page) - (self.start >> 12); 
        if let Some(_) = self.pages.get(index) {
            for i in 0..pages {
                self.pages[index + i] = false;
            }
        }
    }
}

lazy_static! {
    pub static ref PAGE_ALLOCATOR: Mutex<MemoryPageAllocator> = Mutex::new(MemoryPageAllocator::new());
}

pub fn init() {
    extern "C"{
        fn end();
    }

    // 初始化页表 Vector中每一个元素代表一个页表 通过这种方法来分配页表
    PAGE_ALLOCATOR.lock().init(end as usize, ADDR_END);
}