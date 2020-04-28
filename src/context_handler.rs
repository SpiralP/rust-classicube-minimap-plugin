use crate::helpers::*;
use classicube_helpers::events::gfx::{ContextLostEventHandler, ContextRecreatedEventHandler};
use classicube_sys::{OwnedGfxVertexBuffer, VertexFormat__VERTEX_FORMAT_P3FT2FC4B};
use std::cell::RefCell;

thread_local!(
    static CONTEXT_LOST_HANDLER: RefCell<ContextLostEventHandler> =
        RefCell::new(ContextLostEventHandler::new());
);

thread_local!(
    static CONTEXT_RECREATED_HANDLER: RefCell<ContextRecreatedEventHandler> =
        RefCell::new(ContextRecreatedEventHandler::new());
);

pub fn initialize() {
    CONTEXT_RECREATED_HANDLER.with(|cell| {
        let context_recreated_handler = &mut *cell.borrow_mut();

        context_recreated_handler.on(|_| {
            log::debug!("Context Recreated");

            // create texture buffer
            TEX_VB.with(|cell| {
                *cell.borrow_mut() = Some(OwnedGfxVertexBuffer::create(
                    VertexFormat__VERTEX_FORMAT_P3FT2FC4B,
                    4,
                ));
            });
        });
    });

    CONTEXT_LOST_HANDLER.with(|cell| {
        let context_lost_handler = &mut *cell.borrow_mut();

        context_lost_handler.on(|_| {
            log::debug!("Context Lost");

            // delete texture buffer
            TEX_VB.with(|cell| {
                cell.borrow_mut().take();
            });
        });
    });
}

// TODO shutdown
