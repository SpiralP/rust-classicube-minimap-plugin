use crate::helpers::Texture_RenderShaded;
use classicube_helpers::{detour::static_detour, tick::TickEventHandler, CellGetSet};
use classicube_sys::*;
use std::cell::{Cell, RefCell};

const WHITE_TRANSPARENT: PackedCol = PackedCol_Make(255, 255, 255, 0);
const TEXTURE_WIDTH: usize = 1024;
const TEXTURE_HEIGHT: usize = 1024;

thread_local!(
    static TEXTURE: RefCell<OwnedGfxTexture> = {
        // must be a vec or else we try to fit huge array onto stack and crash!
        let mut pixels: Vec<u8> = vec![255; 4 * TEXTURE_WIDTH * TEXTURE_HEIGHT];

        let mut bmp = Bitmap {
            Scan0: pixels.as_mut_ptr(),
            Width: TEXTURE_WIDTH as i32,
            Height: TEXTURE_HEIGHT as i32,
        };

        RefCell::new(OwnedGfxTexture::create(&mut bmp, true, false))
    };
);

// fn update_texture() {
//     let part = Bitmap {
//         Scan0: new_pixels as *mut _,
//         Width: TEXTURE_WIDTH as i32,
//         Height: TEXTURE_HEIGHT as i32,
//     };

//     Gfx_UpdateTexturePart(self.texture.resource_id, 0, 0, &mut part, 0);
// }

extern "C" fn draw() {
    TEXTURE.with(|cell| {
        let owned_texture = &*cell.borrow();

        unsafe {
            let mut texture = Texture {
                ID: owned_texture.resource_id,
                X: 400,
                Y: 50,
                Width: 300 as _,
                Height: 200 as _,
                uv: TextureRec {
                    U1: 0.0,
                    V1: 0.0,
                    U2: 1.0,
                    V2: 1.0,
                },
            };

            Texture_RenderShaded(&mut texture, WHITE_TRANSPARENT);
        }
    });
}

thread_local!(
    static FIRST_IN_TICK: Cell<bool> = Cell::new(false);
);

thread_local!(
    static ISO_TRANSFORM: Cell<Option<*mut Matrix>> = Default::default();
);

static_detour!(
    static LOAD_MATRIX_DETOUR: unsafe extern "C" fn(MatrixType, *mut Matrix);
);
fn load_matrix(type_: MatrixType, matrix: *mut Matrix) {
    unsafe {
        // log::debug!("load_matrix {:?} {:?}", type_, matrix);

        if type_ == MatrixType__MATRIX_VIEW {
            if let Some(iso_transform) = ISO_TRANSFORM.get() {
                if matrix == iso_transform && FIRST_IN_TICK.get() {
                    FIRST_IN_TICK.set(false);
                    log::warn!("GOTEM");
                    draw();
                }
            }
        }

        LOAD_MATRIX_DETOUR.call(type_, matrix);
    }
}

thread_local!(
    static ABOUT_TO_CALL_MUL: Cell<bool> = Cell::new(false);
);

static_detour!(
    static MATRIX_MUL_DETOUR: unsafe extern "C" fn(*mut Matrix, *const Matrix, *const Matrix);
);
fn matrix_mul(result: *mut Matrix, left: *const Matrix, right: *const Matrix) {
    unsafe {
        if ABOUT_TO_CALL_MUL.get() {
            ISO_TRANSFORM.set(Some(result));
            ABOUT_TO_CALL_MUL.set(false);

            log::info!("FOUND ISO_TRANSFORM");

            MATRIX_MUL_DETOUR.disable().unwrap();
            Matrix_Mul(result, left, right);
        } else {
            MATRIX_MUL_DETOUR.call(result, left, right);
            MATRIX_MUL_DETOUR.disable().unwrap();
            Matrix_Mul(result, left, right);
            MATRIX_MUL_DETOUR.enable().unwrap();
        }
    }
}

static_detour!(
    static MATRIX_ROTATEX_DETOUR: unsafe extern "C" fn(*mut Matrix, f32);
);
fn matrix_rotatex(result: *mut Matrix, angle: f32) {
    unsafe {
        if (angle - -30f32 * MATH_DEG2RAD as f32).abs() < 0.00001 {
            ABOUT_TO_CALL_MUL.set(true);

            MATRIX_ROTATEX_DETOUR.disable().unwrap();
            Matrix_RotateX(result, angle);
        } else {
            MATRIX_ROTATEX_DETOUR.disable().unwrap();
            Matrix_RotateX(result, angle);
            MATRIX_ROTATEX_DETOUR.enable().unwrap();
        }
    }
}

thread_local!(
    static TICK: RefCell<TickEventHandler> = RefCell::new(TickEventHandler::new());
);

pub fn initialize() {
    unsafe {
        LOAD_MATRIX_DETOUR
            .initialize(Gfx_LoadMatrix, load_matrix)
            .unwrap();
        LOAD_MATRIX_DETOUR.enable().unwrap();

        MATRIX_MUL_DETOUR
            .initialize(Matrix_Mul, matrix_mul)
            .unwrap();
        MATRIX_MUL_DETOUR.enable().unwrap();

        MATRIX_ROTATEX_DETOUR
            .initialize(Matrix_RotateX, matrix_rotatex)
            .unwrap();
        MATRIX_ROTATEX_DETOUR.enable().unwrap();
    }

    TICK.with(|cell| {
        let tick = &mut *cell.borrow_mut();

        tick.on(|_| {
            log::debug!("tick");
            FIRST_IN_TICK.set(true);
        });
    });
}
