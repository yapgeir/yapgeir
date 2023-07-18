use std::{collections::HashMap, mem::size_of};

use bytemuck::{Pod, Zeroable};
use egui::{
    epaint::{textures::TextureFilter, Primitive, Vertex},
    ClippedPrimitive, Color32, TexturesDelta,
};
use yapgeir_graphics_hal::{
    buffer::{Buffer, BufferKind, BufferUsage},
    draw_descriptor::VertexBindings,
    draw_params::{Blend, DrawParameters},
    frame_buffer::{FrameBuffer, Indices},
    index_buffer::PrimitiveMode,
    sampler::{Filter, MinFilter, Sampler, SamplerState, WrapFunction},
    samplers::SamplerAttribute,
    shader::TextShaderSource,
    texture::{PixelFormat, Texture},
    uniforms::{UniformBuffer, Uniforms},
    vertex_buffer::{AttributeKind, VectorSize, VertexAttribute},
    Graphics, Rect, Size,
};

#[derive(Default)]
pub struct EguiDrawData {
    pub meshes: Vec<ClippedPrimitive>,
    pub delta: TexturesDelta,
}

use {egui::epaint::Mesh, std::rc::Rc};

#[cfg(not(target_os = "vita"))]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        #version 120

        uniform vec2 u_screen_size;

        attribute vec2 a_pos;
        attribute vec4 a_srgba; // 0-255 sRGB
        attribute vec2 a_tc;

        varying vec4 v_rgba_gamma; // 0-1 gamma sRGBA
        varying vec2 v_tc;
        
        void main() {
            gl_Position = vec4(
                            2.0 * a_pos.x / u_screen_size.x - 1.0,
                            1.0 - 2.0 * a_pos.y / u_screen_size.y,
                            0.0,
                            1.0);
            v_rgba_gamma = a_srgba / 255.0;
            v_tc = a_tc;

            // Flip Y coordinate in UV.
            gl_Position.y = -gl_Position.y;
        }
    "#,
    fragment: r#"
        #version 120

        #ifdef WEB
        precision highp float;
        #endif

        uniform sampler2D u_sampler;

        varying vec4 v_rgba_gamma; // 0-1 gamma sRGBA
        varying vec2 v_tc;
        
        void main() {
            gl_FragColor = v_rgba_gamma * texture2D(u_sampler, v_tc);
        }
    "#,
};

#[cfg(target_os = "vita")]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        uniform float2 u_screen_size;

        void main(
            float2 a_pos,
            float4 a_srgba, // 0-255 sRGB
            float2 a_tc,
    
            float4 out v_rgba_gamma : TEXCOORD1, // 0-1 gamma sRGBA
            float2 out v_tc : TEXCOORD0,
            float4 out gl_Position : POSITION
        ) {
            gl_Position = float4(
                            2.0 * a_pos.x / u_screen_size.x - 1.0,
                            1.0 - 2.0 * a_pos.y / u_screen_size.y,
                            0.0,
                            1.0);
            v_rgba_gamma = a_srgba / 255.0;
            v_tc = a_tc;
        }
    "#,
    fragment: r#"
        uniform sampler2D u_sampler;
        
        float4 main(
            varying float4 v_rgba_gamma : TEXCOORD1, // 0-1 gamma sRGBA
            varying float2 v_tc : TEXCOORD0
        ) {
            return v_rgba_gamma * tex2D(u_sampler, v_tc);
        }
    "#,
};

const VERTEX_FORMAT: &'static [VertexAttribute] = &[
    VertexAttribute {
        name: "a_pos",
        offset: 0,
        kind: AttributeKind::F32,
        size: VectorSize::N2,
    },
    VertexAttribute {
        name: "a_tc",
        offset: 8,
        kind: AttributeKind::F32,
        size: VectorSize::N2,
    },
    VertexAttribute {
        name: "a_srgba",
        offset: 16,
        kind: AttributeKind::U8,
        size: VectorSize::N4,
    },
];

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Uniforms)]
struct EguiUniforms {
    #[uniforms(name = "u_screen_size")]
    screen_size: [f32; 2],
}

