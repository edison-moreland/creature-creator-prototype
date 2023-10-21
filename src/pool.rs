// A fixed size memory pool allocator. It doesn't not allocate memory but indices
pub trait PoolAllocator<const SIZE: usize> {
    fn insert(&mut self) -> usize;
    fn remove(&mut self, i: usize);
}

pub struct StackPoolAllocator<const S: usize> {
    returned_indices: Vec<usize>, // We take from this pool first
    pool_end: usize,              // If there are no returned, one is given from the end
}

impl<const S: usize> StackPoolAllocator<S> {
    pub fn new() -> Self {
        StackPoolAllocator {
            returned_indices: vec![],
            pool_end: 0,
        }
    }
}

impl<const S: usize> PoolAllocator<S> for StackPoolAllocator<S> {
    fn insert(&mut self) -> usize {
        match self.returned_indices.pop() {
            Some(i) => i,
            None => {
                if self.pool_end == S {
                    panic!("Pool exceeded!")
                }

                self.pool_end += 1;

                self.pool_end - 1
            }
        }
    }

    fn remove(&mut self, i: usize) {
        self.returned_indices.push(i)
    }
}
