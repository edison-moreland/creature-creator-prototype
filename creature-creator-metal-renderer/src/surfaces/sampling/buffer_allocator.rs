// A BufferAllocator is responsible for allocating indices in a large static buffer
pub trait BufferAllocator<const SIZE: usize> {
    // get the next free index
    fn insert(&mut self) -> usize;

    // return an index to the allocator
    fn remove(&mut self, index: usize);
}

// uses a simple stack based method for tracking free indices
pub struct StackBufferAllocator<const SIZE: usize> {
    buffer_head: usize,
    // The highest index given out
    returned_indices: Vec<usize>,
}

impl<const SIZE: usize> StackBufferAllocator<SIZE> {
    pub fn new() -> Self {
        StackBufferAllocator {
            buffer_head: 0,
            returned_indices: vec![],
        }
    }

    fn compact(&mut self) {
        self.reduce_head();

        // We need to identify the contiguous occupied ranges in the stack
        // let mut contiguous_ranges = vec![];
        // for i in self.returned_indices.split_off() {
        //
        // }
    }

    fn reduce_head(&mut self) {
        // If possible, reduce the size of the stack by reducing the head

        // TODO: this doesn't have to look at the whole stack
        let mut partition_point = None;
        for i in (0..self.returned_indices.len()).rev() {
            if self.returned_indices[i] == (self.buffer_head - 1) {
                partition_point = Some(i);
                self.buffer_head -= 1;
            }
        }

        if let Some(i) = partition_point {
            // The head has already been reduced, we just need to cut the old indices off the edge
            assert_eq!(self.buffer_head, self.returned_indices[i]);
            self.returned_indices.drain(0..(i + 1));
        }
    }
}

impl<const SIZE: usize> BufferAllocator<SIZE> for StackBufferAllocator<SIZE> {
    fn insert(&mut self) -> usize {
        // dbg!(self.buffer_head, &self.returned_indices);

        match self.returned_indices.pop() {
            Some(i) => i,
            None => {
                let i = self.buffer_head;
                self.buffer_head += 1;

                assert!(self.buffer_head < SIZE);

                i
            }
        }
    }

    fn remove(&mut self, index: usize) {
        // Use partition point to make sure returned_indices stays sorted
        let idx = self.returned_indices.partition_point(|&x| x > index);

        if idx == (self.buffer_head - 1) {
            self.buffer_head -= 1;
        } else {
            self.returned_indices.insert(idx, index);
        }

        self.compact();
    }
}
