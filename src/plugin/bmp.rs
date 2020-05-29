#![allow(non_snake_case)]

use crate::helpers::Texture_RenderShaded;
use classicube_sys::*;
use std::{cell::RefCell, slice};

const WHITE_TRANSPARENT: PackedCol = PackedCol_Make(255, 255, 255, 0);
const TEXTURE_WIDTH: usize = 1024;
const TEXTURE_HEIGHT: usize = 1024;

thread_local!(
    static TEXTURE: RefCell<Option<OwnedGfxTexture>> = Default::default();
);

pub fn init() {
    TEXTURE.with(|cell| {
        let opt = &mut *cell.borrow_mut();

        // must be a vec or else we try to fit huge array onto stack and crash!
        let mut pixels: Vec<u8> = vec![255; 4 * TEXTURE_WIDTH * TEXTURE_HEIGHT];

        let mut bmp = Bitmap {
            Scan0: pixels.as_mut_ptr(),
            Width: TEXTURE_WIDTH as i32,
            Height: TEXTURE_HEIGHT as i32,
        };

        *opt = Some(OwnedGfxTexture::create(&mut bmp, true, false));
    });
}

pub fn free() {
    TEXTURE.with(|cell| {
        let opt = &mut *cell.borrow_mut();

        drop(opt.take());
    });
}

fn update_texture(bmp: &mut Bitmap) {
    TEXTURE.with(|cell| {
        let opt = &mut *cell.borrow_mut();
        let owned_texture = opt.as_mut().unwrap();

        unsafe {
            Gfx_UpdateTexturePart(owned_texture.resource_id, 0, 0, bmp, 0);
        }
    });
}

pub fn draw() {
    TEXTURE.with(|cell| {
        let opt = &*cell.borrow();
        let owned_texture = opt.as_ref().unwrap();

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

pub fn update() {
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
                    // debug!("{}", atlas_id);
                    let first_pixel_index = atlas_id as usize * Atlas2D.TileSize as usize;
                    // let random_pixel =
                    //     first_pixel_index + rng.gen_range(0, Atlas2D.TileSize as usize);
                    // debug!("{}", first_pixel_index);
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

// const FACE_XMIN: usize = 0; // Face X = 0
// const FACE_XMAX: usize = 1; // Face X = 1
// const FACE_ZMIN: usize = 2; // Face Z = 0
// const FACE_ZMAX: usize = 3; // Face Z = 1
// const FACE_YMIN: usize = 4; // Face Y = 0
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
