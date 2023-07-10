use std::cell::RefCell;

use bytemuck::Pod;
use yapgeir_graphics_hal::{uniforms::UniformBuffer, WindowBackend};

use crate::Gles;

pub struct GlesUniformBuffer<T> {
    pub(crate) value: RefCell<T>,
}

impl<B: WindowBackend, T: Pod> UniformBuffer<Gles<B>, T> for GlesUniformBuffer<T> {
    fn new(_: Gles<B>, initial: &T) -> Self {
        Self {
            value: RefCell::new(*initial),
        }
    }

    fn write(&self, value: &T) {
        let mut v = self.value.borrow_mut();
        *v = *value;
    }
}
