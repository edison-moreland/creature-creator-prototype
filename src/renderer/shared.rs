use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::addr_of;

use metal::{Buffer, DeviceRef, MTLResourceOptions};

// Shared allows any sized T to be shared between the cpu and gpu
pub struct Shared<T: Sized> {
    value: Pin<Box<T>>,
    buffer: Buffer,
}

impl<T: Sized> Shared<T> {
    pub fn new(device: &DeviceRef, v: T) -> Self {
        let value = Box::pin(v);
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
