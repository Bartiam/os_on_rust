use super::align_up;
use core::mem;
use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    /// create new LinkedListAllocator.
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// Initialize the allocator with the boundaries of the heap.
    ///
    /// This function is unsafe because the caller must ensure that the received
    /// the boundaries of the heap will be correct and the heap will not be used.
    /// This method must be called once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        unsafe {
            self.add_free_region(heap_start, heap_size);
        }
    }

    /// Adds the received memory to the end of the list.
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // check that the released area can store the ListNode
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        // create a new node and add it to the top of the list
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        unsafe {
            node_ptr.write(node);
            self.head.next = Some(&mut *node_ptr)
        }
    }

    /// We are looking at a free area of a given size and alignment and
    /// removing it from the list.
    ///
    /// Returning the tuple from the list and the initial allocation address.
    fn find_region(&mut self, size: usize, align: usize)
        -> Option<(&'static mut ListNode, usize)>
    {
        // a link to the current node, updated with each iteration
        let mut current = &mut self.head;
        // search for a sufficiently large area of memory in a linked list
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                // the area is suitable for allocation -> remove node from the list
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                // the area does not fit -> go to the next one
                current = current.next.as_mut().unwrap();
            }
        }

        // no suitable area was found
        None
    }

    /// an attempt to use a region to allocate memory of a given size and alignment.
    ///
    /// If successful, it returns the initial address of the allocated memory.
    fn alloc_from_region(region: &ListNode, size: usize, align: usize)
        -> Result<usize, ()>
    {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            // region too small
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            // the rest of the area is too small to store a ListNode.
            // this is necessary because allocation divides the area into used and free
            return Err(());
        }

        // the area is suitable for allocation
        Ok(alloc_start)
    }

    /// Adjust the resulting layout so that the allocated memory area is
    /// it could also store a `ListNode'.
    ///
    /// Returns the adjusted size and alignment as a tuple (size, alignment).
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // adjusting the memory layout
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                unsafe {
                    allocator.add_free_region(alloc_end, excess_size);
                }
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // adjusting the memory layout
        let (size, _) = LinkedListAllocator::size_align(layout);

        unsafe { self.lock().add_free_region(ptr as usize, size) }
    }
}
