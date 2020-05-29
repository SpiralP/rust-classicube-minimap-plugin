#![allow(non_snake_case)]

use super::bmp;
use classicube_helpers::{detour::static_detour, CellGetSet};
use classicube_sys::*;
use log::*;
use std::{cell::Cell, os::raw::*};

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
        // debug!("load_matrix {:?} {:?}", type_, matrix);

        if type_ == MatrixType__MATRIX_VIEW {
            if let Some(iso_transform) = ISO_TRANSFORM.get() {
                if matrix == iso_transform && FIRST_IN_RENDER.get() {
                    FIRST_IN_RENDER.set(false);

                    bmp::update();
                    bmp::draw();
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

            info!("FOUND ISO_TRANSFORM");

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
