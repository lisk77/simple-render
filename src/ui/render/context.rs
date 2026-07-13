use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use super::{Listener, WidgetId};

pub struct Context<S> {
    state: Weak<RefCell<S>>,
    focus_request: Rc<RefCell<Option<Option<WidgetId>>>>,
    close_request: Rc<Cell<bool>>,
}

impl<S: 'static> Context<S> {
    pub(crate) fn new(
        state: Weak<RefCell<S>>,
        focus_request: Rc<RefCell<Option<Option<WidgetId>>>>,
        close_request: Rc<Cell<bool>>,
    ) -> Self {
        Self {
            state,
            focus_request,
            close_request,
        }
    }

    pub fn listener<E: 'static>(&self, callback: impl Fn(&mut S, &E) + 'static) -> Listener<E> {
        let state = self.state.clone();
        Listener::new(move |event| {
            if let Some(state) = state.upgrade() {
                callback(&mut state.borrow_mut(), event);
            }
        })
    }

    pub fn focus(&self, id: impl Into<WidgetId>) {
        *self.focus_request.borrow_mut() = Some(Some(id.into()));
    }

    pub fn clear_focus(&self) {
        *self.focus_request.borrow_mut() = Some(None);
    }

    pub fn close<E>(&self) -> Listener<E> {
        let close_request = self.close_request.clone();
        Listener::new(move |_| close_request.set(true))
    }
}
