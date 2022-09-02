// 中断调用列表
pub const SYS_GETCWD:usize  = 17;
pub const SYS_DUP: usize    = 23;
pub const SYS_DUP3: usize   = 24;
pub const SYS_FCNTL: usize  = 25;
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
pub const SYS_SENDFILE: usize = 71;
pub const SYS_PPOLL: usize = 73;
pub const SYS_READLINKAT: usize = 78;
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
pub const SYS_TGKILL: usize = 131;
pub const SYS_SIGACTION: usize = 134;
pub const SYS_SIGPROCMASK: usize = 135;
pub const SYS_SIGTIMEDWAIT: usize = 137;
pub const SYS_SIGRETURN: usize = 139;
pub const SYS_TIMES: usize  = 153;
pub const SYS_UNAME: usize  = 160;
pub const SYS_GETRUSAGE: usize = 165;
pub const SYS_GETTIMEOFDAY: usize= 169;
pub const SYS_GETPID:usize  = 172;
pub const SYS_GETPPID:usize = 173;
pub const SYS_GETUID: usize = 174;
// pub const SYS_GETEUID: usize = 175;
pub const SYS_GETGID: usize = 176;
pub const SYS_GETTID: usize = 178;
pub const SYS_SOCKET: usize = 198;
pub const SYS_BIND: usize   = 200;
pub const SYS_LISTEN: usize = 201;
pub const SYS_CONNECT: usize = 203;
pub const SYS_GETSOCKNAME: usize = 204;
pub const SYS_SENDTO: usize = 206;
pub const SYS_RECVFROM: usize = 207;
pub const SYS_SETSOCKOPT: usize = 208;
pub const SYS_BRK:   usize  = 214;
pub const SYS_CLONE: usize  = 220;
pub const SYS_EXECVE:usize  = 221;
pub const SYS_MMAP: usize   = 222;
pub const SYS_MPROTECT:usize= 226;
pub const SYS_MUNMAP:usize  = 215;
pub const SYS_WAIT4: usize  = 260;