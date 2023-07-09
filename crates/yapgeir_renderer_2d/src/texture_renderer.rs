use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use derive_more::Constructor;
use yapgeir_graphics_hal::{
    buffer::{BufferKind, BufferUsage},
    draw_descriptor::{AsVertexBindings, IndexBinding},
    draw_params::DrawParameters,
    frame_buffer::{FrameBuffer, Indices},
    index_buffer::PrimitiveMode,
    sampler::Sampler,
    samplers::SamplerAttribute,
    shader::TextShaderSource,
    vertex_buffer::Vertex,
    Graphics,
};

#[cfg(not(target_os = "vita"))]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        #version 120

        attribute vec2 position;
        varying vec2 v_tex_position;

        void main() {
            v_tex_position = (position + 1) * 0.5;
            gl_Position = vec4(position, 1, 1);
        }
    "#,
    fragment: r#"
        #version 120

        uniform sampler2D tex;

        varying vec2 v_tex_position;
        void main() {
            gl_FragColor = texture2D(tex, v_tex_position);
        }
    "#,
};

#[cfg(target_os = "vita")]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        void main(
            float2 position,
            float2 out v_tex_position: TEXCOORD0,
            float4 out gl_Position : POSITION
        ) {
            v_tex_position = (position + 1) * 0.5;
            gl_Position = float4(position, 1, 1);
        }
    "#,
    fragment: r#"
        uniform sampler2D tex: TEXUNIT0;

        float4 main(float2 v_tex_position: TEXCOORD0) {
            return tex2D(tex, v_tex_position);
        }
    "#,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Constructor, Zeroable, Pod, Vertex)]
struct BlitVertex {
    position: [f32; 2],
}

pub struct TextureRenderer<G: Graphics> {
    draw_descriptor: G::DrawDescriptor,
}

impl<G: Graphics> TextureRenderer<G> {
    pub fn new<'a>(ctx: &G) -> Self
    where
        G::ShaderSource: From<TextShaderSource<'a>>,
    {
        let shader = Rc::new(ctx.new_shader(&SHADER.into()));
        let vertices = ctx.new_buffer(
            BufferKind::Vertex,
            BufferUsage::Static,
            &[
                BlitVertex::new([-1., -1.]),
                BlitVertex::new([-1., 1.]),
                BlitVertex::new([1., 1.]),
                BlitVertex::new([1., -1.]),
            ],
        );

        let draw_descriptor =
            ctx.new_draw_descriptor(shader, IndexBinding::None, &[vertices.bindings()]);

        Self { draw_descriptor }
    }

    pub fn render<'t>(
        &self,
        surface: &impl FrameBuffer<G>,
        sampler: Sampler<G, &'t G::Texture>,
        draw_parameters: &DrawParameters,
    ) {
        surface.draw::<()>(
            &self.draw_descriptor,
            draw_parameters,
            &SamplerAttribute::named([("tex", &sampler)]),
            None,
            &Indices {
                mode: PrimitiveMode::TriangleFan,
                offset: 0,
                len: 4,
            },
        );
    }
}
