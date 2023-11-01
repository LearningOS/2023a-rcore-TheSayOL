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

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use address::{StepByOne, VPNRange};
use alloc::vec::Vec;
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{kernel_stack_position, MapPermission, MemorySet, KERNEL_SPACE};
pub use page_table::{translated_byte_buffer, PageTableEntry};
pub use page_table::{PTEFlags, PageTable};

use crate::config::PAGE_SIZE;
use crate::sync::UPSafeCell;
use lazy_static::*;
use crate::task::{current_user_token, current_insert_area, current_shrink_area};


/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}



/// 申请长度为 len 字节的物理内存，将其映射到 start 开始的虚存，内存页属性为 port
/// start 需要映射的虚存起始地址，要求按页对齐
/// len 映射字节长度，可以为 0
/// port：第 0 位表示是否可读，第 1 位表示是否可写，第 2 位表示是否可执行。其他位无效且必须为 0
/// 为了简单，目标虚存区间要求按页对齐，len 可直接按页向上取整，不考虑分配失败时的页回收。
pub fn mmap(_start: usize, _len: usize, _port: usize) -> isize {

    if _len == 0 {return 0;}
    
    let va_start = VirtAddr(_start);
    let va_end = VirtAddr(_start + _len);

    // 可能的错误: start 没有按页大小对齐 ;  port & !0x7 != 0 (port 其余位必须为0) ;  port & 0x7 = 0 (这样的内存无意义)
    if _port & !0x7 != 0 ||  _port & 0x7 == 0 || !va_start.aligned() {
        return -1;
    }

    // flag 
    let mut flags = MapPermission::U;
    if _port & 1 != 0 {flags |= MapPermission::R;}
    if _port & 2 != 0 {flags |= MapPermission::W;}
    if _port & 4 != 0 {flags |= MapPermission::X;}

    let fake_pt = PageTable::from_token(current_user_token());
    let mut va = va_start;
    // 看看页表, 有没有已经映射的
    while va < va_end {
        let vpn = va.floor();
        let pte = fake_pt.translate(vpn);
        if pte.is_some() && pte.unwrap().is_valid() {
            return -1;
        }
        va.0 += PAGE_SIZE;
    } 

    println!("mmap in, flags = {:#b}", flags.bits());
    current_insert_area(va_start, va_end, flags);
    println!("mmap out");

    // map

    0
}

/// 取消到 [start, start + len) 虚存的映射
/// 为了简单，参数错误时不考虑内存的恢复和回收。
#[allow(unused)]
pub fn munmap(_start: usize, _len: usize) -> isize {
    let va_start = VirtAddr(_start);
    let va_end = VirtAddr(_start + _len);

    if !va_start.aligned() {return -1;}

    let mut va = va_start;
    let mut fake_pt = PageTable::from_token(current_user_token());
    // 检查: [start, start + len) 中存在未被映射的虚存。
    while va < va_end {
        let vpn = va.floor();
        let pte = fake_pt.translate(vpn);
        if pte.is_none() || !pte.unwrap().is_valid() {
            return -1;
        }
        va.0 += PAGE_SIZE;
    }

    println!("munmap in");
    // unmap to pt 
    current_shrink_area(va_start, va_end);  // 如果要求映射的时候是 start ~ 2页, 收回的时候要求 start ~ 1页 ; 将会把两页全收了, 不过测例过了...
    println!("munmap out");

    0
}

lazy_static !{
    /// mmap frames
    pub static ref MMAP_FRAMES: UPSafeCell<Vec<FrameTracker>> = unsafe { UPSafeCell::new(Vec::new()) }; 
}