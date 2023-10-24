// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// the following definition come from Linux (kernel 6.3.3) source file:
// 'arch/x86/entry/syscalls/syscall_64.tbl'
//
// for other arch such as aarch64 and newer arch such as riscv64, checkout
// 'include/uapi/asm-generic/unistd.h' and
// 'arch/{riscv}/include/uapi/asm/unistd.h'
//
// ref:
// - https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md
// - https://man7.org/linux/man-pages/man2/syscall.2.html

#[repr(usize)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum SysCallNum {
    read = 0,                     //	common				sys_read
    write = 1,                    //	common				sys_write
    open = 2,                     //	common				sys_open
    close = 3,                    //	common				sys_close
    stat = 4,                     //	common				sys_newstat
    fstat = 5,                    //	common				sys_newfstat
    lstat = 6,                    //	common				sys_newlstat
    poll = 7,                     //	common				sys_poll
    lseek = 8,                    //	common				sys_lseek
    mmap = 9,                     //	common				sys_mmap
    mprotect = 10,                //	common			sys_mprotect
    munmap = 11,                  //	common				sys_munmap
    brk = 12,                     //	common				sys_brk
    rt_sigaction = 13,            //	64			sys_rt_sigaction
    rt_sigprocmask = 14,          //	common			sys_rt_sigprocmask
    rt_sigreturn = 15,            //	64			sys_rt_sigreturn
    ioctl = 16,                   //	64				sys_ioctl
    pread64 = 17,                 //	common				sys_pread64
    pwrite64 = 18,                //	common			sys_pwrite64
    readv = 19,                   //	64				sys_readv
    writev = 20,                  //	64				sys_writev
    access = 21,                  //	common				sys_access
    pipe = 22,                    //	common				sys_pipe
    select = 23,                  //	common				sys_select
    sched_yield = 24,             //	common			sys_sched_yield
    mremap = 25,                  //	common				sys_mremap
    msync = 26,                   //	common				sys_msync
    mincore = 27,                 //	common				sys_mincore
    madvise = 28,                 //	common				sys_madvise
    shmget = 29,                  //	common				sys_shmget
    shmat = 30,                   //	common				sys_shmat
    shmctl = 31,                  //	common				sys_shmctl
    dup = 32,                     //	common				sys_dup
    dup2 = 33,                    //	common				sys_dup2
    pause = 34,                   //	common				sys_pause
    nanosleep = 35,               //	common			sys_nanosleep
    getitimer = 36,               //	common			sys_getitimer
    alarm = 37,                   //	common				sys_alarm
    setitimer = 38,               //	common			sys_setitimer
    getpid = 39,                  //	common				sys_getpid
    sendfile = 40,                //	common			sys_sendfile64
    socket = 41,                  //	common				sys_socket
    connect = 42,                 //	common				sys_connect
    accept = 43,                  //	common				sys_accept
    sendto = 44,                  //	common				sys_sendto
    recvfrom = 45,                //	64			sys_recvfrom
    sendmsg = 46,                 //	64				sys_sendmsg
    recvmsg = 47,                 //	64				sys_recvmsg
    shutdown = 48,                //	common			sys_shutdown
    bind = 49,                    //	common				sys_bind
    listen = 50,                  //	common				sys_listen
    getsockname = 51,             //	common			sys_getsockname
    getpeername = 52,             //	common			sys_getpeername
    socketpair = 53,              //	common			sys_socketpair
    setsockopt = 54,              //	64			sys_setsockopt
    getsockopt = 55,              //	64			sys_getsockopt
    clone = 56,                   //	common				sys_clone
    fork = 57,                    //	common				sys_fork
    vfork = 58,                   //	common				sys_vfork
    execve = 59,                  //	64				sys_execve
    exit = 60,                    //	common				sys_exit
    wait4 = 61,                   //	common				sys_wait4
    kill = 62,                    //	common				sys_kill
    uname = 63,                   //	common				sys_newuname
    semget = 64,                  //	common				sys_semget
    semop = 65,                   //	common				sys_semop
    semctl = 66,                  //	common				sys_semctl
    shmdt = 67,                   //	common				sys_shmdt
    msgget = 68,                  //	common				sys_msgget
    msgsnd = 69,                  //	common				sys_msgsnd
    msgrcv = 70,                  //	common				sys_msgrcv
    msgctl = 71,                  //	common				sys_msgctl
    fcntl = 72,                   //	common				sys_fcntl
    flock = 73,                   //	common				sys_flock
    fsync = 74,                   //	common				sys_fsync
    fdatasync = 75,               //	common			sys_fdatasync
    truncate = 76,                //	common			sys_truncate
    ftruncate = 77,               //	common			sys_ftruncate
    getdents = 78,                //	common			sys_getdents
    getcwd = 79,                  //	common				sys_getcwd
    chdir = 80,                   //	common				sys_chdir
    fchdir = 81,                  //	common				sys_fchdir
    rename = 82,                  //	common				sys_rename
    mkdir = 83,                   //	common				sys_mkdir
    rmdir = 84,                   //	common				sys_rmdir
    creat = 85,                   //	common				sys_creat
    link = 86,                    //	common				sys_link
    unlink = 87,                  //	common				sys_unlink
    symlink = 88,                 //	common				sys_symlink
    readlink = 89,                //	common			sys_readlink
    chmod = 90,                   //	common				sys_chmod
    fchmod = 91,                  //	common				sys_fchmod
    chown = 92,                   //	common				sys_chown
    fchown = 93,                  //	common				sys_fchown
    lchown = 94,                  //	common				sys_lchown
    umask = 95,                   //	common				sys_umask
    gettimeofday = 96,            //	common			sys_gettimeofday
    getrlimit = 97,               //	common			sys_getrlimit
    getrusage = 98,               //	common			sys_getrusage
    sysinfo = 99,                 //	common				sys_sysinfo
    times = 100,                  //	common				sys_times
    ptrace = 101,                 //	64				sys_ptrace
    getuid = 102,                 //	common				sys_getuid
    syslog = 103,                 //	common				sys_syslog
    getgid = 104,                 //	common				sys_getgid
    setuid = 105,                 //	common				sys_setuid
    setgid = 106,                 //	common				sys_setgid
    geteuid = 107,                //	common				sys_geteuid
    getegid = 108,                //	common				sys_getegid
    setpgid = 109,                //	common				sys_setpgid
    getppid = 110,                //	common				sys_getppid
    getpgrp = 111,                //	common				sys_getpgrp
    setsid = 112,                 //	common				sys_setsid
    setreuid = 113,               //	common			sys_setreuid
    setregid = 114,               //	common			sys_setregid
    getgroups = 115,              //	common			sys_getgroups
    setgroups = 116,              //	common			sys_setgroups
    setresuid = 117,              //	common			sys_setresuid
    getresuid = 118,              //	common			sys_getresuid
    setresgid = 119,              //	common			sys_setresgid
    getresgid = 120,              //	common			sys_getresgid
    getpgid = 121,                //	common				sys_getpgid
    setfsuid = 122,               //	common			sys_setfsuid
    setfsgid = 123,               //	common			sys_setfsgid
    getsid = 124,                 //	common				sys_getsid
    capget = 125,                 //	common				sys_capget
    capset = 126,                 //	common				sys_capset
    rt_sigpending = 127,          //	64			sys_rt_sigpending
    rt_sigtimedwait = 128,        //	64			sys_rt_sigtimedwait
    rt_sigqueueinfo = 129,        //	64			sys_rt_sigqueueinfo
    rt_sigsuspend = 130,          //	common			sys_rt_sigsuspend
    sigaltstack = 131,            //	64			sys_sigaltstack
    utime = 132,                  //	common				sys_utime
    mknod = 133,                  //	common				sys_mknod
    uselib = 134,                 //	64
    personality = 135,            //	common			sys_personality
    ustat = 136,                  //	common				sys_ustat
    statfs = 137,                 //	common				sys_statfs
    fstatfs = 138,                //	common				sys_fstatfs
    sysfs = 139,                  //	common				sys_sysfs
    getpriority = 140,            //	common			sys_getpriority
    setpriority = 141,            //	common			sys_setpriority
    sched_setparam = 142,         //	common			sys_sched_setparam
    sched_getparam = 143,         //	common			sys_sched_getparam
    sched_setscheduler = 144,     //	common		sys_sched_setscheduler
    sched_getscheduler = 145,     //	common		sys_sched_getscheduler
    sched_get_priority_max = 146, //	common		sys_sched_get_priority_max
    sched_get_priority_min = 147, //	common		sys_sched_get_priority_min
    sched_rr_get_interval = 148,  //	common		sys_sched_rr_get_interval
    mlock = 149,                  //	common				sys_mlock
    munlock = 150,                //	common				sys_munlock
    mlockall = 151,               //	common			sys_mlockall
    munlockall = 152,             //	common			sys_munlockall
    vhangup = 153,                //	common				sys_vhangup
    modify_ldt = 154,             //	common			sys_modify_ldt
    pivot_root = 155,             //	common			sys_pivot_root
    sysctl_ = 156,                //	64				sys_ni_syscall
    prctl = 157,                  //	common				sys_prctl
    arch_prctl = 158,             //	common			sys_arch_prctl
    adjtimex = 159,               //	common			sys_adjtimex
    setrlimit = 160,              //	common			sys_setrlimit
    chroot = 161,                 //	common				sys_chroot
    sync = 162,                   //	common				sys_sync
    acct = 163,                   //	common				sys_acct
    settimeofday = 164,           //	common			sys_settimeofday
    mount = 165,                  //	common				sys_mount
    umount2 = 166,                //	common				sys_umount
    swapon = 167,                 //	common				sys_swapon
    swapoff = 168,                //	common				sys_swapoff
    reboot = 169,                 //	common				sys_reboot
    sethostname = 170,            //	common			sys_sethostname
    setdomainname = 171,          //	common			sys_setdomainname
    iopl = 172,                   //	common				sys_iopl
    ioperm = 173,                 //	common				sys_ioperm
    create_module = 174,          //	64
    init_module = 175,            //	common			sys_init_module
    delete_module = 176,          //	common			sys_delete_module
    get_kernel_syms = 177,        //	64
    query_module = 178,           //	64
    quotactl = 179,               //	common			sys_quotactl
    nfsservctl = 180,             //	64
    getpmsg = 181,                //	common
    putpmsg = 182,                //	common
    afs_syscall = 183,            //	common
    tuxcall = 184,                //	common
    security = 185,               //	common
    gettid = 186,                 //	common				sys_gettid
    readahead = 187,              //	common			sys_readahead
    setxattr = 188,               //	common			sys_setxattr
    lsetxattr = 189,              //	common			sys_lsetxattr
    fsetxattr = 190,              //	common			sys_fsetxattr
    getxattr = 191,               //	common			sys_getxattr
    lgetxattr = 192,              //	common			sys_lgetxattr
    fgetxattr = 193,              //	common			sys_fgetxattr
    listxattr = 194,              //	common			sys_listxattr
    llistxattr = 195,             //	common			sys_llistxattr
    flistxattr = 196,             //	common			sys_flistxattr
    removexattr = 197,            //	common			sys_removexattr
    lremovexattr = 198,           //	common			sys_lremovexattr
    fremovexattr = 199,           //	common			sys_fremovexattr
    tkill = 200,                  //	common				sys_tkill
    time = 201,                   //	common				sys_time
    futex = 202,                  //	common				sys_futex
    sched_setaffinity = 203,      //	common		sys_sched_setaffinity
    sched_getaffinity = 204,      //	common		sys_sched_getaffinity
    set_thread_area = 205,        //	64
    io_setup = 206,               //	64			sys_io_setup
    io_destroy = 207,             //	common			sys_io_destroy
    io_getevents = 208,           //	common			sys_io_getevents
    io_submit = 209,              //	64			sys_io_submit
    io_cancel = 210,              //	common			sys_io_cancel
    get_thread_area = 211,        //	64
    lookup_dcookie = 212,         //	common			sys_lookup_dcookie
    epoll_create = 213,           //	common			sys_epoll_create
    epoll_ctl_old = 214,          //	64
    epoll_wait_old = 215,         //	64
    remap_file_pages = 216,       //	common		sys_remap_file_pages
    getdents64 = 217,             //	common			sys_getdents64
    set_tid_address = 218,        //	common			sys_set_tid_address
    restart_syscall = 219,        //	common			sys_restart_syscall
    semtimedop = 220,             //	common			sys_semtimedop
    fadvise64 = 221,              //	common			sys_fadvise64
    timer_create = 222,           //	64			sys_timer_create
    timer_settime = 223,          //	common			sys_timer_settime
    timer_gettime = 224,          //	common			sys_timer_gettime
    timer_getoverrun = 225,       //	common		sys_timer_getoverrun
    timer_delete = 226,           //	common			sys_timer_delete
    clock_settime = 227,          //	common			sys_clock_settime
    clock_gettime = 228,          //	common			sys_clock_gettime
    clock_getres = 229,           //	common			sys_clock_getres
    clock_nanosleep = 230,        //	common			sys_clock_nanosleep
    exit_group = 231,             //	common			sys_exit_group
    epoll_wait = 232,             //	common			sys_epoll_wait
    epoll_ctl = 233,              //	common			sys_epoll_ctl
    tgkill = 234,                 //	common				sys_tgkill
    utimes = 235,                 //	common				sys_utimes
    vserver = 236,                //	64
    mbind = 237,                  //	common				sys_mbind
    set_mempolicy = 238,          //	common			sys_set_mempolicy
    get_mempolicy = 239,          //	common			sys_get_mempolicy
    mq_open = 240,                //	common				sys_mq_open
    mq_unlink = 241,              //	common			sys_mq_unlink
    mq_timedsend = 242,           //	common			sys_mq_timedsend
    mq_timedreceive = 243,        //	common			sys_mq_timedreceive
    mq_notify = 244,              //	64			sys_mq_notify
    mq_getsetattr = 245,          //	common			sys_mq_getsetattr
    kexec_load = 246,             //	64			sys_kexec_load
    waitid = 247,                 //	64				sys_waitid
    add_key = 248,                //	common				sys_add_key
    request_key = 249,            //	common			sys_request_key
    keyctl = 250,                 //	common				sys_keyctl
    ioprio_set = 251,             //	common			sys_ioprio_set
    ioprio_get = 252,             //	common			sys_ioprio_get
    inotify_init = 253,           //	common			sys_inotify_init
    inotify_add_watch = 254,      //	common		sys_inotify_add_watch
    inotify_rm_watch = 255,       //	common		sys_inotify_rm_watch
    migrate_pages = 256,          //	common			sys_migrate_pages
    openat = 257,                 //	common				sys_openat
    mkdirat = 258,                //	common				sys_mkdirat
    mknodat = 259,                //	common				sys_mknodat
    fchownat = 260,               //	common			sys_fchownat
    futimesat = 261,              //	common			sys_futimesat
    newfstatat = 262,             //	common			sys_newfstatat
    unlinkat = 263,               //	common			sys_unlinkat
    renameat = 264,               //	common			sys_renameat
    linkat = 265,                 //	common				sys_linkat
    symlinkat = 266,              //	common			sys_symlinkat
    readlinkat = 267,             //	common			sys_readlinkat
    fchmodat = 268,               //	common			sys_fchmodat
    faccessat = 269,              //	common			sys_faccessat
    pselect6 = 270,               //	common			sys_pselect6
    ppoll = 271,                  //	common				sys_ppoll
    unshare = 272,                //	common				sys_unshare
    set_robust_list = 273,        //	64			sys_set_robust_list
    get_robust_list = 274,        //	64			sys_get_robust_list
    splice = 275,                 //	common				sys_splice
    tee = 276,                    //	common				sys_tee
    sync_file_range = 277,        //	common			sys_sync_file_range
    vmsplice = 278,               //	64			sys_vmsplice
    move_pages = 279,             //	64			sys_move_pages
    utimensat = 280,              //	common			sys_utimensat
    epoll_pwait = 281,            //	common			sys_epoll_pwait
    signalfd = 282,               //	common			sys_signalfd
    timerfd_create = 283,         //	common			sys_timerfd_create
    eventfd = 284,                //	common				sys_eventfd
    fallocate = 285,              //	common			sys_fallocate
    timerfd_settime = 286,        //	common			sys_timerfd_settime
    timerfd_gettime = 287,        //	common			sys_timerfd_gettime
    accept4 = 288,                //	common				sys_accept4
    signalfd4 = 289,              //	common			sys_signalfd4
    eventfd2 = 290,               //	common			sys_eventfd2
    epoll_create1 = 291,          //	common			sys_epoll_create1
    dup3 = 292,                   //	common				sys_dup3
    pipe2 = 293,                  //	common				sys_pipe2
    inotify_init1 = 294,          //	common			sys_inotify_init1
    preadv = 295,                 //	64				sys_preadv
    pwritev = 296,                //	64				sys_pwritev
    rt_tgsigqueueinfo = 297,      //	64		sys_rt_tgsigqueueinfo
    perf_event_open = 298,        //	common			sys_perf_event_open
    recvmmsg = 299,               //	64			sys_recvmmsg
    fanotify_init = 300,          //	common			sys_fanotify_init
    fanotify_mark = 301,          //	common			sys_fanotify_mark
    prlimit64 = 302,              //	common			sys_prlimit64
    name_to_handle_at = 303,      //	common		sys_name_to_handle_at
    open_by_handle_at = 304,      //	common		sys_open_by_handle_at
    clock_adjtime = 305,          //	common			sys_clock_adjtime
    syncfs = 306,                 //	common				sys_syncfs
    sendmmsg = 307,               //	64			sys_sendmmsg
    setns = 308,                  //	common				sys_setns
    getcpu = 309,                 //	common				sys_getcpu
    process_vm_readv = 310,       //	64		sys_process_vm_readv
    process_vm_writev = 311,      //	64		sys_process_vm_writev
    kcmp = 312,                   //	common				sys_kcmp
    finit_module = 313,           //	common			sys_finit_module
    sched_setattr = 314,          //	common			sys_sched_setattr
    sched_getattr = 315,          //	common			sys_sched_getattr
    renameat2 = 316,              //	common			sys_renameat2
    seccomp = 317,                //	common				sys_seccomp
    getrandom = 318,              //	common			sys_getrandom
    memfd_create = 319,           //	common			sys_memfd_create
    kexec_file_load = 320,        //	common			sys_kexec_file_load
    bpf = 321,                    //	common				sys_bpf
    execveat = 322,               //	64			sys_execveat
    userfaultfd = 323,            //	common			sys_userfaultfd
    membarrier = 324,             //	common			sys_membarrier
    mlock2 = 325,                 //	common				sys_mlock2
    copy_file_range = 326,        //	common			sys_copy_file_range
    preadv2 = 327,                //	64				sys_preadv2
    pwritev2 = 328,               //	64			sys_pwritev2
    pkey_mprotect = 329,          //	common			sys_pkey_mprotect
    pkey_alloc = 330,             //	common			sys_pkey_alloc
    pkey_free = 331,              //	common			sys_pkey_free
    statx = 332,                  //	common				sys_statx
    io_pgetevents = 333,          //	common			sys_io_pgetevents
    rseq = 334,                   //	common				sys_rseq
    // # don't use numbers 387 through 423, add new calls after the last
    // # 'common' entry
    pidfd_send_signal = 424,       //	common		sys_pidfd_send_signal
    io_uring_setup = 425,          //	common			sys_io_uring_setup
    io_uring_enter = 426,          //	common			sys_io_uring_enter
    io_uring_register = 427,       //	common		sys_io_uring_register
    open_tree = 428,               //	common			sys_open_tree
    move_mount = 429,              //	common			sys_move_mount
    fsopen = 430,                  //	common				sys_fsopen
    fsconfig = 431,                //	common			sys_fsconfig
    fsmount = 432,                 //	common				sys_fsmount
    fspick = 433,                  //	common				sys_fspick
    pidfd_open = 434,              //	common			sys_pidfd_open
    clone3 = 435,                  //	common				sys_clone3
    close_range = 436,             //	common			sys_close_range
    openat2 = 437,                 //	common				sys_openat2
    pidfd_getfd = 438,             //	common			sys_pidfd_getfd
    faccessat2 = 439,              //	common			sys_faccessat2
    process_madvise = 440,         //	common			sys_process_madvise
    epoll_pwait2 = 441,            //	common			sys_epoll_pwait2
    mount_setattr = 442,           //	common			sys_mount_setattr
    quotactl_fd = 443,             //	common			sys_quotactl_fd
    landlock_create_ruleset = 444, //	common		sys_landlock_create_ruleset
    landlock_add_rule = 445,       //	common		sys_landlock_add_rule
    landlock_restrict_self = 446,  //	common		sys_landlock_restrict_self
    memfd_secret = 447,            //	common			sys_memfd_secret
    process_mrelease = 448,        //	common		sys_process_mrelease
    futex_waitv = 449,             //	common			sys_futex_waitv
    set_mempolicy_home_node = 450, //	common		sys_set_mempolicy_home_node
}

