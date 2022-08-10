use alloc::{vec::Vec, collections::VecDeque};

use crate::{sync::mutex::Mutex, memory::page::get_free_page_num, task::task_scheduler::add_task_to_scheduler};


use super::exec;

lazy_static! {
    pub static ref TASK_QUEUE: Mutex<VecDeque<&'static str>> = Mutex::new(VecDeque::from(vec![
        // "sh test.sh date.lua",
        // "sh test.sh file_io.lua",
        // "sh test.sh max_min.lua",
        // "sh test.sh random.lua",
        // "sh test.sh remove.lua",
        // "sh test.sh round_num.lua",
        // "sh test.sh sin30.lua",
        // "sh test.sh sort.lua",
        // "sh test.sh strings.lua",
        // "sh busybox_testcode.sh",
        // "ls"
        "busybox du"

        // "sh echo_busybox.sh"
        // "cat busybox_cmd.txt"

        // 减少内存消耗 所以暂时不用下方的
        // "busybox sh lua_testcode.sh"
        // "lmbench_all"

        // "busybox sh echo.sh"
        // "busybox cat ./busybox_cmd.txt"
        // "lua date.lua",
        // "lua file_io.lua",
        // "lua max_min.lua",
        // "lua random.lua",
        // "lua remove.lua",
        // "lua round_num.lua",
        // "lua sin30.lua",
        // "lua sort.lua",
        // "lua strings.lua",
        // "busybox echo latency measurements",
        // "lmbench_all lat_syscall -P 1 null",

    ]));
}


pub fn exec_by_str(str: &str) {
    debug!("执行任务: {}", str);
    let args: Vec<&str> = str.split(" ").collect();
    if let Ok(task) = exec(args[0], args[0..].to_vec()) {
        add_task_to_scheduler(task);
    }
}

// 加载下一个任务
pub fn load_next_task() -> bool {
    if let Some(pro_name) = TASK_QUEUE.lock().pop_front() {
        info!("剩余页表: {}", get_free_page_num());
        exec_by_str(pro_name);
        true
    } else {
        info!("剩余页表: {}", get_free_page_num());
        false
    }
}

