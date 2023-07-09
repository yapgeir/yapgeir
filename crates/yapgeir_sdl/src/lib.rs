use nalgebra::Vector2;
use yapgeir_realm::{Plugin, Realm};

pub mod events;
pub mod input;
pub mod timer;
pub mod window;

pub struct SdlSettings {
    pub screen_size: Vector2<u32>,
    pub gl_profile: sdl2::video::GLProfile,
    pub depth_size: u8,
}

impl Default for SdlSettings {
    fn default() -> Self {
        Self {
            screen_size: Vector2::new(1920, 1080),
            gl_profile: sdl2::video::GLProfile::Compatibility,
            depth_size: 16,
        }
    }
}

pub fn plugin(settings: SdlSettings) -> impl Plugin {
    move |realm: &mut Realm| {
        realm
            .add_plugin(window::plugin(settings))
            .add_plugin(timer::plugin)
            .add_plugin(events::plugin)
            .add_plugin(input::plugin);
    }
}
