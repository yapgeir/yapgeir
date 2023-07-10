use std::{borrow::Borrow, rc::Rc};

use bytemuck::{Pod, Zeroable};
use derive_more::Constructor;
use yapgeir_graphics_hal::{
    buffer::ByteBuffer,
    buffer::{Buffer, BufferData, BufferKind, BufferUsage},
    draw_descriptor::{AsVertexBindings, IndexBinding},
    draw_params::DrawParameters,
    frame_buffer::{FrameBuffer, Indices},
    index_buffer::PrimitiveMode,
    sampler::Sampler,
    samplers::SamplerAttribute,
    shader::TextShaderSource,
    texture::Texture,
    vertex_buffer::Vertex,
    Graphics, ImageSize, Rect,
};

#[cfg(not(target_os = "vita"))]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        #version 120

        attribute vec2 draw_position;
        attribute vec2 texture_position;

        varying vec2 v_tex_position;

        void main() {
            v_tex_position = texture_position;
            gl_Position = vec4(draw_position, 1, 1);
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
            float2 draw_position,
            float2 texture_position,
            float2 out v_tex_position: TEXCOORD0,
            float4 out gl_Position : POSITION
        ) {
            v_tex_position = texture_position;
            gl_Position = float4(draw_position, 1, 1);
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
    draw_position: [f32; 2],
    texture_position: [f32; 2],
}

pub struct TextureRenderer<G: Graphics> {
    vertices: Rc<Buffer<G, BlitVertex>>,
    draw_descriptor: G::DrawDescriptor,
}

pub enum BlitArea {
    Full,
    PreserveAspectRatio,
    Custom(Rect<f32>),
}

impl BlitArea {
    fn vertices(
        &self,
        screen_size: ImageSize<u32>,
        framebuffer_size: ImageSize<u32>,
    ) -> [BlitVertex; 4] {
        match self {
            BlitArea::Full => [
                BlitVertex::new([-1., -1.], [0., 0.]),
                BlitVertex::new([1., -1.], [1., 0.]),
                BlitVertex::new([1., 1.], [1., 1.]),
                BlitVertex::new([-1., 1.], [0., 1.]),
            ],
            BlitArea::PreserveAspectRatio => {
                let screen_ratio = screen_size.w as f32 / screen_size.h as f32;
                let framebuffer_ratio = framebuffer_size.w as f32 / framebuffer_size.h as f32;

                if screen_ratio > framebuffer_ratio {
                    let scale = framebuffer_ratio / screen_ratio;
                    [
                        BlitVertex::new([-scale, -1.], [0., 0.]),
                        BlitVertex::new([scale, -1.], [1., 0.]),
                        BlitVertex::new([scale, 1.], [1., 1.]),
                        BlitVertex::new([-scale, 1.], [0., 1.]),
                    ]
                } else {
                    let scale = screen_ratio / framebuffer_ratio;
                    [
                        BlitVertex::new([-1., -scale], [0., 0.]),
                        BlitVertex::new([1., -scale], [1., 0.]),
                        BlitVertex::new([1., scale], [1., 1.]),
                        BlitVertex::new([-1., scale], [0., 1.]),
                    ]
                }
            }
            BlitArea::Custom(rect) => [
                BlitVertex::new([rect.x, rect.y], [0., 0.]),
                BlitVertex::new([rect.x, rect.y], [1., 0.]),
                BlitVertex::new([rect.x, rect.y], [1., 1.]),
                BlitVertex::new([rect.x, rect.y], [0., 1.]),
            ],
        }
    }
}

impl<G: Graphics> TextureRenderer<G> {
    pub fn new<'a>(ctx: &G) -> Self {
        let shader = Rc::new(ctx.new_shader(&SHADER.into()));
        let vertices = Rc::new(ctx.new_buffer(
            BufferKind::Vertex,
            BufferUsage::Stream,
            BufferData::<BlitVertex>::Empty(4),
        ));

        let draw_descriptor =
            ctx.new_draw_descriptor(shader, IndexBinding::None, &[vertices.bindings()]);

        Self {
            draw_descriptor,
            vertices,
        }
    }

    pub fn render(
        &self,
        surface: &impl FrameBuffer<G>,
        sampler: Sampler<G, &G::Texture>,
        area: BlitArea,
        draw_parameters: &DrawParameters,
    ) {
        let texture: &G::Texture = sampler.texture.borrow();
        let size = texture.size();

        self.vertices.write(0, &area.vertices(surface.size(), size));

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
