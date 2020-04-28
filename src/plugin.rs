#![allow(non_snake_case)]

use crate::helpers::Texture_RenderShaded;
use classicube_helpers::{detour::static_detour, CellGetSet};
use classicube_sys::*;
use std::{
    cell::{Cell, RefCell},
    os::raw::*,
    slice,
};

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

fn update_texture(bmp: &mut Bitmap) {
    TEXTURE.with(|cell| {
        let owned_texture = &mut *cell.borrow_mut();

        unsafe {
            Gfx_UpdateTexturePart(owned_texture.resource_id, 0, 0, bmp, 0);
        }
    });
}

fn draw() {
    TEXTURE.with(|cell| {
        let owned_texture = &*cell.borrow();

        unsafe {
            let me = &*Entities.List[ENTITIES_SELF_ID as usize];
            let p = me.Position;
            let u = (p.X.max(0.0).min(127.0) / 128f32).max(0.0).min(1.0);
            let v = (p.Z.max(0.0).min(127.0) / 128f32).max(0.0).min(1.0);

            let mut texture = Texture {
                ID: owned_texture.resource_id,
                X: 640,
                Y: 20,
                Width: 200 as _,
                Height: 200 as _,
                uv: TextureRec {
                    U1: ((68f32 * u) / 1024f32),
                    V1: ((68f32 * v) / 1024f32),
                    U2: ((68f32 * (1.0 + u)) / 1024f32),
                    V2: ((68f32 * (1.0 + v)) / 1024f32),
                },
            };

            Texture_RenderShaded(&mut texture, WHITE_TRANSPARENT);
        }
    });
}

thread_local!(
    static FIRST_IN_RENDER: Cell<bool> = Cell::new(false);
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
                if matrix == iso_transform && FIRST_IN_RENDER.get() {
                    FIRST_IN_RENDER.set(false);

                    doot();
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

static_detour! {
    static LOCAL_PLAYER_RENDER_MODEL_DETOUR: unsafe extern "C" fn(*mut Entity, c_double, c_float);
}

/// This is called when LocalPlayer_RenderModel is called.
fn render_model(local_player_entity: *mut Entity, delta: c_double, t: c_float) {
    unsafe {
        LOCAL_PLAYER_RENDER_MODEL_DETOUR.call(local_player_entity, delta, t);
    }

    FIRST_IN_RENDER.set(true);
}

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

        let me = &*Entities.List[ENTITIES_SELF_ID as usize];
        let v_table = &*me.VTABLE;
        let target = v_table.RenderModel.unwrap();

        LOCAL_PLAYER_RENDER_MODEL_DETOUR
            .initialize(target, render_model)
            .unwrap();
        LOCAL_PLAYER_RENDER_MODEL_DETOUR.enable().unwrap();
    }
}

fn doot() {
    let width = 128;
    let height = 128;

    let mut pixels: Vec<u8> = vec![255; 4 * width * height];

    unsafe {
        let me = &*Entities.List[ENTITIES_SELF_ID as usize];
        // me.Position

        let atlas_pixels = slice::from_raw_parts_mut(
            Atlas2D.Bmp.Scan0,
            (4 * Atlas2D.Bmp.Width * Atlas2D.Bmp.Height) as usize,
        );

        for x in 0..width {
            for z in 0..height {
                for y in (0..64).rev() {
                    let block_id = World_GetBlock(x as i32, y as i32, z as i32) as usize;

                    if block_id == 0 {
                        continue;
                    }

                    let tex_index = block_id * FACE_COUNT;
                    let tex_loc = Blocks.Textures[tex_index + FACE_YMAX] as usize;

                    let atlas_x = Atlas2D_TileX(tex_loc);
                    let atlas_y = Atlas2D_TileY(tex_loc);
                    let atlas_id = atlas_x + atlas_y * ATLAS2D_TILES_PER_ROW;
                    // log::debug!("{}", atlas_id);
                    let first_pixel_index = atlas_id as usize * Atlas2D.TileSize as usize;
                    // let random_pixel =
                    //     first_pixel_index + rng.gen_range(0, Atlas2D.TileSize as usize);
                    // log::debug!("{}", first_pixel_index);
                    let first_color_of_tile =
                        &atlas_pixels[first_pixel_index * 4..(first_pixel_index * 4 + 4)];

                    let pixels_index = x + z * width;
                    let p = &mut pixels[pixels_index * 4..pixels_index * 4 + 4];

                    p[0] = first_color_of_tile[0];
                    p[1] = first_color_of_tile[1];
                    p[2] = first_color_of_tile[2];
                    p[3] = 255; // alpha

                    break;
                }
            }
        }

        let x = me.Position.X.max(0.0).min(127.0) as usize;
        let z = me.Position.Z.max(0.0).min(127.0) as usize;

        let pixels_index = x + z * width;
        let p = &mut pixels[pixels_index * 4..pixels_index * 4 + 4];

        p[0] = 255;
        p[1] = 255;
        p[2] = 255;
        p[3] = 255;
    }

    let mut bmp = Bitmap {
        Scan0: pixels.as_mut_ptr(),
        Width: width as i32,
        Height: height as i32,
    };

    update_texture(&mut bmp);
}
// for (i = 0; i < Array_Elems(Blocks.Textures); i++) {
//     maxLoc = max(maxLoc, Blocks.Textures[i]);
// }
// return Atlas1D_Index(maxLoc) + 1;

const FACE_XMIN: usize = 0; // Face X = 0
const FACE_XMAX: usize = 1; // Face X = 1
const FACE_ZMIN: usize = 2; // Face Z = 0
const FACE_ZMAX: usize = 3; // Face Z = 1
const FACE_YMIN: usize = 4; // Face Y = 0
const FACE_YMAX: usize = 5; // Face Y = 1
const FACE_COUNT: usize = 6; // Number of faces on a cube

const ATLAS2D_TILES_PER_ROW: usize = 16;
const ATLAS2D_MASK: usize = 15;
const ATLAS2D_SHIFT: usize = 4;

fn Atlas2D_TileX(tex_loc: usize) -> usize {
    tex_loc & ATLAS2D_MASK
}
fn Atlas2D_TileY(tex_loc: usize) -> usize {
    tex_loc >> ATLAS2D_SHIFT
}

// #define Atlas2D_TileX(texLoc) ((texLoc) &  ATLAS2D_MASK)  /* texLoc % ATLAS2D_TILES_PER_ROW */
// #define Atlas2D_TileY(texLoc) ((texLoc) >> ATLAS2D_SHIFT) /* texLoc / ATLAS2D_TILES_PER_ROW */
// /* Returns the index of the given tile id within a 1D atlas */
// #define Atlas1D_RowId(texLoc) ((texLoc)  & Atlas1D.Mask)  /* texLoc % Atlas1D_TilesPerAtlas */
// /* Returns the index of the 1D atlas within the array of 1D atlases that contains the given tile id */
// #define Atlas1D_Index(texLoc) ((texLoc) >> Atlas1D.Shift) /* texLoc / Atlas1D_TilesPerAtlas */
