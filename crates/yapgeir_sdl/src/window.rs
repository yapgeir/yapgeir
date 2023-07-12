use std::rc::Rc;

use yapgeir_core::{ScreenPpt, WindowSize};
use yapgeir_realm::{Plugin, Realm, Res, ResMut};

use crate::SdlSettings;

fn update_window_size(mut window_size: ResMut<WindowSize>, window: Res<Rc<sdl2::video::Window>>) {
    let size: (u32, u32) = window.drawable_size();
    window_size.w = size.0;
    window_size.h = size.1;
}

pub fn plugin(settings: SdlSettings) -> impl Plugin {
    move |realm: &mut Realm| {
        let sdl = sdl2::init().expect("Unable to load sdl");
        let video = sdl.video().expect("Unable to init video");

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(settings.gl_profile);
        gl_attr.set_depth_size(settings.depth_size);

        let window = video
            .window(
                &settings.title,
                settings.window_size.w,
                settings.window_size.h,
            )
            .opengl()
            .allow_highdpi()
            .resizable()
            .build()
            .expect("Unable to init window");

        let gl_context = window
            .gl_create_context()
            .expect("Unable to create GLContext");

        let ppt = ScreenPpt(window.drawable_size().0 as f32 / window.size().0 as f32);

        realm
            .add_resource(settings.window_size)
            .add_resource(ppt)
            .add_resource(sdl)
            .add_resource(video)
            .add_resource(Rc::new(window))
            .add_resource(gl_context)
            .add_system(update_window_size);
    }
}
