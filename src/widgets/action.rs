pub trait WidgetAction<S> {
    fn call(self, state: &mut S);
}

impl<S> WidgetAction<S> for () {
    fn call(self, _: &mut S) {}
}

impl<S, F> WidgetAction<S> for F
where
    F: FnOnce(&mut S),
{
    fn call(self, state: &mut S) {
        self(state);
    }
}

pub trait WidgetValueAction<S, T> {
    fn call(self, state: &mut S, value: T);
}

impl<S, T> WidgetValueAction<S, T> for () {
    fn call(self, _: &mut S, _: T) {}
}

impl<S, T, F> WidgetValueAction<S, T> for F
where
    F: FnOnce(&mut S, T),
{
    fn call(self, state: &mut S, value: T) {
        self(state, value);
    }
}
