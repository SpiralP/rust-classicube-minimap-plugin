mod bmp;
mod screen;

use self::screen::Screen;
use log::*;
use std::{cell::RefCell, os::raw::c_void};

thread_local!(
    static SCREEN: RefCell<Option<Screen>> = Default::default();
);

pub fn init() {
    bmp::init();

    SCREEN.with(|cell| {
        let opt = &mut *cell.borrow_mut();

        let screen = Screen::new(render);
        *opt = Some(screen);
    });
}

pub fn on_new_map_loaded() {
    SCREEN.with(|cell| {
        let opt = &mut *cell.borrow_mut();
        let screen = opt.as_mut().unwrap();

        screen.add();
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

unsafe extern "C" fn render(_elem: *mut c_void, _delta: f64) {
    bmp::update();
    bmp::draw();
}