// # Due to a historical design error, certain syscalls are numbered differently
// # in x32 as compared to native x86_64.  These syscalls have numbers 512-547.
// # Do not add new syscalls to this range.  Numbers 548 and above are available
// # for non-x32 use.
// #
// 512	x32	rt_sigaction		compat_sys_rt_sigaction
// 513	x32	rt_sigreturn		compat_sys_x32_rt_sigreturn
// 514	x32	ioctl			compat_sys_ioctl
// 515	x32	readv			sys_readv
// 516	x32	writev			sys_writev
// 517	x32	recvfrom		compat_sys_recvfrom
// 518	x32	sendmsg			compat_sys_sendmsg
// 519	x32	recvmsg			compat_sys_recvmsg
// 520	x32	execve			compat_sys_execve
// 521	x32	ptrace			compat_sys_ptrace
// 522	x32	rt_sigpending		compat_sys_rt_sigpending
// 523	x32	rt_sigtimedwait		compat_sys_rt_sigtimedwait_time64
// 524	x32	rt_sigqueueinfo		compat_sys_rt_sigqueueinfo
// 525	x32	sigaltstack		compat_sys_sigaltstack
// 526	x32	timer_create		compat_sys_timer_create
// 527	x32	mq_notify		compat_sys_mq_notify
// 528	x32	kexec_load		compat_sys_kexec_load
// 529	x32	waitid			compat_sys_waitid
// 530	x32	set_robust_list		compat_sys_set_robust_list
// 531	x32	get_robust_list		compat_sys_get_robust_list
// 532	x32	vmsplice		sys_vmsplice
// 533	x32	move_pages		sys_move_pages
// 534	x32	preadv			compat_sys_preadv64
// 535	x32	pwritev			compat_sys_pwritev64
// 536	x32	rt_tgsigqueueinfo	compat_sys_rt_tgsigqueueinfo
// 537	x32	recvmmsg		compat_sys_recvmmsg_time64
// 538	x32	sendmmsg		compat_sys_sendmmsg
// 539	x32	process_vm_readv	sys_process_vm_readv
// 540	x32	process_vm_writev	sys_process_vm_writev
// 541	x32	setsockopt		sys_setsockopt
// 542	x32	getsockopt		sys_getsockopt
// 543	x32	io_setup		compat_sys_io_setup
// 544	x32	io_submit		compat_sys_io_submit
// 545	x32	execveat		compat_sys_execveat
// 546	x32	preadv2			compat_sys_preadv64v2
// 547	x32	pwritev2		compat_sys_pwritev64v2
// # This is the end of the legacy x32 range.  Numbers 548 and above are
// # not special and are not to be used for x32-specific syscalls.
