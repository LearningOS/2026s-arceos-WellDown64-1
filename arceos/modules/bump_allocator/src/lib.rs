#![no_std]

use core::sync::atomic::AtomicUsize;

use allocator::{AllocError, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    addr_start: usize,
    addr_end: usize,
    b_pos: usize,
    p_pos: usize,
    alloced: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            addr_start: 0,
            addr_end: 0,
            b_pos: 0,
            p_pos: 0,
            alloced: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.addr_start = start;
        self.addr_end = start + size;
        self.b_pos = self.addr_start;
        self.p_pos = self.addr_end;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> allocator::AllocResult {
        Ok(())
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let align = layout.align();
        let aligned_b_pos = (self.b_pos + align - 1) & !(align - 1);
        let next_b_pos = aligned_b_pos + layout.size();
        if next_b_pos > self.p_pos {
            return Err(AllocError::NoMemory);
        }
        self.b_pos = next_b_pos;
        self.alloced += layout.size();
        Ok(core::ptr::NonNull::new(next_b_pos as *mut u8).expect("next bpos cannot be 0"))
    }

    fn dealloc(&mut self, _pos: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {}

    fn total_bytes(&self) -> usize {
        self.addr_end - self.addr_start
    }

    fn used_bytes(&self) -> usize {
        self.alloced
    }

    fn available_bytes(&self) -> usize {
        self.addr_end - self.addr_start - self.alloced
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        let aligned_p_pos = self.p_pos & !((1 << align_pow2) - 1);
        let p_pos_next = aligned_p_pos - num_pages * Self::PAGE_SIZE;
        if p_pos_next < self.b_pos {
            return Err(AllocError::NoMemory);
        }
        self.p_pos = p_pos_next;
        self.alloced += num_pages * Self::PAGE_SIZE;
        Ok(p_pos_next)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {}

    fn total_pages(&self) -> usize {
        (self.addr_end - self.addr_start) / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.addr_end - self.p_pos) / Self::PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / Self::PAGE_SIZE
    }
}
