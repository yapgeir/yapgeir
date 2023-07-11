use bytemuck::{Pod, Zeroable};
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
    texture::Texture,
    uniforms::Uniforms,
    vertex_buffer::Vertex,
    Graphics, ImageSize, Point, Rect,
};

use crate::{
    batch_renderer::{Batch, BatchIndices},
    quad_index_buffer::QuadIndexBuffer,
};

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
            if (gl_FragColor.a == 0.0) discard;
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
        varying in float4 gl_FragCoord : POS;
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
    pub depth: f32,
}

impl SpriteVertex {
    pub fn new(position: [f32; 2], tex_position: [f32; 2], depth: f32) -> Self {
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

pub struct SpriteBatch<'a, G>
where
    G: Graphics,
{
    batch: Batch<
        'a,
        G,
        SpriteVertex,
        SpriteUniforms,
        &'a G::Texture,
        [SamplerAttribute<G, &'a G::Texture>; 1],
    >,
    texture: &'a G::Texture,
}

pub enum DrawRegion {
    /// A point in world space. This will render the sprite with it's center
    /// at this point without any other transformations.
    Point(Point<f32>),

    /// A rectangle in world space. Will render the sprite in this rectangle
    /// implicitly doing non-uniform scaling.
    Rect(Rect<f32>),

    /// Will render the sprite in the quad.
    /// This should be used when you have already pre-calculated transformations.
    /// The quad can be both front or back facing.
    ///
    /// Passing non-convex quads is undefined behavior.
    Quad([Point<f32>; 4]),
}

impl DrawRegion {
    /// Calculate a quad in world-space coordinates.
    pub fn quad(
        self,
        texture_region: &TextureRegion,
        texture_size: ImageSize<u32>,
    ) -> [Point<f32>; 4] {
        match self {
            DrawRegion::Point(point) => {
                // Here we calculate the quad assuming that the center of it is our point,
                // and the size is defined by texture regions pixel size.
                let quad_size = texture_region.pixel_size(texture_size);
                let half_size = (quad_size.w as f32 / 2., quad_size.h as f32 / 2.);

                [
                    Point::new(point.x - half_size.0, point.y - half_size.1),
                    Point::new(point.x + half_size.0, point.y - half_size.1),
                    Point::new(point.x + half_size.0, point.y + half_size.1),
                    Point::new(point.x - half_size.0, point.y + half_size.1),
                ]
            }
            DrawRegion::Rect(rect) => rect.points(),
            DrawRegion::Quad(quad) => quad,
        }
    }
}

pub enum TextureRegion {
    /// Use the whole texture
    Full,

    /// Rectangle in pixels with (0; 0) representing top-left coordinate
    Pixels(Rect<u32>),

    /// Rectangle in texture space with (0; 0) representing top-left corner,
    /// and (1; 1) representing bottom right corner.
    TextureSpace(Rect<f32>),
}

impl TextureRegion {
    pub fn in_texture_space(&self, texture_size: ImageSize<u32>) -> Rect<f32> {
        match self {
            TextureRegion::Full => Rect::new(0.0, 0.0, 1.0, 1.0),
            TextureRegion::TextureSpace(rect) => rect.clone(),
            TextureRegion::Pixels(rect) => Rect::new(
                rect.x as f32 / texture_size.w as f32,
                rect.y as f32 / texture_size.h as f32,
                rect.w as f32 / texture_size.w as f32,
                rect.h as f32 / texture_size.h as f32,
            ),
        }
    }

    pub fn pixel_size(&self, texture_size: ImageSize<u32>) -> ImageSize<u32> {
        match self {
            TextureRegion::Full => texture_size,
            TextureRegion::TextureSpace(rect) => ImageSize::new(
                (rect.w * texture_size.w as f32) as u32,
                (rect.h * texture_size.h as f32) as u32,
            ),
            TextureRegion::Pixels(rect) => (rect.w, rect.h).into(),
        }
    }
}

impl<'a, G> SpriteBatch<'a, G>
where
    G: Graphics,
{
    pub fn draw_sprite(&mut self, sprite: DrawRegion, texture_region: TextureRegion, depth: u16) {
        let quad = sprite.quad(&texture_region, self.texture.size());
        let texture_region = texture_region
            .in_texture_space(self.texture.size())
            .points();

        let depth = (depth as f32 - 32768.) / u16::MAX as f32;

        self.batch.draw(&[
            SpriteVertex::new(quad[0].into(), texture_region[2].into(), depth),
            SpriteVertex::new(quad[1].into(), texture_region[3].into(), depth),
            SpriteVertex::new(quad[2].into(), texture_region[0].into(), depth),
            SpriteVertex::new(quad[3].into(), texture_region[1].into(), depth),
        ])
    }
}

pub struct SpriteRenderer<G>
where
    G: Graphics,
{
    renderer: BatchRenderer<G, SpriteVertex, SpriteUniforms>,
    draw_parameters: DrawParameters,
}

impl<G> SpriteRenderer<G>
where
    G: Graphics,
{
    pub fn new<'a>(ctx: &G, quad_index_buffer: QuadIndexBuffer<G>) -> Self {
        let shader = Rc::new(ctx.new_shader(&SHADER.into()));
        let uniforms = Rc::new(ctx.new_uniform_buffer(&SpriteUniforms::default()));

        Self {
            renderer: BatchRenderer::new(
                ctx,
                shader,
                BatchIndices::Quad(quad_index_buffer),
                uniforms,
                (u16::MAX as usize, 1),
            ),
            draw_parameters: DrawParameters {
                // Use depth buffer to "sort" sprites by their depth on GPU.
                // This won't work for semi-transparent pixels (such as light)
                depth: Some(DrawDepth {
                    test: DepthStencilTest::Less,
                    write: true,
                    // Use the whole range of possible depths
                    range: (-1., 1.),
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
        }
    }

    pub fn start_batch<'a>(
        &'a mut self,
        fb: &'a G::FrameBuffer,
        uniforms: &SpriteUniforms,
        sampler: Sampler<G, &'a G::Texture>,
    ) -> SpriteBatch<'a, G> {
        SpriteBatch {
            texture: sampler.texture,
            batch: self.renderer.start_batch(
                fb,
                &self.draw_parameters,
                uniforms,
                [SamplerAttribute {
                    name: "tex",
                    location: 0,
                    sampler,
                }],
            ),
        }
    }
}
