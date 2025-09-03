// All user space related code and structures lie above USER_SPACE_START
pub const USER_SPACE_START: usize = 0x10000000000; // 1 TiB

// Code lies at the beginning of the user space (Max size: 1 GiB)
pub const USER_SPACE_CODE_START: usize = USER_SPACE_START;

// User space environment data (Max size: 1 GiB)
pub const USER_SPACE_ENV_START: usize = USER_SPACE_CODE_START + 0x40000000; // 1 GiB
pub const USER_SPACE_ARG_START: usize = USER_SPACE_ENV_START;

// User space stacks (Max size per stack: 1 GiB)
pub const MAX_USER_STACK_SIZE: usize = 0x40000000; // 1 GiB
pub const MAIN_USER_STACK_START: usize = USER_SPACE_ENV_START + 0x40000000; // 1 GiB
pub const KERNEL_STACK_PAGES: usize = 64;
pub const STACK_ENTRY_SIZE: usize = 8;

//===============================================================================================
// update 15.08.2025: increase heap page size (as suggested by M. Schoettner)
// Note: Kernel heap pages are chunks of memory pages allocated to the kernel’s
// dynamic heap, from which the OS allocates memory for its internal data
// structures.
// old value:
// pub const KERNEL_HEAP_PAGES: usize = 0x400; // number of heap pages for booting the OS (old value)
//===============================================================================================
pub const KERNEL_HEAP_PAGES: usize = 0x4000; // number of heap pages for booting the OS
