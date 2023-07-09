use bytemuck::{Pod, Zeroable};
use derive_more::{Deref, DerefMut};
use std::rc::Rc;
use yapgeir_graphics_hal::{
    draw_params::Blend,
    draw_params::{
        BlendingFactor, BlendingFunction, Depth as DrawDepth, DepthStencilTest, DrawParameters,
        SeparateBlending,
    },
    sampler::Sampler,
    samplers::SamplerAttribute,
    shader::TextShaderSource,
    texture::Samplers,
    uniforms::Uniforms,
    vertex_buffer::Vertex,
    Graphics,
};

use crate::{batch_renderer::BatchIndices, quad_index_buffer::QuadIndexBuffer};

use super::batch_renderer::BatchRenderer;

#[cfg(not(target_os = "vita"))]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        #version 120

        uniform mat3 view;
        uniform vec2 scale;

        attribute vec2 position;
        attribute vec2 tex_position;
        attribute float depth;

        varying vec2 v_tex_position;

        vec2 round(vec2 value) { 
            return floor(value + vec2(0.5));
        }

        void main() {
            v_tex_position = tex_position;
            vec2 px = round((view * vec3(position, 1.0)).xy);
            vec2 sc = px * scale;
            gl_Position = vec4(sc, depth, 1.0);
        }
    "#,
    fragment: r#"
        #version 120

        uniform sampler2D tex;

        varying vec2 v_tex_position;

        void main() {
            gl_FragColor = texture2D(tex, v_tex_position);
            gl_FragDepth = gl_FragCoord.z * (1-gl_FragColor.a);
        }
    "#,
};

#[cfg(target_os = "vita")]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        uniform float3x3 view;
        uniform float2 scale;

        void main(
            float2 position,
            float2 tex_position,
            float depth,
            float2 out v_tex_position: TEXCOORD0,
            float4 out gl_Position : POSITION
        ) {
            v_tex_position = tex_position;
            float2 px = round((mul(view, float3(position, 1.0f))).xy);
            float2 sc = px * scale;
            gl_Position = float4(sc, depth, 1.0f);
        }
    "#,
    fragment: r#"
        varying out float4 gl_FragColor : COLOR;
        varying out float gl_FragDepth : DEPTH;
        varying in float4 gl_FragCoord : WPOS;
        uniform sampler2D tex: TEXUNIT0;

        void main(
            float2 v_tex_position: TEXCOORD0
        ) {
            gl_FragColor = tex2D(tex, v_tex_position);
            gl_FragDepth = gl_FragCoord.z * (1-gl_FragColor.a);
        }
    "#,
};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, Zeroable, Pod, Vertex)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub tex_position: [f32; 2],
    __padding: [u8; 2],
    pub depth: u16,
}

impl SpriteVertex {
    pub fn new(position: [f32; 2], tex_position: [f32; 2], depth: u16) -> Self {
        Self {
            position,
            tex_position,
            depth,
            ..Default::default()
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, Zeroable, Pod, Uniforms)]
pub struct SpriteUniforms {
    pub view: [[f32; 3]; 3],
    pub scale: [f32; 2],
}

#[derive(Samplers)]
pub struct SpriteSamplers<G: Graphics> {
    pub tex: Sampler<G>,
}

#[derive(Deref, DerefMut)]
pub struct SpriteRenderer<G: Graphics>(BatchRenderer<G, SpriteVertex, SpriteUniforms, G::Texture>);

impl<G: Graphics> SpriteRenderer<G> {
    pub fn new<'a>(ctx: G, quad_index_buffer: QuadIndexBuffer<G>) -> Self
    where
        G::ShaderSource: From<TextShaderSource<'a>>,
    {
        let shader = Rc::new(ctx.new_shader(&SHADER.into()));
        let uniforms = Rc::new(ctx.new_uniform_buffer(&SpriteUniforms::default()));

        Self(BatchRenderer::new(
            ctx,
            shader,
            BatchIndices::Quad(quad_index_buffer),
            Vec::with_capacity(1),
            uniforms,
            DrawParameters {
                // Use depth buffer to "sort" sprites by their depth on GPU.
                // This won't work for semi-transparent pixels (such as light)
                depth: Some(DrawDepth {
                    test: DepthStencilTest::Less,
                    write: true,
                    ..Default::default()
                }),
                blend: Some(Blend {
                    function: SeparateBlending::all(BlendingFunction {
                        source: BlendingFactor::SourceAlpha,
                        destination: BlendingFactor::OneMinusSourceAlpha,
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            (u16::MAX as usize, 1),
        ))
    }

    pub fn set_texture(&mut self, sampler: Sampler<G, G::Texture>) {
        self.textures.clear();
        self.textures.push(SamplerAttribute {
            name: "tex",
            location: 0,
            sampler,
        })
    }
}
