use alloc::{vec::Vec, collections::VecDeque};

use crate::{sync::mutex::Mutex, memory::page::get_free_page_num, task::task_scheduler::add_task_to_scheduler};


use super::exec;

lazy_static! {
    pub static ref TASK_QUEUE: Mutex<VecDeque<&'static str>> = Mutex::new(VecDeque::from(vec![
        "runtest.exe -w entry-static.exe argv",
        "runtest.exe -w entry-static.exe env",
        "runtest.exe -w entry-static.exe basename",
        "runtest.exe -w entry-static.exe clock_gettime",
        "runtest.exe -w entry-static.exe dirname",
        "runtest.exe -w entry-static.exe fnmatch",
        "runtest.exe -w entry-static.exe iconv_open",
        "runtest.exe -w entry-static.exe inet_pton",
        "runtest.exe -w entry-static.exe mbc",
        "runtest.exe -w entry-static.exe random",
        "runtest.exe -w entry-static.exe search_insque",
        "runtest.exe -w entry-static.exe search_lsearch",
        "runtest.exe -w entry-static.exe search_tsearch",
        "runtest.exe -w entry-static.exe string",
        "runtest.exe -w entry-static.exe string_memcpy",
        "runtest.exe -w entry-static.exe string_memmem",
        "runtest.exe -w entry-static.exe string_memset",
        "runtest.exe -w entry-static.exe string_strchr",
        "runtest.exe -w entry-static.exe string_strcspn",
        "runtest.exe -w entry-static.exe string_strstr",
        "runtest.exe -w entry-static.exe strtol",
        "runtest.exe -w entry-static.exe time",
        "runtest.exe -w entry-static.exe tls_align",
        "runtest.exe -w entry-static.exe udiv",
        "runtest.exe -w entry-static.exe wcsstr",
        "runtest.exe -w entry-static.exe wcstol",
        "runtest.exe -w entry-static.exe pleval",
        "runtest.exe -w entry-static.exe dn_expand_empty",
        "runtest.exe -w entry-static.exe dn_expand_ptr_0",
        "runtest.exe -w entry-static.exe fgets_eof",
        "runtest.exe -w entry-static.exe fgetwc_buffering",
        "runtest.exe -w entry-static.exe fpclassify_invalid_ld80",
        "runtest.exe -w entry-static.exe getpwnam_r_crash",
        "runtest.exe -w entry-static.exe getpwnam_r_errno",
        "runtest.exe -w entry-static.exe iconv_roundtrips",
        "runtest.exe -w entry-static.exe inet_ntop_v4mapped",
        "runtest.exe -w entry-static.exe inet_pton_empty_last_field",
        "runtest.exe -w entry-static.exe iswspace_null",
        "runtest.exe -w entry-static.exe lrand48_signextend",
        "runtest.exe -w entry-static.exe malloc_0",
        "runtest.exe -w entry-static.exe mbsrtowcs_overflow",
        "runtest.exe -w entry-static.exe memmem_oob_read",
        "runtest.exe -w entry-static.exe memmem_oob",
        "runtest.exe -w entry-static.exe mkdtemp_failure",
        "runtest.exe -w entry-static.exe mkstemp_failure",
        "runtest.exe -w entry-static.exe printf_fmt_n",
        "runtest.exe -w entry-static.exe regex_escaped_high_byte",
        "runtest.exe -w entry-static.exe regexec_nosub",
        "runtest.exe -w entry-static.exe scanf_bytes_consumed",
        "runtest.exe -w entry-static.exe scanf_match_literal_eof",
        "runtest.exe -w entry-static.exe scanf_nullbyte_char",
        "runtest.exe -w entry-static.exe setvbuf_unget",
        "runtest.exe -w entry-static.exe sigprocmask_internal",
        "runtest.exe -w entry-static.exe strverscmp",
        "runtest.exe -w entry-static.exe uselocale_0",
        "runtest.exe -w entry-static.exe wcsncpy_read_overflow",
        "runtest.exe -w entry-static.exe wcsstr_false_negative",
        "runtest.exe -w entry-static.exe syscall_sign_extend",
        "runtest.exe -w entry-static.exe statvfs",
        "runtest.exe -w entry-static.exe rlimit_open_files",
        "runtest.exe -w entry-static.exe rewind_clear_error",
        "runtest.exe -w entry-static.exe ungetc",
        "runtest.exe -w entry-static.exe stat",
        "runtest.exe -w entry-static.exe utime",
        "runtest.exe -w entry-static.exe fscanf",
        "runtest.exe -w entry-static.exe fwscanf",
        "runtest.exe -w entry-static.exe snprintf",
        "runtest.exe -w entry-static.exe sscanf",
        "runtest.exe -w entry-static.exe strtod",
        "runtest.exe -w entry-static.exe strtod_simple",
        "runtest.exe -w entry-static.exe strtof",
        "runtest.exe -w entry-static.exe strtold",
        "runtest.exe -w entry-static.exe swprintf",
        "runtest.exe -w entry-static.exe tgmath",
        "runtest.exe -w entry-static.exe printf_1e9_oob",
        "runtest.exe -w entry-static.exe printf_fmt_g_round",
        "runtest.exe -w entry-static.exe printf_fmt_g_zeros",
        "runtest.exe -w entry-static.exe sscanf_eof",
        "runtest.exe -w entry-static.exe lseek_large",
        "runtest.exe -w entry-static.exe setjmp",
        "runtest.exe -w entry-static.exe daemon_failure",
        "runtest.exe -w entry-static.exe fflush_exit",
        "runtest.exe -w entry-static.exe putenv_doublefree",
        "runtest.exe -w entry-static.exe strftime",
        "runtest.exe -w entry-static.exe search_hsearch",
        // 扩大栈可过
        "runtest.exe -w entry-static.exe qsort",
        "runtest.exe -w entry-static.exe fdopen",
        "runtest.exe -w entry-static.exe ftello_unflushed_append",

        // 可能出现exception
        "runtest.exe -w entry-static.exe memstream",
        "runtest.exe -w entry-static.exe regex_backref_0",
        "runtest.exe -w entry-static.exe regex_bracket_icase",
        "runtest.exe -w entry-static.exe regex_ere_backref",
        "runtest.exe -w entry-static.exe regex_negated_range",

        // 比较耗时的
        "runtest.exe -w entry-static.exe clocale_mbfuncs",
        "runtest.exe -w entry-static.exe crypt",

        // 刚完成 未测试
        "runtest.exe -w entry-static.exe pthread_tsd",
        "runtest.exe -w entry-static.exe pthread_robust_detach",
        "runtest.exe -w entry-static.exe pthread_condattr_setclock",
        "runtest.exe -w entry-static.exe pthread_rwlock_ebusy",
        "runtest.exe -w entry-static.exe pthread_exit_cancel",
        "runtest.exe -w entry-static.exe pthread_cond",
        "runtest.exe -w entry-static.exe strptime",
        "runtest.exe -w entry-static.exe pthread_cancel",
        "runtest.exe -w entry-static.exe pthread_cancel_sem_wait",
        "runtest.exe -w entry-static.exe pthread_once_deadlock",
        "runtest.exe -w entry-static.exe pthread_cancel_points",
        "runtest.exe -w entry-static.exe sscanf_long",
        "runtest.exe -w entry-static.exe socket",




        // dynamic
        "runtest.exe -w entry-dynamic.exe argv",
        "runtest.exe -w entry-dynamic.exe env",
        "runtest.exe -w entry-dynamic.exe basename",
        "runtest.exe -w entry-dynamic.exe clock_gettime",
        "runtest.exe -w entry-dynamic.exe dirname",
        "runtest.exe -w entry-dynamic.exe fnmatch",
        "runtest.exe -w entry-dynamic.exe inet_pton",
        "runtest.exe -w entry-dynamic.exe mbc",
        "runtest.exe -w entry-dynamic.exe random",
        "runtest.exe -w entry-dynamic.exe search_insque",
        "runtest.exe -w entry-dynamic.exe search_lsearch",
        "runtest.exe -w entry-dynamic.exe search_tsearch",
        "runtest.exe -w entry-dynamic.exe string",
        "runtest.exe -w entry-dynamic.exe string_memcpy",
        "runtest.exe -w entry-dynamic.exe string_memmem",
        "runtest.exe -w entry-dynamic.exe string_memset",
        "runtest.exe -w entry-dynamic.exe string_strchr",
        "runtest.exe -w entry-dynamic.exe string_strcspn",
        "runtest.exe -w entry-dynamic.exe string_strstr",
        "runtest.exe -w entry-dynamic.exe strtol",
        "runtest.exe -w entry-dynamic.exe time",
        "runtest.exe -w entry-dynamic.exe udiv",
        "runtest.exe -w entry-dynamic.exe wcsstr",
        "runtest.exe -w entry-dynamic.exe wcstol",
        "runtest.exe -w entry-dynamic.exe dn_expand_empty",
        "runtest.exe -w entry-dynamic.exe dn_expand_ptr_0",
        "runtest.exe -w entry-dynamic.exe fgets_eof",
        "runtest.exe -w entry-dynamic.exe fgetwc_buffering",
        "runtest.exe -w entry-dynamic.exe getpwnam_r_errno",
        "runtest.exe -w entry-dynamic.exe iconv_roundtrips",
        "runtest.exe -w entry-dynamic.exe inet_ntop_v4mapped",
        "runtest.exe -w entry-dynamic.exe inet_pton_empty_last_field",
        "runtest.exe -w entry-dynamic.exe iswspace_null",
        "runtest.exe -w entry-dynamic.exe lrand48_signextend",
        "runtest.exe -w entry-dynamic.exe malloc_0",
        "runtest.exe -w entry-dynamic.exe mbsrtowcs_overflow",
        "runtest.exe -w entry-dynamic.exe memmem_oob_read",
        "runtest.exe -w entry-dynamic.exe memmem_oob",
        "runtest.exe -w entry-dynamic.exe mkdtemp_failure",
        "runtest.exe -w entry-dynamic.exe mkstemp_failure",
        "runtest.exe -w entry-dynamic.exe printf_fmt_n",
        "runtest.exe -w entry-dynamic.exe regex_escaped_high_byte",
        "runtest.exe -w entry-dynamic.exe regexec_nosub",
        "runtest.exe -w entry-dynamic.exe scanf_bytes_consumed",
        "runtest.exe -w entry-dynamic.exe scanf_match_literal_eof",
        "runtest.exe -w entry-dynamic.exe scanf_nullbyte_char",
        "runtest.exe -w entry-dynamic.exe setvbuf_unget",
        "runtest.exe -w entry-dynamic.exe sigprocmask_internal",
        "runtest.exe -w entry-dynamic.exe strverscmp",
        "runtest.exe -w entry-dynamic.exe uselocale_0",
        "runtest.exe -w entry-dynamic.exe wcsncpy_read_overflow",
        "runtest.exe -w entry-dynamic.exe wcsstr_false_negative",
        "runtest.exe -w entry-dynamic.exe stat",
        "runtest.exe -w entry-dynamic.exe ungetc",
        "runtest.exe -w entry-dynamic.exe rewind_clear_error",
        "runtest.exe -w entry-dynamic.exe rlimit_open_files",
        "runtest.exe -w entry-dynamic.exe statvfs",
        "runtest.exe -w entry-dynamic.exe syscall_sign_extend",
        "runtest.exe -w entry-dynamic.exe utime",
        "runtest.exe -w entry-dynamic.exe fscanf",
        "runtest.exe -w entry-dynamic.exe fwscanf",
        "runtest.exe -w entry-dynamic.exe snprintf",
        "runtest.exe -w entry-dynamic.exe sscanf",
        "runtest.exe -w entry-dynamic.exe strtod",
        "runtest.exe -w entry-dynamic.exe strtod_simple",
        "runtest.exe -w entry-dynamic.exe strtof",
        "runtest.exe -w entry-dynamic.exe strtold",
        "runtest.exe -w entry-dynamic.exe swprintf",
        "runtest.exe -w entry-dynamic.exe tgmath",
        "runtest.exe -w entry-dynamic.exe printf_1e9_oob",
        "runtest.exe -w entry-dynamic.exe printf_fmt_g_round",
        "runtest.exe -w entry-dynamic.exe printf_fmt_g_zeros",
        "runtest.exe -w entry-dynamic.exe sscanf_eof",
        "runtest.exe -w entry-dynamic.exe lseek_large",
        "runtest.exe -w entry-dynamic.exe setjmp",
        "runtest.exe -w entry-dynamic.exe daemon_failure",
        "runtest.exe -w entry-dynamic.exe fflush_exit",
        "runtest.exe -w entry-dynamic.exe putenv_doublefree",
        "runtest.exe -w entry-dynamic.exe strftime",
        "runtest.exe -w entry-dynamic.exe search_hsearch",

        // 扩大栈可过
        "runtest.exe -w entry-dynamic.exe qsort",

        // 申请临时内存作为虚拟文件
        "runtest.exe -w entry-dynamic.exe fdopen",
        "runtest.exe -w entry-dynamic.exe iconv_open",
        "runtest.exe -w entry-dynamic.exe fpclassify_invalid_ld80",
        "runtest.exe -w entry-dynamic.exe getpwnam_r_crash",
        "runtest.exe -w entry-dynamic.exe ftello_unflushed_append",

        // 可能出现 Exception(StoreMisaligned)   k210 error
        "runtest.exe -w entry-dynamic.exe memstream",
        "runtest.exe -w entry-dynamic.exe regex_backref_0",
        "runtest.exe -w entry-dynamic.exe regex_bracket_icase",
        "runtest.exe -w entry-dynamic.exe regex_ere_backref",
        "runtest.exe -w entry-dynamic.exe regex_negated_range",

        // 刚刚完成 未测试的
        "runtest.exe -w entry-dynamic.exe pthread_tsd",
        "runtest.exe -w entry-dynamic.exe pthread_condattr_setclock",
        "runtest.exe -w entry-dynamic.exe pthread_robust_detach",
        "runtest.exe -w entry-dynamic.exe pthread_once_deadlock",        // error?
        "runtest.exe -w entry-dynamic.exe pthread_rwlock_ebusy",
        "runtest.exe -w entry-dynamic.exe pthread_exit_cancel",
        "runtest.exe -w entry-dynamic.exe strptime",
        "runtest.exe -w entry-dynamic.exe dlopen",
        "runtest.exe -w entry-dynamic.exe pthread_cancel",
        "runtest.exe -w entry-dynamic.exe pthread_cond",
        "runtest.exe -w entry-dynamic.exe sem_init",
        "runtest.exe -w entry-dynamic.exe tls_get_new_dtv",
        "runtest.exe -w entry-dynamic.exe pthread_cancel_points",
        "runtest.exe -w entry-dynamic.exe sscanf_long",
        "runtest.exe -w entry-dynamic.exe socket",


        // 不稳定的
        "runtest.exe -w entry-dynamic.exe tls_local_exec",
        "runtest.exe -w entry-dynamic.exe tls_init",

        // 比较耗时的
        "runtest.exe -w entry-dynamic.exe clocale_mbfuncs",
        "runtest.exe -w entry-dynamic.exe crypt",
        
        // 莫名奇妙过得
        "runtest.exe -w entry-static.exe pthread_cond_smasher",
        "runtest.exe -w entry-dynamic.exe pthread_cond_smasher",

        // 待完成功能

        // "runtest.exe -w entry-dynamic.exe pthread_cancel_sem_wait",

        // "runtest.exe -w entry-dynamic.exe tls_align",
        // "runtest.exe -w entry-dynamic.exe pleval",


    ]));
}


pub fn exec_by_str(str: &str) {
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