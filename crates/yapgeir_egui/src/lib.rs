use std::{rc::Rc, time::Instant};

use egui::{ClippedPrimitive, TexturesDelta};
use egui_sdl2_platform::Platform;
use painter::GuiPainter;
use yapgeir_core::Ppt;
use yapgeir_events::Events;
use yapgeir_graphics_hal::{shader::TextShaderSource, Graphics, ImageSize};
use yapgeir_realm::{IntoSystem, Plugin, Realm, Res, ResMut, System};

mod painter;

pub struct GuiDrawData {
    meshes: Vec<ClippedPrimitive>,
    delta: TexturesDelta,
}

pub struct Gui {
    platform: Platform,
    start_time: Instant,
}

pub struct GuiRenderer<G: Graphics> {
    painter: GuiPainter<G>,
    data: Option<GuiDrawData>,
}

impl Gui {
    pub fn context(&mut self) -> egui::Context {
        self.platform.context()
    }
}

impl Gui {
    pub fn new(screen_size: ImageSize<u32>, ppt: Ppt) -> Self {
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
    ppt: Res<Ppt>,
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
    mut renderer: ResMut<GuiRenderer<G>>,
    mut video: ResMut<sdl2::VideoSubsystem>,
) {
    let full_output = gui
        .platform
        .end_frame(&mut video)
        .expect("Unable to end frame");

    renderer.data = Some(GuiDrawData {
        meshes: gui.platform.tessellate(&full_output),
        delta: full_output.textures_delta,
    });
}

pub fn render<'a, G: Graphics>(renderer: &mut GuiRenderer<G>, fb: &G::FrameBuffer, ppt: Ppt) {
    if let Some(data) = renderer.data.as_ref() {
        renderer.painter.paint(fb, *ppt, data);
    }
}

pub fn plugin<'a, G: Graphics, I, S: System<()> + 'static>(
    gui_system: impl IntoSystem<I, (), System = S>,
) -> impl Plugin
where
    G::ShaderSource: From<TextShaderSource<'a>>,
{
    move |realm: &mut Realm| {
        realm
            .initialize_resource_with(|sdl: Res<Rc<sdl2::video::Window>>, ppt: Res<Ppt>| {
                Gui::new(sdl.drawable_size().into(), *ppt)
            })
            .initialize_resource_with(|ctx: Res<G>| GuiRenderer {
                painter: GuiPainter::new(ctx.clone()),
                data: None,
            })
            .add_system(process_input)
            .add_system(gui_system)
            .add_system(tesselate::<G>);
    }
}
