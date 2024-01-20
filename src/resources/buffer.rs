use std::{mem, any::type_name};

use crate::context::Context;

pub struct CpuBuffer<T> {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) data: Vec<T>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> CpuBuffer<T> {
    pub fn with_capacity(context: &Context, capacity: u32, usage: wgpu::BufferUsages) -> Self {
        let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("CpuBuffer<{}>", type_name::<T>())),
            size: capacity as u64 * mem::size_of::<T>() as u64,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            data: Vec::with_capacity(capacity as _),
        }
    }

    pub fn batch<'a>(&'a mut self, context: &'a Context) -> BufferBatch<'a, T> {
        BufferBatch {
            start_index: self.data.len() as u32,
            buffer: self,
            context,
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> u32 {
        self.data.len() as _
    }
}

pub struct BufferBatch<'a, T: bytemuck::Pod + bytemuck::Zeroable> {
    pub(crate) buffer: &'a mut CpuBuffer<T>,
    pub(crate) context: &'a Context<'a>,
    pub(crate) start_index: u32,
}

impl<'a, T: bytemuck::Pod + bytemuck::Zeroable> BufferBatch<'a, T> {
    pub fn push(&mut self, item: T) -> &mut Self {
        self.buffer.data.push(item);
        self
    }
}

impl<'a, T: bytemuck::Pod + bytemuck::Zeroable> Drop for BufferBatch<'a, T> {
    fn drop(&mut self) {
        let offset = self.start_index as u64 * mem::size_of::<T>() as u64;
        self.context.queue.write_buffer(
            &self.buffer.buffer,
            offset,
            bytemuck::cast_slice(&self.buffer.data[self.start_index as usize..]),
        )
    }
}
