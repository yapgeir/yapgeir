use std::{
    alloc::{GlobalAlloc, Layout, System},
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
};

pub struct CountingAllocator {
    counter: AtomicU64,
}

#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator {
    counter: AtomicU64::new(0),
};

pub struct Counter {
    _private: PhantomData<()>,
}

impl Counter {
    pub fn count(&self) -> u64 {
        ALLOC.counter.load(Ordering::Acquire)
        // COUNTER.with(|c| *c.borrow())
    }
}

impl CountingAllocator {
    pub fn counter() -> Counter {
        ALLOC.counter.store(0, Ordering::Release);
        // COUNTER.with(|c| *c.borrow_mut() = 0);
        Counter {
            _private: PhantomData,
        }
    }
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOC.counter.fetch_add(1, Ordering::AcqRel);
        // COUNTER.with(|c| *c.borrow_mut() += 1);
        System.alloc(l)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, l: Layout) {
        System.dealloc(ptr, l);
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOC.counter.fetch_add(1, Ordering::AcqRel);
        // COUNTER.with(|c| *c.borrow_mut() += 1);
        System.alloc_zeroed(layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOC.counter.fetch_add(1, Ordering::AcqRel);
        // COUNTER.with(|c| *c.borrow_mut() += 1);
        System.realloc(ptr, layout, new_size)
    }
}
