use super::{InputState, WidgetId, WidgetInteraction};

pub struct UiContext<'a, S> {
    state: &'a mut S,
    input: &'a mut InputState,
    actions_enabled: bool,
    changed: bool,
}

impl<'a, S> UiContext<'a, S> {
    pub fn new(state: &'a mut S, input: &'a mut InputState) -> Self {
        Self {
            state,
            input,
            actions_enabled: true,
            changed: false,
        }
    }

    pub fn layout(state: &'a mut S, input: &'a mut InputState) -> Self {
        Self {
            state,
            input,
            actions_enabled: false,
            changed: false,
        }
    }

    pub fn state(&self) -> &S {
        self.state
    }
    pub fn state_mut(&mut self) -> &mut S {
        self.state
    }
    pub fn input(&self) -> &InputState {
        self.input
    }
    pub fn input_mut(&mut self) -> &mut InputState {
        self.input
    }
    pub fn interaction(&self, id: impl Into<WidgetId>) -> WidgetInteraction {
        self.input.interaction(id)
    }
    pub fn actions_enabled(&self) -> bool {
        self.actions_enabled
    }
    pub fn changed(&self) -> bool {
        self.changed
    }
    pub fn mark_changed(&mut self) {
        self.changed = true;
    }
    pub fn consume_click(&mut self, id: &WidgetId) -> bool {
        self.actions_enabled && self.input.consume_click(id)
    }
    pub fn set_focus(&mut self, id: impl Into<WidgetId>) {
        self.input.set_focus(id);
    }
    pub fn clear_focus(&mut self) {
        self.input.clear_focus();
    }
}
