mod bmp;
mod screen;

use self::screen::Screen;
use std::{cell::RefCell, os::raw::c_void};

thread_local!(
    static SCREEN: RefCell<Option<Screen>> = Default::default();
);

pub fn initialize() {
    //
}

pub fn on_new_map_loaded() {
    SCREEN.with(|cell| {
        let opt = &mut *cell.borrow_mut();

        let mut screen = Screen::new(render);

        screen.add();

        *opt = Some(screen);
    });
}

unsafe extern "C" fn render(_elem: *mut c_void, _delta: f64) {
    bmp::update();
    bmp::draw();
}
