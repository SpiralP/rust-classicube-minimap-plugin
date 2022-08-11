mod bmp;

use std::{cell::RefCell, os::raw::c_void};

use classicube_sys::screen::{OwnedScreen, Priority};
use tracing::debug;

thread_local!(
    static SCREEN: RefCell<Option<OwnedScreen>> = Default::default();
);

pub fn init() {
    bmp::init();

    SCREEN.with(|cell| {
        let opt = &mut *cell.borrow_mut();

        let mut screen = OwnedScreen::new();
        screen.on_render(render);
        *opt = Some(screen);
    });
}

pub fn free() {
    debug!("plugin::free()");

    SCREEN.with(|cell| {
        let opt = &mut *cell.borrow_mut();
        drop(opt.take());
    });

    bmp::free();
}

pub fn on_new_map_loaded() {
    SCREEN.with(|cell| {
        let opt = &mut *cell.borrow_mut();
        let screen = opt.as_mut().unwrap();

        screen.add(Priority::UnderEverything);
    });
}

unsafe extern "C" fn render(_elem: *mut c_void, _delta: f64) {
    bmp::update();
    bmp::draw();
}
