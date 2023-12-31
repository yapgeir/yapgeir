use std::{cell::RefCell, ffi::c_void, rc::Rc};

use yapgeir_graphics_hal::{Graphics, Size, WindowBackend};
use yapgeir_realm::{Realm, Res};
use yapgeir_sdl::sdl2::{self, video::SwapInterval};

pub struct SdlWindowBackend(Rc<RefCell<sdl2::video::Window>>);

impl WindowBackend for SdlWindowBackend {
    fn swap_buffers(&self) {
        self.0.borrow().gl_swap_window();
    }

    fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.0.borrow().subsystem().gl_get_proc_address(symbol) as *const c_void
    }

    fn default_frame_buffer_size(&self) -> Size<u32> {
        self.0.borrow().drawable_size().into()
    }
}

pub fn plugin<G>(realm: &mut Realm)
where
    G: Graphics<Backend = SdlWindowBackend>,
{
    realm.initialize_resource_with(move |window: Res<Rc<RefCell<sdl2::video::Window>>>| {
        let backend = SdlWindowBackend(window.clone());
        let renderer = G::new(backend);

        window
            .borrow()
            .gl_set_context_to_current()
            .expect("unable to set current gl context");

        window
            .borrow()
            .subsystem()
            .gl_set_swap_interval(SwapInterval::VSync)
            .expect("Unable to set swap interval");

        renderer
    });
}
