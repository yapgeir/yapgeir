use yapgeir_core::WindowSize;
use yapgeir_realm::{Plugin, Realm};

pub use sdl2;

pub mod events;
pub mod input;
pub mod timer;
pub mod window;

pub struct SdlSettings {
    pub title: String,
    pub window_size: WindowSize,
    pub gl_profile: sdl2::video::GLProfile,
    pub depth_size: u8,
}

impl Default for SdlSettings {
    fn default() -> Self {
        Self {
            title: "Hello, yapgeir!".into(),
            window_size: WindowSize::new(1920, 1080),
            #[cfg(not(target_os = "emscripten"))]
            gl_profile: sdl2::video::GLProfile::Compatibility,
            #[cfg(target_os = "emscripten")]
            gl_profile: sdl2::video::GLProfile::GLES,
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
