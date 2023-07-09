use std::rc::Rc;

use crate::batch_renderer::{BatchIndices, BatchRenderer};
use bytemuck::{Pod, Zeroable};
use yapgeir_graphics_hal::{
    draw_params::DrawParameters, index_buffer::PrimitiveMode, shader::TextShaderSource,
    uniforms::Uniforms, vertex_buffer::Vertex, Graphics, Rgba,
};

#[cfg(not(target_os = "vita"))]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        #version 120
        
        attribute vec2 position;
        attribute vec4 color;
        
        uniform mat3 view;
        
        varying vec4 o_color;
        
        void main() {
            o_color = color;
            gl_Position = vec4(view * vec3(position, 1.0), 1.0);
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
        uniform float3x3 view;

        void main(
            float2 position,
            float4 color,
            float4 out v_color : COLOR1,
            float4 out gl_Position : POSITION
        ) {
            v_color = color;
            gl_Position = float4(mul(view, float3(position, 1.0f)), 1.0f);
        }
    "#,
    fragment: r#"
        float4 main(float4 v_color : COLOR1) {
            return v_color;
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
    pub view: [[f32; 3]; 3],
}

pub struct PrimitiveRenderer<G: Graphics> {
    pub uniforms: Rc<G::UniformBuffer<PrimitiveUniforms>>,
    pub lines: BatchRenderer<G, PrimitiveVertex, PrimitiveUniforms>,
    pub dots: BatchRenderer<G, PrimitiveVertex, PrimitiveUniforms>,
}

impl<G: Graphics> PrimitiveRenderer<G> {
    pub fn new<'a>(ctx: &G) -> Self
    where
        G::ShaderSource: From<TextShaderSource<'a>>,
    {
        let shader = Rc::new(ctx.new_shader(&SHADER.into()));
        let uniforms = Rc::new(ctx.new_uniform_buffer(&PrimitiveUniforms::default()));

        let lines = BatchRenderer::new(
            ctx,
            shader.clone(),
            BatchIndices::Primitive(PrimitiveMode::Lines),
            vec![],
            uniforms.clone(),
            DrawParameters::default(),
            (u16::MAX as usize, 1),
        );

        let dots = BatchRenderer::new(
            ctx,
            shader,
            BatchIndices::Primitive(PrimitiveMode::Points),
            vec![],
            uniforms.clone(),
            DrawParameters::default(),
            (u16::MAX as usize, 1),
        );

        Self {
            uniforms,
            lines,
            dots,
        }
    }

    pub fn draw_rect(&mut self, fb: &G::FrameBuffer, points: [[f32; 2]; 4], color: Rgba<f32>) {
        let points = points.map(|position| PrimitiveVertex {
            position,
            color: [color.r, color.g, color.b, color.a],
        });
        self.lines.draw(fb, &points_to_rect(&points));
    }
}

#[inline]
pub fn points_to_rect<T: Clone>(points: &[T; 4]) -> [T; 8] {
    [0, 1, 1, 2, 2, 3, 3, 0].map(|id| points[id].clone())
}