struct DrawResources<G: Graphics> {
    ctx: G,
    shader: Rc<G::Shader>,
    draw_descriptor: Option<G::DrawDescriptor>,
    vertex_buffer: Buffer<G, Vertex>,
    index_buffer: Buffer<G, u32>,
}

impl<G: Graphics> DrawResources<G> {
    fn new<'a>(ctx: &G) -> Self {
        Self {
            shader: Rc::new(ctx.new_shader(&SHADER.into())),
            vertex_buffer: ctx.new_buffer(BufferKind::Vertex, BufferUsage::Stream, 2000),
            index_buffer: ctx.new_buffer(BufferKind::Index, BufferUsage::Stream, 2000),
            draw_descriptor: None,
            ctx: ctx.clone(),
        }
    }

    fn write_indices(&mut self, indices: &[u32]) {
        if self.index_buffer.len() < indices.len() {
            let new_len = (self.index_buffer.len() * 2).max(indices.len());
            self.draw_descriptor = None;
            self.index_buffer =
                self.ctx
                    .new_buffer(BufferKind::Index, BufferUsage::Stream, new_len);
        }

        self.index_buffer.write(0, indices);
    }

    fn write_vertices(&mut self, vertices: &[Vertex]) {
        if self.vertex_buffer.len() < vertices.len() {
            let new_len = (self.vertex_buffer.len() * 2).max(vertices.len());
            self.draw_descriptor = None;
            self.vertex_buffer =
                self.ctx
                    .new_buffer(BufferKind::Vertex, BufferUsage::Stream, new_len);
        }

        self.vertex_buffer.write(0, vertices);
    }

    fn draw_descriptor<'a>(&'a mut self) -> &'a G::DrawDescriptor {
        self.draw_descriptor.get_or_insert_with(|| {
            let vertices = &[VertexBindings {
                buffer: self.vertex_buffer.bytes.clone(),
                attributes: VERTEX_FORMAT,
                stride: size_of::<Vertex>(),
            }];

            self.ctx
                .new_draw_descriptor(self.shader.clone(), Some(&self.index_buffer), &vertices)
        })
    }
}

pub struct EguiPainter<G: Graphics> {
    resources: DrawResources<G>,
    uniform_buffer: G::UniformBuffer<EguiUniforms>,
    samplers: HashMap<egui::TextureId, Sampler<G, G::Texture>>,
}

