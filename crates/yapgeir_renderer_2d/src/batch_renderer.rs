use bytemuck::Pod;
use nalgebra::{point, Point2};
use std::{borrow::Borrow, iter::repeat_with, ops::Deref, rc::Rc};
use yapgeir_graphics_hal::{
    buffer::{Buffer, BufferKind, BufferUsage},
    draw_descriptor::{AsVertexBindings, IndexBinding},
    draw_params::DrawParameters,
    frame_buffer::{FrameBuffer, Indices},
    index_buffer::PrimitiveMode,
    samplers::SamplerAttribute,
    uniforms::Uniforms,
    vertex_buffer::Vertex,
    Graphics,
};

use crate::quad_index_buffer::QuadIndexBuffer;

pub static CENTERED_UNIT_RECT: [Point2<f32>; 4] = [
    point![-0.5, -0.5],
    point![-0.5, 0.5],
    point![0.5, 0.5],
    point![0.5, -0.5],
];

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

pub struct BatchRenderer<
    G: Graphics,
    V: Vertex + Pod,
    U: Uniforms + Pod = (),
    T: Borrow<G::Texture> = <G as Graphics>::Texture,
> {
    pub textures: Vec<SamplerAttribute<G, T>>,
    pub uniforms: Rc<G::UniformBuffer<U>>,
    pub draw_parameters: DrawParameters,

    // This will store our vertices until they are flushed to GPU.
    unflushed: Vec<V>,

    // Keep multiple vertex buffers, and use them as a ring buffer
    // moving to the next one when vertices are flushed.
    vertices: Vec<(Buffer<G, V>, G::DrawDescriptor)>,
    current_buffer: usize,

    indices: BatchIndices<G>,
}

impl<G: Graphics, V: Vertex + Pod, U: Uniforms + Pod, T: Borrow<G::Texture>>
    BatchRenderer<G, V, U, T>
{
    pub fn new<'a>(
        ctx: G,
        shader: Rc<G::Shader>,
        indices: BatchIndices<G>,
        textures: Vec<SamplerAttribute<G, T>>,
        uniforms: Rc<G::UniformBuffer<U>>,
        draw_parameters: DrawParameters,
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
            textures,
            uniforms,
            draw_parameters,

            unflushed,
            vertices,
            current_buffer: 0,

            indices,
        }
    }

    pub fn draw(&mut self, fb: &G::FrameBuffer, vertices: &[V]) {
        assert!(vertices.len() <= self.unflushed.capacity());

        // If vertices don't fit the unflushed vec, force a draw call to clean it up.
        if vertices.len() > self.unflushed.capacity() - self.unflushed.len() {
            self.flush(fb);
        }

        for v in vertices {
            self.unflushed.push(*v);
        }
    }

    pub fn flush(&mut self, fb: &G::FrameBuffer) {
        if self.unflushed.is_empty() {
            return;
        }

        let current = &self.vertices[self.current_buffer];
        current.0.write(0, &self.unflushed);

        fb.draw(
            &current.1,
            &self.draw_parameters,
            &self.textures,
            Some(&self.uniforms),
            &self.indices.indices(self.unflushed.len()),
        );

        self.unflushed.clear();
        self.current_buffer = (self.current_buffer + 1) % self.vertices.len();
    }
}
