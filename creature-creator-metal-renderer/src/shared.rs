use std::alloc::{alloc_zeroed, Layout};
use std::mem;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::addr_of;

use metal::{Buffer, DeviceRef, MTLResourceOptions};

// Used to construct a type on the heap, without involving the stack
// This is to prevent a seg-fault when allocating the huge buffers for the particle system
pub(super) fn new_zeroed_box<T: Sized>() -> Box<T> {
    unsafe {
        // This is safe because the T is sized and we're using the global allocator
        Box::from_raw(mem::transmute(alloc_zeroed(Layout::new::<T>())))
    }
}

// Shared allows any sized T to be shared between the cpu and gpu
pub struct Shared<T: Sized> {
    value: Pin<Box<T>>,
    buffer: Buffer,
}

impl<T: Sized> Shared<T> {
    pub fn new(device: &DeviceRef, value: T) -> Self {
        Self::from_pinned(device, Box::pin(value))
    }

    pub fn new_zeroed(device: &DeviceRef) -> Self {
        Self::from_pinned(device, new_zeroed_box::<T>().into())
    }

    pub fn from_pinned(device: &DeviceRef, value: Pin<Box<T>>) -> Self {
        // TODO: This needs to be page aligned
        let buffer = device.new_buffer_with_bytes_no_copy(
            addr_of!(*value.deref()) as *const _,
            size_of::<T>() as u64,
            MTLResourceOptions::StorageModeShared,
            None,
        );

        Shared { value, buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

impl<T: Sized> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<T: Sized + Unpin> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.deref_mut()
    }
}
