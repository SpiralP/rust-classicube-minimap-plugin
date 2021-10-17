use super::helpers::*;
use classicube_helpers::events::gfx::{ContextLostEventHandler, ContextRecreatedEventHandler};
use classicube_sys::{OwnedGfxVertexBuffer, VertexFormat__VERTEX_FORMAT_TEXTURED};
use tracing::debug;
use std::cell::Cell;

pub struct ContextHandler {
    context_lost_handler: ContextLostEventHandler,
    context_recreated_handler: ContextRecreatedEventHandler,
}

impl ContextHandler {
    pub fn new() -> Self {
        Self {
            context_lost_handler: ContextLostEventHandler::new(),
            context_recreated_handler: ContextRecreatedEventHandler::new(),
        }
    }

    fn context_recreated() {
        // create texture buffer

        TEX_VB.with(|cell| {
            *cell.borrow_mut() = Some(OwnedGfxVertexBuffer::create(
                VertexFormat__VERTEX_FORMAT_TEXTURED,
                4,
            ));
        });
    }

    fn context_lost() {
        // delete texture buffer

        TEX_VB.with(|cell| {
            cell.borrow_mut().take();
        });
    }

    pub fn initialize(&mut self) {
        // we start with context created
        Self::context_recreated();

        self.context_lost_handler.on(|_| {
            debug!("ContextLost");

            Self::context_lost();
        });

        self.context_recreated_handler.on(|_| {
            debug!("ContextRecreated");

            Self::context_recreated();
        });
    }

    pub fn shutdown(&mut self) {
        Self::context_lost();
    }
}

thread_local!(
    static CONTEXT_HANDLER: Cell<Option<ContextHandler>> = Default::default();
);

pub fn initialize() {
    CONTEXT_HANDLER.with(|cell| {
        let mut context_handler = ContextHandler::new();
        context_handler.initialize();
        cell.set(Some(context_handler));
    });
}

pub fn shutdown() {
    CONTEXT_HANDLER.with(|cell| {
        if let Some(mut context_handler) = cell.take() {
            context_handler.shutdown();
        }
    });
}
