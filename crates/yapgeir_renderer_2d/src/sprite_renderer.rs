use bytemuck::{Pod, Zeroable};
use std::rc::Rc;
use yapgeir_geometry::{Box2D, Rect};
use yapgeir_graphics_hal::{
    draw_params::{Depth as DrawDepth, DepthStencilTest, DrawParameters},
    frame_buffer::FrameBuffer,
    sampler::Sampler,
    samplers::SamplerAttribute,
    shader::TextShaderSource,
    texture::Texture,
    uniforms::Uniforms,
    vertex_buffer::Vertex,
    Graphics, Size,
};

use crate::{
    batch_renderer::{Batch, BatchIndices},
    quad_index_buffer::QuadIndexBuffer,
    NdcProjection,
};

use super::batch_renderer::BatchRenderer;

#[cfg(not(target_os = "vita"))]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        #version 120

        uniform mat3 view_camera;
        uniform vec2 projection_scale;
        uniform vec2 projection_offset;

        attribute vec2 position;
        attribute vec2 tex_position;
        attribute float depth;

        varying vec2 v_tex_position;

        vec2 round(vec2 value) { 
            return floor(value + vec2(0.5));
        }

        void main() {
            v_tex_position = tex_position;
            vec2 px = round((view_camera * vec3(position, 1.0)).xy);
            vec2 uv = (px + projection_offset) * projection_scale;
            gl_Position = vec4(uv, depth, 1.0);

            // Flip Y axis in the UV.
            gl_Position.y = -gl_Position.y;
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
        uniform float3x3 view_camera;
        uniform float2 projection_scale;
        uniform float2 projection_offset;

        void main(
            float2 position,
            float2 tex_position,
            float depth,

            float2 out v_tex_position: TEXCOORD0,
            float4 out gl_Position : POSITION
        ) {
            v_tex_position = tex_position;
            float2 px = round((mul(view_camera, float3(position, 1.0f))).xy);
            float2 uv = (px + projection_offset) * projection_scale;
            gl_Position = float4(uv, depth, 1.0f);

            // Flip Y axis in the UV.
            gl_Position.y = -gl_Position.y;
        }
    "#,
    fragment: r#"
        uniform sampler2D tex: TEXUNIT0;

        float4 main(
            float2 v_tex_position: TEXCOORD0
        ) {
            float4 gl_FragColor = tex2D(tex, v_tex_position);
            if (gl_FragColor.a == 0.0) discard;

            return gl_FragColor;
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
    pub view_camera: [[f32; 3]; 3],
    pub projection_offset: [f32; 2],
    pub projection_scale: [f32; 2],
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
    Point([f32; 2]),

    /// A rectangle in world space. Will "map" the texture rectangle onto the region.
    Rect(Rect<f32>),

    /// Will render the sprite in the quad.
    /// This should be used when you have already pre-calculated transformations.
    /// The quad points should be in clockwise order.
    ///
    /// Passing non-convex quads is undefined behavior.
    Quad([[f32; 2]; 4]),
}

impl DrawRegion {
    /// Calculate a quad in world-space coordinates.
    pub fn quad(self, texture_region: &TextureRegion, texture_size: Size<u32>) -> [[f32; 2]; 4] {
        match self {
            DrawRegion::Point(point) => {
                // Here we calculate the quad assuming that the center of it is our point,
                // and the size is defined by texture regions pixel size.
                let quad_size = texture_region.pixel_size(texture_size);
                let half_size = (quad_size.w as f32 / 2., quad_size.h as f32 / 2.);

                [
                    [point[0] - half_size.0, point[1] - half_size.1],
                    [point[0] - half_size.0, point[1] + half_size.1],
                    [point[0] + half_size.0, point[1] + half_size.1],
                    [point[0] + half_size.0, point[1] - half_size.1],
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

    #[deprecated = "use TexelsRect instead"]
    Texels(Rect<f32>),

    /// Rectangle in texture space with (0; 0) representing top-left corner,
    /// and (1; 1) representing bottom right corner.
    ///
    /// A Rect is used for convenience. Internally to convert it to points,
    /// some mathematical operations are required making it less efficient than
    /// a Box2D.
    TexelsRect(Rect<f32>),

    /// Rectangle in texture space with (0; 0) representing top-left corner,
    /// and (1; 1) representing bottom right corner.
    ///
    /// Box2D requires no math for conversion to points, so this is the most
    /// efficient way to do draw calls.
    TexelsBox2D(Box2D<f32>),
}

impl TextureRegion {
    pub fn to_texel_quad(&self, texture_size: Size<u32>) -> [[f32; 2]; 4] {
        match self {
            TextureRegion::Full => Rect::new(0.0, 0.0, 1.0, 1.0).points(),
            TextureRegion::TexelsBox2D(box2d) => box2d.points(),
            #[allow(deprecated)]
            TextureRegion::Texels(rect) => rect.points(),
            TextureRegion::TexelsRect(rect) => rect.points(),
            TextureRegion::Pixels(rect) => Rect::new(
                rect.x as f32 / texture_size.w as f32,
                rect.y as f32 / texture_size.h as f32,
                rect.w as f32 / texture_size.w as f32,
                rect.h as f32 / texture_size.h as f32,
            )
            .points(),
        }
    }

    pub fn pixel_size(&self, texture_size: Size<u32>) -> Size<u32> {
        match self {
            TextureRegion::Full => texture_size,
            TextureRegion::Pixels(rect) => rect.size(),
            #[allow(deprecated)]
            TextureRegion::Texels(rect) => Size::new(
                (rect.w * texture_size.w as f32) as u32,
                (rect.h * texture_size.h as f32) as u32,
            ),
            TextureRegion::TexelsRect(rect) => Size::new(
                (rect.w * texture_size.w as f32) as u32,
                (rect.h * texture_size.h as f32) as u32,
            ),
            TextureRegion::TexelsBox2D(box2d) => {
                let size = box2d.size();
                Size::new(
                    (size.w * texture_size.w as f32) as u32,
                    (size.h * texture_size.h as f32) as u32,
                )
            }
        }
    }
}

impl<'a, G> SpriteBatch<'a, G>
where
    G: Graphics,
{
    pub fn draw_sprite(&mut self, sprite: DrawRegion, texture_region: TextureRegion, depth: u16) {
        let quad = sprite.quad(&texture_region, self.texture.size());
        let texture_region = texture_region.to_texel_quad(self.texture.size());

        let depth = (depth as f32 - 32768.) / u16::MAX as f32;

        // Correctly map the UV to the texture region.
        // Since texture and NDC space have different Y axis directions,
        // we must flip the Y axis for the texture region.
        //
        //   1---2                                      0---3
        //   | / | in NDC should be mapped to a texture | \ |
        //   0---3                                      1---2
        self.batch.draw(&[
            SpriteVertex::new(quad[0].into(), texture_region[1].into(), depth),
            SpriteVertex::new(quad[1].into(), texture_region[0].into(), depth),
            SpriteVertex::new(quad[2].into(), texture_region[3].into(), depth),
            SpriteVertex::new(quad[3].into(), texture_region[2].into(), depth),
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
                ..Default::default()
            },
        }
    }

    /// Create a new sprite draw batch.
    ///
    /// Batch will be flushed on drop, so ensure that it is dropped before swap_buffers is called.
    /// It is not recommended to do any other draw calls between start_batch and a drop, because
    /// batch can be flushed at any time when a number of vertices exceeds the buffer size, and not
    /// just on drop.
    ///
    /// # Arguments
    ///
    /// * `frame_buffer` - Frame buffer to draw to.
    /// * `view_camera` - A camera matrix that will transform world space to pixel space.
    /// A camera is separate from a projection, because sprite shader does UV rounding,
    /// and this rounding should be done at pixel scale.
    /// * `projection` - Describes how pixels are projected to normalized display coordinates.
    /// Normalized display coordinates are in a range from [-1; -1] to [1; 1], Y-up with [0; 0]
    /// in the screen center.
    /// * `sampler` - A texture and sampling parameters that will be used for drawing sprites.
    pub fn start_batch<'a>(
        &'a mut self,
        frame_buffer: &'a G::FrameBuffer,
        view_camera: [[f32; 3]; 3],
        projection: NdcProjection,
        sampler: Sampler<G, &'a G::Texture>,
    ) -> SpriteBatch<'a, G> {
        let size = frame_buffer.size();

        let (projection_offset, projection_scale) = projection.offset_and_scale(size);

        SpriteBatch {
            texture: sampler.texture,
            batch: self.renderer.start_batch(
                frame_buffer,
                &self.draw_parameters,
                &SpriteUniforms {
                    view_camera,
                    projection_offset,
                    projection_scale,
                },
                [SamplerAttribute {
                    name: "tex",
                    location: 0,
                    sampler,
                }],
            ),
        }
    }

    /// Create a new sprite draw batch and execute draw calls with it.
    ///
    /// # Arguments
    ///
    /// * `frame_buffer` - Frame buffer to draw to.
    /// * `view_camera` - A camera matrix that will transform world space to pixel space.
    /// A camera is separate from a projection, because sprite shader does UV rounding,
    /// and this rounding should be done at pixel scale.
    /// * `projection` - Describes how pixels are projected to normalized display coordinates.
    /// Normalized display coordinates are in a range from [-1; -1] to [1; 1], Y-up with [0; 0]
    /// in the screen center.
    /// * `sampler` - A texture and sampling parameters that will be used for drawing sprites.
    /// * `draw` - A clojure that does the actual drawing using a batch provided in the argument
    pub fn batch<'a>(
        &'a mut self,
        frame_buffer: &'a G::FrameBuffer,
        view_camera: [[f32; 3]; 3],
        projection: NdcProjection,
        sampler: Sampler<G, &'a G::Texture>,

        draw: impl FnOnce(&mut SpriteBatch<'a, G>),
    ) {
        let mut batch = self.start_batch(frame_buffer, view_camera, projection, sampler);
        draw(&mut batch);
    }
}
