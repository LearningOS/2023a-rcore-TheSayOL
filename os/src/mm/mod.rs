//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

use address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{kernel_token, MapPermission, MemorySet, KERNEL_SPACE};
use page_table::PTEFlags;
pub use page_table::{
    translated_byte_buffer, translated_ref, translated_refmut, translated_str, PageTable,
    PageTableEntry, UserBuffer, UserBufferIterator,
};

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}



/// is_this_va_used_by_current
pub fn is_this_va_used_by_current(va: VirtAddr) -> bool {
    use crate::task::current_user_token;
    let pte = PageTable::from_token(current_user_token()).translate(va.floor());
    pte.is_some() && pte.unwrap().is_valid()
}

/// into mem_area to current task's mem_set 
pub fn current_insert_area(start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) {
    use crate::task::current_task;
    let tcb = current_task().unwrap();
    let mut tcb_inner = tcb.inner_exclusive_access();
    tcb_inner.memory_set.insert_framed_area(start_va, end_va, permission);
}

/// shrink mem_area of current task's mem_set
/// start_va: must be an area's start
/// end_va: will be this area's new_start 
pub fn current_shrink_area(start_va: VirtAddr, end_va: VirtAddr){
    use crate::task::current_task;
    let tcb = current_task().unwrap();
    let mut tcb_inner = tcb.inner_exclusive_access();
    tcb_inner.memory_set.shrink_from(start_va, end_va);
} 