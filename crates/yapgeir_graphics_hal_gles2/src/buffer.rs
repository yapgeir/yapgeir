use glow::HasContext;
use yapgeir_graphics_hal::{
    buffer::{BufferData, BufferKind, BufferUsage, ByteBuffer},
    WindowBackend,
};

use crate::{constants::GlConstant, Gles};

pub struct GlesBuffer<B: WindowBackend> {
    pub ctx: Gles<B>,
    pub len: usize,
    pub kind: BufferKind,
    pub buffer: glow::Buffer,
}

impl<B: WindowBackend> ByteBuffer<Gles<B>> for GlesBuffer<B> {
    type Usage = BufferUsage;

    fn new<'a>(
        ctx: Gles<B>,
        kind: BufferKind,
        usage: Self::Usage,
        data: BufferData<'a, u8>,
    ) -> Self {
        let len = data.len();

        let buffer = unsafe {
            let mut ctx = ctx.get_ref();
            let buffer = ctx.gl.create_buffer().expect("Unable to create buffer.");

            ctx.bind_buffer(kind, Some(buffer));

            let kind = kind.gl_const();
            let usage = usage.gl_const();

            match data {
                BufferData::Data(data) => {
                    let data = bytemuck::cast_slice(data);
                    ctx.gl.buffer_data_u8_slice(kind, data, usage);
                }
                BufferData::Empty(len) => {
                    let len = len;
                    ctx.gl.buffer_data_size(kind, len as i32, usage);
                }
            }

            buffer
        };

        Self {
            ctx,
            len,
            buffer,
            kind,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn write(&self, offset: usize, data: &[u8]) {
        assert!(
            offset + data.len() <= self.len,
            "attempting to write beyond buffers limit"
        );

        let mut ctx = self.ctx.get_ref();
        ctx.bind_vertex_array(None);
        ctx.bind_buffer(self.kind, Some(self.buffer));

        unsafe {
            ctx.gl
                .buffer_sub_data_u8_slice(self.kind.gl_const(), offset as i32, data)
        };
    }
}

impl<B: WindowBackend> Drop for GlesBuffer<B> {
    fn drop(&mut self) {
        unsafe {
            let mut ctx = self.ctx.get_ref();
            if ctx.state.bound_buffers[self.kind] == Some(self.buffer) {
                ctx.bind_buffer(self.kind, None);
            }
            ctx.gl.delete_buffer(self.buffer);
        }
    }
}
