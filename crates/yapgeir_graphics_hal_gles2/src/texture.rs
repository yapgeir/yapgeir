use glow::{HasContext, PixelUnpackData};
use yapgeir_graphics_hal::{
    texture::{PixelFormat, Texture},
    ImageSize, Rect, WindowBackend,
};

use crate::{constants::GlConstant, Gles};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgbLayout {
    U8,
    U16_5_6_5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgbaLayout {
    U8,
    U16_4_4_4_4,
    U16_5_5_5_1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlesPixelFormat {
    Alpha,
    Lumi,
    Lumia,
    Rgb(RgbLayout),
    Rgba(RgbaLayout),
}

impl GlesPixelFormat {
    fn stride(self) -> usize {
        match self {
            GlesPixelFormat::Alpha => 1,
            GlesPixelFormat::Lumi => 1,
            GlesPixelFormat::Lumia => 2,
            GlesPixelFormat::Rgb(layout) => match layout {
                RgbLayout::U8 => 3,
                RgbLayout::U16_5_6_5 => 2,
            },
            GlesPixelFormat::Rgba(layout) => match layout {
                RgbaLayout::U8 => 4,
                RgbaLayout::U16_4_4_4_4 => 2,
                RgbaLayout::U16_5_5_5_1 => 2,
            },
        }
    }
}

impl From<PixelFormat> for GlesPixelFormat {
    fn from(value: PixelFormat) -> Self {
        match value {
            PixelFormat::Alpha => Self::Alpha,
            PixelFormat::Lumi => Self::Lumi,
            PixelFormat::Lumia => Self::Lumia,
            PixelFormat::Rgb => Self::Rgb(RgbLayout::U8),
            PixelFormat::Rgba => Self::Rgba(RgbaLayout::U8),
        }
    }
}

impl GlesPixelFormat {
    fn gl(self) -> (u32, u32) {
        match self {
            GlesPixelFormat::Alpha => (glow::ALPHA, glow::UNSIGNED_BYTE),
            GlesPixelFormat::Lumi => (glow::LUMINANCE, glow::UNSIGNED_BYTE),
            GlesPixelFormat::Lumia => (glow::LUMINANCE_ALPHA, glow::UNSIGNED_BYTE),
            GlesPixelFormat::Rgb(f) => (glow::RGB, f.gl_const()),
            GlesPixelFormat::Rgba(f) => (glow::RGBA, f.gl_const()),
        }
    }
}

pub struct GlesTexture<B: WindowBackend> {
    ctx: Gles<B>,
    format: GlesPixelFormat,
    pub size: ImageSize<u32>,
    pub texture: glow::Texture,
}

impl<B: WindowBackend> Texture<Gles<B>> for GlesTexture<B> {
    type PixelFormat = GlesPixelFormat;

    fn new(
        ctx: Gles<B>,
        format: Self::PixelFormat,
        size: ImageSize<u32>,
        bytes: Option<&[u8]>,
    ) -> Self {
        if let Some(bytes) = bytes {
            let stride = format.stride();
            assert_eq!(bytes.len(), (size.w * size.h) as usize * stride);
        }

        let gl = &ctx.gl;
        let texture = unsafe {
            let (format, ty) = format.gl();
            let texture = gl.create_texture().expect("unable to create a texture");

            ctx.get_ref().activate_texture(texture);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                format as i32,
                size.w as i32,
                size.h as i32,
                0,
                format,
                ty,
                bytes,
            );

            texture
        };

        GlesTexture {
            ctx,
            format,
            size,
            texture,
        }
    }

    fn size(&self) -> ImageSize<u32> {
        self.size
    }

    fn write(
        &self,
        mipmap_level: u32,
        format: Self::PixelFormat,
        size: ImageSize<u32>,
        bytes: &[u8],
    ) {
        let stride = format.stride();
        let (format, ty) = format.gl();
        assert_eq!(format, self.format.gl().0, "format must not change");
        assert_eq!(bytes.len(), (size.w * size.h) as usize * stride);

        self.ctx.get_ref().activate_texture(self.texture);
        unsafe {
            self.ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                mipmap_level as i32,
                format as i32,
                size.w as i32,
                size.h as i32,
                0,
                format,
                ty,
                Some(bytes),
            )
        };
    }

    fn write_rect(
        &self,
        mipmap_level: u32,
        format: Self::PixelFormat,
        rect: Rect<u32>,
        bytes: &[u8],
    ) {
        let stride = format.stride();
        let (format, ty) = format.gl();
        assert_eq!(format, self.format.gl().0, "format must not change");
        assert_eq!(bytes.len(), (rect.w * rect.h) as usize * stride);

        self.ctx.get_ref().activate_texture(self.texture);
        unsafe {
            self.ctx.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                mipmap_level as i32,
                rect.x as i32,
                rect.y as i32,
                rect.w as i32,
                rect.h as i32,
                format,
                ty,
                PixelUnpackData::Slice(bytes),
            )
        };
    }

    fn generate_mipmaps(&self) {
        self.ctx.get_ref().activate_texture(self.texture);
        unsafe {
            let gl = &self.ctx.gl;
            gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }
}

impl<B: WindowBackend> Drop for GlesTexture<B> {
    fn drop(&mut self) {
        let mut ctx = self.ctx.get_ref();

        unsafe {
            // Unbind texture from all units
            let mut u = None;

            for i in 0..ctx.state.texture_unit_limit {
                let unit = &mut ctx.state.texture_units[i];
                if unit.texture.map_or(false, |t| t == self.texture) {
                    ctx.gl.active_texture(glow::TEXTURE0 + i as u32);
                    ctx.gl.bind_texture(glow::TEXTURE_2D, None);
                    unit.texture = None;
                    u = Some(i);
                }
            }

            if let Some(u) = u {
                ctx.state.active_texture_unit = u as u32;
            }

            ctx.clean_texture(self.texture);
            ctx.gl.delete_texture(self.texture);
        }
    }
}
