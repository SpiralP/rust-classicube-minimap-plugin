#![allow(non_snake_case)]
#![allow(unused_variables)]

use classicube_sys::{Gui_Add, Gui_Remove, Screen as CCScreen, ScreenVTABLE};
use std::{
    mem,
    os::raw::{c_char, c_int, c_void},
    pin::Pin,
};

pub struct Screen {
    _vtable: Pin<Box<ScreenVTABLE>>,
    screen: Pin<Box<CCScreen>>,
}

impl Screen {
    pub fn new(render: unsafe extern "C" fn(elem: *mut c_void, delta: f64)) -> Self {
        let mut _vtable = Box::pin(ScreenVTABLE {
            Init: Some(Self::Init),
            Update: Some(Self::Update),
            Free: Some(Self::Free),
            Render: Some(render),
            BuildMesh: Some(Self::BuildMesh),
            HandlesInputDown: Some(Self::HandlesInputDown),
            HandlesInputUp: Some(Self::HandlesInputUp),
            HandlesKeyPress: Some(Self::HandlesKeyPress),
            HandlesTextChanged: Some(Self::HandlesTextChanged),
            HandlesPointerDown: Some(Self::HandlesPointerDown),
            HandlesPointerUp: Some(Self::HandlesPointerUp),
            HandlesPointerMove: Some(Self::HandlesPointerMove),
            HandlesMouseScroll: Some(Self::HandlesMouseScroll),
            Layout: Some(Self::Layout),
            ContextLost: Some(Self::ContextLost),
            ContextRecreated: Some(Self::ContextRecreated),
        });

        let screen = Box::pin(unsafe {
            let mut screen: CCScreen = mem::zeroed();
            screen.VTABLE = _vtable.as_mut().get_unchecked_mut();
            screen
        });

        Self { _vtable, screen }
    }

    pub fn add(&mut self) {
        unsafe {
            Gui_Add(self.screen.as_mut().get_unchecked_mut(), 0);
        }
    }

    pub fn remove(&mut self) {
        unsafe {
            Gui_Remove(self.screen.as_mut().get_unchecked_mut());
        }
    }

    /// Initialises persistent state.
    unsafe extern "C" fn Init(elem: *mut c_void) {}

    /// Updates this screen, called every frame just before Render().
    unsafe extern "C" fn Update(elem: *mut c_void, delta: f64) {}

    /// Frees/releases persistent state.
    unsafe extern "C" fn Free(elem: *mut c_void) {}

    /// Draws this screen and its widgets on screen.
    // unsafe extern "C" fn Render(elem: *mut c_void, delta: f64) {}

    /// Builds the vertex mesh for all the widgets in the screen.
    unsafe extern "C" fn BuildMesh(elem: *mut c_void) {}

    /// Returns non-zero if an input press is handled.
    unsafe extern "C" fn HandlesInputDown(elem: *mut c_void, key: c_int) -> c_int {
        0
    }

    /// Returns non-zero if an input release is handled.
    unsafe extern "C" fn HandlesInputUp(elem: *mut c_void, key: c_int) -> c_int {
        0
    }

    /// Returns non-zero if a key character press is handled.
    unsafe extern "C" fn HandlesKeyPress(elem: *mut c_void, keyChar: c_char) -> c_int {
        0
    }

    /// Returns non-zero if a key character press is handled.
    /// Currently only raised by on-screen keyboard in web client.
    unsafe extern "C" fn HandlesTextChanged(
        elem: *mut c_void,
        str: *const classicube_sys::String,
    ) -> c_int {
        0
    }

    /// Returns non-zero if a pointer press is handled.
    unsafe extern "C" fn HandlesPointerDown(
        elem: *mut c_void,
        id: c_int,
        x: c_int,
        y: c_int,
    ) -> c_int {
        0
    }

    /// Returns non-zero if a pointer release is handled.
    unsafe extern "C" fn HandlesPointerUp(
        elem: *mut c_void,
        id: c_int,
        x: c_int,
        y: c_int,
    ) -> c_int {
        0
    }

    /// Returns non-zero if a pointer movement is handled.
    unsafe extern "C" fn HandlesPointerMove(
        elem: *mut c_void,
        id: c_int,
        x: c_int,
        y: c_int,
    ) -> c_int {
        0
    }

    /// Returns non-zero if a mouse wheel scroll is handled.
    unsafe extern "C" fn HandlesMouseScroll(elem: *mut c_void, delta: f32) -> c_int {
        0
    }

    /// Positions widgets on screen. Typically called on window resize.
    unsafe extern "C" fn Layout(elem: *mut c_void) {}

    /// Destroys graphics resources. (textures, vertex buffers, etc)
    unsafe extern "C" fn ContextLost(elem: *mut c_void) {}

    /// Allocates graphics resources. (textures, vertex buffers, etc)
    unsafe extern "C" fn ContextRecreated(elem: *mut c_void) {}
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.remove();
    }
}

// /* Functions for a Screen instance. */
// struct ScreenVTABLE {
// 	/* Initialises persistent state. */
// 	void (*Init)(void* elem);
// 	/* Updates this screen, called every frame just before Render(). */
// 	void (*Update)(void* elem, double delta);
// 	/* Frees/releases persistent state. */
// 	void (*Free)(void* elem);
// 	/* Draws this screen and its widgets on screen. */
// 	void (*Render)(void* elem, double delta);
// 	/* Builds the vertex mesh for all the widgets in the screen. */
// 	void (*BuildMesh)(void* elem);
// 	/* Returns non-zero if an input press is handled. */
// 	int  (*HandlesInputDown)(void* elem, int key);
// 	/* Returns non-zero if an input release is handled. */
// 	int  (*HandlesInputUp)(void* elem, int key);
// 	/* Returns non-zero if a key character press is handled. */
// 	int  (*HandlesKeyPress)(void* elem, char keyChar);
// 	/* Returns non-zero if a key character press is handled. */
// 	/* Currently only raised by on-screen keyboard in web client. */
// 	int  (*HandlesTextChanged)(void* elem, const String* str);
// 	/* Returns non-zero if a pointer press is handled. */
// 	int  (*HandlesPointerDown)(void* elem, int id, int x, int y);
// 	/* Returns non-zero if a pointer release is handled. */
// 	int  (*HandlesPointerUp)(void* elem,   int id, int x, int y);
// 	/* Returns non-zero if a pointer movement is handled. */
// 	int  (*HandlesPointerMove)(void* elem, int id, int x, int y);
// 	/* Returns non-zero if a mouse wheel scroll is handled. */
// 	int  (*HandlesMouseScroll)(void* elem, float delta);
// 	/* Positions widgets on screen. Typically called on window resize. */
// 	void (*Layout)(void* elem);
// 	/* Destroys graphics resources. (textures, vertex buffers, etc) */
// 	void (*ContextLost)(void* elem);
// 	/* Allocates graphics resources. (textures, vertex buffers, etc) */
// 	void (*ContextRecreated)(void* elem);
// };

// /* Represents a container of widgets and other 2D elements. May cover entire window. */
// struct Screen {
//     const struct ScreenVTABLE* VTABLE;
//     cc_bool grabsInput;  /* Whether this screen grabs input. Causes the cursor to become visible. */
//     cc_bool blocksWorld; /* Whether this screen completely and opaquely covers the game world behind it. */
//     cc_bool closable;    /* Whether this screen is automatically closed when pressing Escape */
//     cc_bool dirty;       /* Whether this screens needs to have its mesh rebuilt. */
//     int maxVertices;
//     GfxResourceID vb;
//     struct Widget** widgets;
//     int numWidgets;
// };
