use std::collections::HashMap;

use glow::HasContext;
use yapgeir_graphics_hal::sampler::SamplerState;

use crate::{constants::GlConstant, context::GlesContextRef};

#[derive(Default)]
pub struct Samplers {
    real_cache: HashMap<SamplerState, glow::Sampler>,
    fallback_cache: HashMap<glow::Texture, SamplerState>,
}

impl<'a> GlesContextRef<'a> {
    pub fn clean_texture(&mut self, texture: glow::Texture) {
        self.state.samplers.fallback_cache.remove(&texture);
    }
}

impl Samplers {
    pub fn drain(&mut self, gl: &glow::Context) {
        for (_, v) in self.real_cache.drain() {
            unsafe {
                gl.delete_sampler(v);
            }
        }
    }
}

impl<'a> GlesContextRef<'a> {
    fn get_sampler_object(&mut self, state: SamplerState) -> glow::Sampler {
        match self.state.samplers.real_cache.get(&state) {
            Some(sampler) => return *sampler,
            None => {}
        };

        let sampler = unsafe {
            let gl = &self.gl;
            let sampler = gl.create_sampler().expect("unable to create sampler");

            let wrap_gl = state.wrap.gl_const();
            let min_filter_gl = state.min_filter.gl_const();
            let mag_filter_gl = state.mag_filter.gl_const();

            gl.sampler_parameter_i32(sampler, glow::TEXTURE_WRAP_S, wrap_gl as i32);
            gl.sampler_parameter_i32(sampler, glow::TEXTURE_WRAP_T, wrap_gl as i32);
            gl.sampler_parameter_i32(sampler, glow::TEXTURE_MIN_FILTER, min_filter_gl as i32);
            gl.sampler_parameter_i32(sampler, glow::TEXTURE_MAG_FILTER, mag_filter_gl as i32);

            sampler
        };

        self.state.samplers.real_cache.insert(state, sampler);

        sampler
    }

    fn bind_sampler_object(&mut self, unit: u32, state: SamplerState) {
        if self.state.texture_units[unit as usize].sampler == state {
            return;
        }

        let sampler = self.get_sampler_object(state);
        unsafe {
            self.gl.bind_sampler(unit, Some(sampler));
        }

        self.state.texture_units[unit as usize].sampler = state;
    }

    fn bind_sampling_data(&mut self, unit: u32, state: SamplerState) {
        let texture = match self.state.texture_units[unit as usize].texture {
            Some(texture) => texture,
            None => return,
        };

        if self.state.samplers.fallback_cache.get(&texture) == Some(&state) {
            // No need to change sampling parameters
            return;
        }

        self.activate_texture_unit(unit);

        let wrap_gl = state.wrap.gl_const();
        let min_filter_gl = state.min_filter.gl_const();
        let mag_filter_gl = state.mag_filter.gl_const();

        unsafe {
            self.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap_gl as i32);
            self.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap_gl as i32);
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                min_filter_gl as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                mag_filter_gl as i32,
            );
        }

        self.state.samplers.fallback_cache.insert(texture, state);
        self.state.texture_units[unit as usize].sampler = state;
    }

    pub fn bind_sampler(&mut self, unit: u32, state: SamplerState) {
        if self.extensions.sampler_objects {
            self.bind_sampler_object(unit, state);
        } else {
            self.bind_sampling_data(unit, state);
        }
    }
}
