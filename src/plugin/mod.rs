mod bmp;

use classicube_sys::{screen, screen::Screen};
use std::{cell::RefCell, os::raw::c_void};
use tracing::debug;

thread_local!(
    static SCREEN: RefCell<Option<Screen>> = Default::default();
);

pub fn init() {
    bmp::init();

    SCREEN.with(|cell| {
        let opt = &mut *cell.borrow_mut();

        let screen = Screen::new(screen::Callbacks {
            render: Some(render),
            ..Default::default()
        });
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

        screen.add(screen::Priority::UnderEverything);
    });
}

unsafe extern "C" fn render(_elem: *mut c_void, _delta: f64) {
    bmp::update();
    bmp::draw();
}
