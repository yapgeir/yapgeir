use std::{ops::Deref, rc::Rc, time::Instant};

use egui_sdl2_platform::Platform;
use yapgeir_core::ScreenPpt;
use yapgeir_egui_painter::{EguiDrawData, EguiPainter};
use yapgeir_events::Events;
use yapgeir_graphics_hal::{Graphics, ImageSize};
use yapgeir_realm::{IntoSystem, Plugin, Realm, Res, ResMut, System};

pub struct EguiRenderer<G: Graphics> {
    painter: EguiPainter<G>,
    data: EguiDrawData,
}

pub struct Gui {
    platform: Platform,
    start_time: Instant,
}

impl Gui {
    pub fn context(&mut self) -> egui::Context {
        self.platform.context()
    }
}

impl Gui {
    pub fn new(screen_size: ImageSize<u32>, ppt: ScreenPpt) -> Self {
        let mut platform =
            Platform::new((screen_size.w, screen_size.h)).expect("Unable to create GUI");
        platform.set_pixels_per_point(Some(ppt.0));

        Self {
            start_time: Instant::now(),
            platform,
        }
    }
}

#[cfg_attr(feature = "instrumentation", yapgeir_instrument::instrument)]
fn process_input(
    mut gui: ResMut<Gui>,
    sdl: Res<sdl2::Sdl>,
    video: Res<sdl2::VideoSubsystem>,
    events: Res<Events<sdl2::event::Event>>,
    ppt: Res<ScreenPpt>,
) {
    for event in events.iter() {
        gui.platform.handle_event(&event, &sdl, &video);
    }

    let elapsed = gui.start_time.elapsed().as_secs_f64();
    gui.platform.update_time(elapsed);
    gui.platform.set_pixels_per_point(Some(ppt.0));
}

#[cfg_attr(feature = "instrumentation", yapgeir_instrument::instrument)]
fn tesselate<G: Graphics>(
    mut gui: ResMut<Gui>,
    mut renderer: ResMut<EguiRenderer<G>>,
    mut video: ResMut<sdl2::VideoSubsystem>,
) {
    let full_output = gui
        .platform
        .end_frame(&mut video)
        .expect("Unable to end frame");

    renderer.data = EguiDrawData {
        meshes: gui.platform.tessellate(&full_output),
        delta: full_output.textures_delta,
    };
}

pub fn render<'a, G: Graphics>(
    renderer: &mut EguiRenderer<G>,
    fb: &G::FrameBuffer,
    ppt: ScreenPpt,
) {
    renderer.painter.paint(fb, *ppt, &renderer.data);
}

pub fn plugin<'a, G: Graphics, I, S: System<()> + 'static>(
    gui_system: impl IntoSystem<I, (), System = S>,
) -> impl Plugin {
    move |realm: &mut Realm| {
        realm
            .initialize_resource_with(|sdl: Res<Rc<sdl2::video::Window>>, ppt: Res<ScreenPpt>| {
                Gui::new(sdl.drawable_size().into(), *ppt)
            })
            .initialize_resource_with(|ctx: Res<G>| EguiRenderer {
                painter: EguiPainter::new(ctx.deref()),
                data: Default::default(),
            })
            .add_system(process_input)
            .add_system(gui_system)
            .add_system(tesselate::<G>);
    }
}
