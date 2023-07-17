use std::rc::Rc;

use crate::batch_renderer::{Batch, BatchIndices, BatchRenderer};
use bytemuck::{Pod, Zeroable};
use yapgeir_geometry::Rect;
use yapgeir_graphics_hal::{
    draw_params::DrawParameters, index_buffer::PrimitiveMode, shader::TextShaderSource,
    uniforms::Uniforms, vertex_buffer::Vertex, Graphics, Rgba,
};

#[cfg(not(target_os = "vita"))]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        #version 120
        
        uniform mat3 view_projection;

        attribute vec2 position;
        attribute vec4 color;

        varying vec4 o_color;
        
        void main() {
            o_color = color;
            gl_Position = vec4(view_projection * vec3(position, 1.0), 1.0);

            // Flip Y axis in the UV
            gl_Position.y = -gl_Position.y;
        }
    "#,
    fragment: r#"
        #version 120
        
        varying vec4 o_color;
        
        void main() {
            gl_FragColor = o_color;
        }
    "#,
};

#[cfg(target_os = "vita")]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        uniform float3x3 view_projection;

        void main(
            float2 position,
            float4 color,
            float4 out o_color : COLOR1,
            float4 out gl_Position : POSITION
        ) {
            o_color = color;
            gl_Position = float4(mul(view_projection, float3(position, 1.0f)), 1.0f);
        }
    "#,
    fragment: r#"
        float4 main(float4 o_color : COLOR1) {
            return o_color;
        }
    "#,
};

#[repr(C)]
#[derive(Copy, Clone, Default, Zeroable, Pod, Vertex)]
pub struct PrimitiveVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Default, Zeroable, Pod, Uniforms)]
pub struct PrimitiveUniforms {
    pub view_projection: [[f32; 3]; 3],
}

pub struct PrimitiveBatch<'a, G: Graphics> {
    batch: Batch<'a, G, PrimitiveVertex, PrimitiveUniforms>,
}

impl<G: Graphics> PrimitiveBatch<'_, G> {
    pub fn draw_line(&mut self, start: [f32; 2], end: [f32; 2], color: Rgba<f32>) {
        self.batch.draw(&[
            PrimitiveVertex {
                position: start.into(),
                color: color.into(),
            },
            PrimitiveVertex {
                position: end.into(),
                color: color.into(),
            },
        ]);
    }

    pub fn draw_polygon(&mut self, points: &[[f32; 2]], color: Rgba<f32>) {
        for i in 0..points.len() {
            self.draw_line(points[i], points[(i + 1) % points.len()], color);
        }
    }

    #[inline]
    pub fn draw_rect(&mut self, rect: Rect<f32>, color: Rgba<f32>) {
        self.draw_polygon(&rect.points(), color);
    }
}

pub struct PrimitiveRenderer<G: Graphics> {
    renderer: BatchRenderer<G, PrimitiveVertex, PrimitiveUniforms>,
}

impl<G: Graphics> PrimitiveRenderer<G> {
    pub fn new<'a>(ctx: &G) -> Self {
        let shader = Rc::new(ctx.new_shader(&SHADER.into()));
        let uniforms = Rc::new(ctx.new_uniform_buffer(&PrimitiveUniforms::default()));

        let renderer = BatchRenderer::new(
            ctx,
            shader.clone(),
            BatchIndices::Primitive(PrimitiveMode::Lines),
            uniforms.clone(),
            (u16::MAX as usize, 1),
        );

        Self { renderer }
    }

    pub fn start_batch<'a>(
        &'a mut self,
        frame_buffer: &'a G::FrameBuffer,
        view_projection: [[f32; 3]; 3],
        draw_parameters: &'a DrawParameters,
    ) -> PrimitiveBatch<'a, G> {
        PrimitiveBatch {
            batch: self.renderer.start_batch(
                frame_buffer,
                &draw_parameters,
                &PrimitiveUniforms { view_projection },
                [],
            ),
        }
    }

    pub fn batch<'a>(
        &'a mut self,
        frame_buffer: &'a G::FrameBuffer,
        view_projection: [[f32; 3]; 3],
        draw_parameters: &'a DrawParameters,

        draw: impl FnOnce(&mut PrimitiveBatch<'a, G>),
    ) {
        let mut batch = self.start_batch(frame_buffer, view_projection, draw_parameters);
        draw(&mut batch);
    }
}