impl<G: Graphics> EguiPainter<G> {
    pub fn new<'a>(ctx: &G) -> Self {
        Self {
            uniform_buffer: ctx.new_uniform_buffer(&EguiUniforms {
                screen_size: [0., 0.],
            }),
            resources: DrawResources::new(ctx),
            samplers: Default::default(),
        }
    }

    pub fn paint(
        &mut self,
        fb: &G::FrameBuffer,
        pixels_per_point: f32,
        EguiDrawData { delta, meshes }: &EguiDrawData,
    ) {
        for (id, image_delta) in &delta.set {
            match &image_delta.image {
                egui::ImageData::Color(image) => {
                    assert_eq!(
                        image.width() * image.height(),
                        image.pixels.len(),
                        "Mismatch between texture size and texel count"
                    );
                    self.set_texture(*id, image_delta, &image.pixels);
                }
                egui::ImageData::Font(image) => {
                    let pixels = image.srgba_pixels(None).collect::<Vec<_>>();
                    self.set_texture(*id, image_delta, &pixels);
                }
            };
        }

        for m in meshes {
            match &m.primitive {
                Primitive::Mesh(mesh) => {
                    self.paint_mesh(fb, pixels_per_point, &m.clip_rect, &mesh);
                }
                Primitive::Callback(_) => {
                    panic!("Custom rendering callbacks are not implemented");
                }
            }
        }

        for &id in &delta.free {
            self.samplers.remove(&id);
        }
    }

    fn paint_mesh(
        &mut self,
        fb: &G::FrameBuffer,
        pixels_per_point: f32,
        clip_rect: &egui::Rect,
        mesh: &Mesh,
    ) {
        debug_assert!(mesh.is_valid());

        self.resources.write_indices(&mesh.indices);
        self.resources.write_vertices(&mesh.vertices);

        let Size {
            w: width_in_pixels,
            h: height_in_pixels,
        } = self.resources.ctx.default_frame_buffer().size();

        let width_in_points = width_in_pixels as f32 / pixels_per_point;
        let height_in_points = height_in_pixels as f32 / pixels_per_point;
        let screen_size = [width_in_points, height_in_points];

        self.uniform_buffer.write(&EguiUniforms { screen_size });

        if let Some(sampler) = self.samplers.get(&mesh.texture_id) {
            // Transform clip rect to physical pixels:
            let clip_min_x = pixels_per_point * clip_rect.min.x;
            let clip_min_y = pixels_per_point * clip_rect.min.y;
            let clip_max_x = pixels_per_point * clip_rect.max.x;
            let clip_max_y = pixels_per_point * clip_rect.max.y;

            // Make sure clip rect can fit within a `u32`:
            let clip_min_x = clip_min_x.clamp(0.0, width_in_pixels as f32);
            let clip_min_y = clip_min_y.clamp(0.0, height_in_pixels as f32);
            let clip_max_x = clip_max_x.clamp(clip_min_x, width_in_pixels as f32);
            let clip_max_y = clip_max_y.clamp(clip_min_y, height_in_pixels as f32);

            let clip_min_x = clip_min_x.round() as u32;
            let clip_min_y = clip_min_y.round() as u32;
            let clip_max_x = clip_max_x.round() as u32;
            let clip_max_y = clip_max_y.round() as u32;

            let draw_parameters = DrawParameters {
                blend: Some(Blend::alpha()),
                scissor: Some(Rect::new(
                    clip_min_x,
                    clip_min_y,
                    clip_max_x - clip_min_x,
                    clip_max_y - clip_min_y,
                )),
                ..Default::default()
            };

            fb.draw(
                self.resources.draw_descriptor(),
                &draw_parameters,
                &SamplerAttribute::named([("u_sampler", &sampler)]),
                Some(&self.uniform_buffer),
                &Indices {
                    mode: PrimitiveMode::Triangles,
                    offset: 0,
                    len: mesh.indices.len(),
                },
            );
        }
    }

    fn set_texture(
        &mut self,
        tex_id: egui::TextureId,
        delta: &egui::epaint::ImageDelta,
        pixels: &[Color32],
    ) {
        let pixels = bytemuck::cast_slice(&pixels);

        let sampler_state = SamplerState {
            wrap: WrapFunction::Clamp,
            min_filter: MinFilter::Origin(filter(delta.options.minification)),
            mag_filter: filter(delta.options.magnification),
        };

        if let Some(pos) = delta.pos {
            // update a sub-region
            if let Some(sampler) = self.samplers.get_mut(&tex_id) {
                sampler.texture.write_rect(
                    1,
                    PixelFormat::Rgba.into(),
                    Rect::new(
                        pos[0] as u32,
                        pos[1] as u32,
                        delta.image.width() as u32,
                        delta.image.height() as u32,
                    ),
                    &pixels,
                );
                sampler.state = sampler_state;
            }
        } else {
            let texture = self.resources.ctx.new_texture(
                PixelFormat::Rgba,
                Size::new(delta.image.width() as u32, delta.image.height() as u32),
                Some(pixels),
            );

            let sampler = Sampler::new(texture, sampler_state);

            self.samplers.insert(tex_id, sampler);
        }
    }
}

fn filter(filter: TextureFilter) -> Filter {
    match filter {
        TextureFilter::Nearest => Filter::Nearest,
        TextureFilter::Linear => Filter::Linear,
    }
}
