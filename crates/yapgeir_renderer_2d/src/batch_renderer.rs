use bytemuck::Pod;
use std::{borrow::Borrow, iter::repeat_with, marker::PhantomData, ops::Deref, rc::Rc};
use yapgeir_graphics_hal::{
    buffer::{Buffer, BufferKind, BufferUsage},
    draw_descriptor::{AsVertexBindings, IndexBinding},
    draw_params::DrawParameters,
    frame_buffer::{FrameBuffer, Indices},
    index_buffer::PrimitiveMode,
    samplers::SamplerAttribute,
    uniforms::{UniformBuffer, Uniforms},
    vertex_buffer::Vertex,
    Graphics,
};

use crate::quad_index_buffer::QuadIndexBuffer;

pub static CENTERED_UNIT_RECT: [[f32; 2]; 4] = [[-0.5, -0.5], [-0.5, 0.5], [0.5, 0.5], [0.5, -0.5]];

pub enum BatchIndices<G: Graphics> {
    Quad(QuadIndexBuffer<G>),
    Primitive(PrimitiveMode),
}

impl<'a, G: Graphics> BatchIndices<G> {
    pub fn indices(&self, vertices: usize) -> Indices {
        Indices {
            mode: match self {
                Self::Quad(_) => PrimitiveMode::Triangles,
                Self::Primitive(mode) => *mode,
            },
            offset: 0,
            len: match self {
                Self::Quad(_) => vertices / 4 * 6,
                Self::Primitive(_) => vertices,
            },
        }
    }
}

pub struct Batch<'a, G, V, U, T = <G as Graphics>::Texture, S = [SamplerAttribute<G, T>; 0]>
where
    G: Graphics,
    V: Vertex + Pod,
    U: Uniforms + Pod,
    T: Borrow<G::Texture>,
    S: Borrow<[SamplerAttribute<G, T>]>,
{
    fb: &'a G::FrameBuffer,
    textures: S,
    renderer: &'a mut BatchRenderer<G, V, U>,
    draw_parameters: &'a DrawParameters,

    _t: PhantomData<T>,
}

impl<'a, G, V, U, T, S> Drop for Batch<'a, G, V, U, T, S>
where
    G: Graphics,
    V: Vertex + Pod,
    U: Uniforms + Pod,
    T: Borrow<G::Texture>,
    S: Borrow<[SamplerAttribute<G, T>]>,
{
    fn drop(&mut self) {
        self.flush();
    }
}

impl<'a, G, V, U, T, S> Batch<'a, G, V, U, T, S>
where
    G: Graphics,
    V: Vertex + Pod,
    U: Uniforms + Pod,
    T: Borrow<G::Texture>,
    S: Borrow<[SamplerAttribute<G, T>]>,
{
    pub fn draw(&mut self, vertices: &[V]) {
        assert!(vertices.len() <= self.renderer.unflushed.capacity());

        // If vertices don't fit the unflushed vec, force a draw call to clean it up.
        if vertices.len() > self.renderer.unflushed.capacity() - self.renderer.unflushed.len() {
            self.flush();
        }

        for v in vertices {
            self.renderer.unflushed.push(*v);
        }
    }

    fn flush(&mut self) {
        if self.renderer.unflushed.is_empty() {
            return;
        }

        let current = &self.renderer.vertices[self.renderer.current_buffer];
        current.0.write(0, &self.renderer.unflushed);

        self.fb.draw(
            &current.1,
            &self.draw_parameters,
            self.textures.borrow(),
            Some(&self.renderer.uniform_buffer),
            &self.renderer.indices.indices(self.renderer.unflushed.len()),
        );

        self.renderer.unflushed.clear();
        self.renderer.current_buffer =
            (self.renderer.current_buffer + 1) % self.renderer.vertices.len();
    }
}

pub struct BatchRenderer<G, V, U = ()>
where
    G: Graphics,
    V: Vertex + Pod,
    U: Uniforms + Pod,
{
    uniform_buffer: Rc<G::UniformBuffer<U>>,

    // This will store our vertices until they are flushed to GPU.
    unflushed: Vec<V>,

    // Keep multiple vertex buffers, and use them as a ring buffer
    // moving to the next one when vertices are flushed.
    vertices: Vec<(Buffer<G, V>, G::DrawDescriptor)>,
    current_buffer: usize,

    indices: BatchIndices<G>,
}

impl<G, V, U> BatchRenderer<G, V, U>
where
    G: Graphics,
    V: Vertex + Pod,
    U: Uniforms + Pod,
{
    pub fn new<'a>(
        ctx: &G,
        shader: Rc<G::Shader>,
        indices: BatchIndices<G>,
        uniforms: Rc<G::UniformBuffer<U>>,
        (buffer_size, buffer_count): (usize, usize),
    ) -> Self {
        let mut unflushed = Vec::with_capacity(buffer_size);

        let vertices = repeat_with(|| {
            let vertex = ctx.new_buffer(BufferKind::Vertex, BufferUsage::Stream, buffer_size);

            let descriptor = ctx.new_draw_descriptor(
                shader.clone(),
                match &indices {
                    BatchIndices::Quad(quad) => quad.deref().clone(),
                    BatchIndices::Primitive(_) => IndexBinding::None,
                },
                &[vertex.bindings()],
            );

            (vertex, descriptor)
        })
        .take(buffer_count)
        .collect();

        unflushed.clear();

        Self {
            uniform_buffer: uniforms,

            unflushed,
            vertices,
            current_buffer: 0,

            indices,
        }
    }

    pub fn start_batch<'a, T, S>(
        &'a mut self,
        fb: &'a G::FrameBuffer,
        draw_parameters: &'a DrawParameters,
        uniforms: &U,
        textures: S,
    ) -> Batch<'a, G, V, U, T, S>
    where
        T: Borrow<G::Texture>,
        S: Borrow<[SamplerAttribute<G, T>]>,
    {
        self.uniform_buffer.write(&uniforms);

        Batch {
            fb,
            textures,
            renderer: self,
            draw_parameters,

            _t: PhantomData,
        }
    }
}
