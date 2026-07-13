use crate::Bounds;
use std::rc::Rc;

#[derive(Clone)]
pub struct Listener<E> {
    callback: Rc<dyn Fn(&E)>,
}

impl<E> Listener<E> {
    pub fn new(callback: impl Fn(&E) + 'static) -> Self {
        Self {
            callback: Rc::new(callback),
        }
    }
    pub(crate) fn call(&self, event: &E) {
        (self.callback)(event);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClickEvent {
    pub x: f64,
    pub y: f64,
    pub bounds: Bounds,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChangeEvent<T> {
    pub value: T,
}
