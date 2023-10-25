// A BufferAllocator is responsible for allocating indices in a large static buffer
pub trait BufferAllocator<const SIZE: usize> {
    // get the next free index
    fn insert(&mut self) -> usize;

    // return an index to the allocator
    fn remove(&mut self, index: usize);
}

// uses a simple stack based method for tracking free indices
pub struct StackBufferAllocator<const SIZE: usize> {
    buffer_head: usize, // The highest index given out
    returned_indices: Vec<usize>,
}

impl<const SIZE: usize> StackBufferAllocator<SIZE> {
    pub fn new() -> Self {
        StackBufferAllocator {
            buffer_head: 0,
            returned_indices: vec![],
        }
    }
}

impl<const SIZE: usize> BufferAllocator<SIZE> for StackBufferAllocator<SIZE> {
    fn insert(&mut self) -> usize {
        match self.returned_indices.pop() {
            Some(i) => i,
            None => {
                let i = self.buffer_head;
                self.buffer_head += 1;

                i
            }
        }
    }

    fn remove(&mut self, index: usize) {
        self.returned_indices.push(index)
    }
}
