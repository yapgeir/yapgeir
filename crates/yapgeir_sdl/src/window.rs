use std::rc::Rc;

use yapgeir_core::Ppt;
use yapgeir_realm::{Plugin, Realm};

use crate::SdlSettings;

pub fn plugin(settings: SdlSettings) -> impl Plugin {
    move |realm: &mut Realm| {
        let sdl = sdl2::init().expect("Unable to load sdl");
        let video = sdl.video().expect("Unable to init video");

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(settings.gl_profile);
        gl_attr.set_depth_size(settings.depth_size);

        let window = video
            .window("", settings.screen_size.x, settings.screen_size.y)
            .opengl()
            .allow_highdpi()
            .resizable()
            .build()
            .expect("Unable to init window");

        let gl_context = window
            .gl_create_context()
            .expect("Unable to create GLContext");

        let ppt = Ppt(window.drawable_size().0 as f32 / window.size().0 as f32);

        realm
            .add_resource(ppt)
            .add_resource(sdl)
            .add_resource(video)
            .add_resource(Rc::new(window))
            .add_resource(gl_context);
    }
}
