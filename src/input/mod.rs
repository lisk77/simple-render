use std::{fmt, sync::Arc};

use crate::{
    ui::{Bounds, FontCtx, Hit, Rect},
    wayland::{PointerButtonState, PointerEvent, PointerEventKind, RenderContext, SurfaceId},
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetId(Arc<str>);

impl WidgetId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WidgetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<&'static str> for WidgetId {
    fn from(id: &'static str) -> Self {
        Self::new(id)
    }
}

impl From<String> for WidgetId {
    fn from(id: String) -> Self {
        Self::new(Arc::<str>::from(id))
    }
}

impl From<Arc<str>> for WidgetId {
    fn from(id: Arc<str>) -> Self {
        Self::new(id)
    }
}

impl From<u64> for WidgetId {
    fn from(id: u64) -> Self {
        Self::from(id.to_string())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyboardModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub caps_lock: bool,
    pub logo: bool,
    pub num_lock: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyboardEventKind {
    Enter {
        pressed_keycodes: Vec<u32>,
        pressed_keysyms: Vec<u32>,
    },
    Leave,
    Key {
        state: KeyState,
        raw_code: u32,
        keysym: u32,
        utf8: Option<String>,
    },
    Modifiers {
        modifiers: KeyboardModifiers,
        layout: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardEvent {
    pub surface: Option<SurfaceId>,
    pub time: Option<u32>,
    pub serial: Option<u32>,
    pub kind: KeyboardEventKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PointerState {
    pub surface: Option<SurfaceId>,
    pub position: Option<(f64, f64)>,
    pub hovered: Option<Hit>,
    pub pressed: Option<Hit>,
    pub pressed_button: Option<u32>,
    pub scroll_delta: (f64, f64),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyboardState {
    pub focused: Option<WidgetId>,
    pub surface: Option<SurfaceId>,
    pub modifiers: KeyboardModifiers,
    pub events: Vec<KeyboardEvent>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetInteraction {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub focused: bool,
    pub disabled: bool,
}

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputState {
    pointer: PointerState,
    keyboard: KeyboardState,
    active: Option<WidgetId>,
    hot: Option<WidgetId>,
    clicked: Option<WidgetId>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pointer(&self) -> &PointerState {
        &self.pointer
    }

    pub fn keyboard(&self) -> &KeyboardState {
        &self.keyboard
    }

    pub fn hot(&self) -> Option<&WidgetId> {
        self.hot.as_ref()
    }

    pub fn active(&self) -> Option<&WidgetId> {
        self.active.as_ref()
    }

    pub fn focused(&self) -> Option<&WidgetId> {
        self.keyboard.focused.as_ref()
    }

    pub fn clicked_id(&self) -> Option<&WidgetId> {
        self.clicked.as_ref()
    }

    pub fn set_focus(&mut self, id: impl Into<WidgetId>) {
        self.keyboard.focused = Some(id.into());
    }

    pub fn clear_focus(&mut self) {
        self.keyboard.focused = None;
    }

    pub fn clear_frame_events(&mut self) {
        self.clicked = None;
        self.pointer.scroll_delta = (0.0, 0.0);
        self.keyboard.events.clear();
    }

    pub fn handle_pointer_event(&mut self, event: PointerEvent) -> bool {
        match event.kind {
            PointerEventKind::Enter | PointerEventKind::Motion => {
                let position = Some((event.x, event.y));
                let changed = self.pointer.surface != Some(event.surface)
                    || self.pointer.position != position;
                self.pointer.surface = Some(event.surface);
                self.pointer.position = position;
                changed
            }
            PointerEventKind::Leave => {
                let changed = self.pointer.surface.is_some()
                    || self.pointer.position.is_some()
                    || self.hot.is_some()
                    || self.pointer.hovered.is_some();
                self.pointer.surface = None;
                self.pointer.position = None;
                self.pointer.hovered = None;
                self.hot = None;
                changed
            }
            PointerEventKind::Button {
                button,
                state: PointerButtonState::Pressed,
            } => {
                self.pointer.pressed = self.pointer.hovered.clone();
                self.pointer.pressed_button = Some(button);
                self.active = self.hot.clone();
                true
            }
            PointerEventKind::Button {
                button,
                state: PointerButtonState::Released,
            } => {
                if self.pointer.pressed_button == Some(button) && self.active == self.hot {
                    self.clicked = self.active.clone();
                }
                self.pointer.pressed = None;
                self.pointer.pressed_button = None;
                self.active = None;
                true
            }
            PointerEventKind::Axis {
                horizontal,
                vertical,
                ..
            } => {
                self.pointer.scroll_delta.0 += horizontal.absolute;
                self.pointer.scroll_delta.1 += vertical.absolute;
                true
            }
        }
    }

    pub fn handle_keyboard_event(&mut self, event: KeyboardEvent) -> bool {
        match &event.kind {
            KeyboardEventKind::Enter { .. } => {
                self.keyboard.surface = event.surface;
            }
            KeyboardEventKind::Leave => {
                if self.keyboard.surface == event.surface {
                    self.keyboard.surface = None;
                    self.keyboard.focused = None;
                }
            }
            KeyboardEventKind::Modifiers { modifiers, .. } => {
                self.keyboard.modifiers = *modifiers;
            }
            KeyboardEventKind::Key { .. } => {}
        }
        self.keyboard.events.push(event);
        true
    }

    pub fn resolve_hover(&mut self, ui: &Rect, bounds: Bounds, fonts: &mut FontCtx) {
        let Some((x, y)) = self.pointer.position else {
            self.pointer.hovered = None;
            self.hot = None;
            return;
        };

        self.pointer.hovered = ui.hit_test_with_fonts(bounds, x, y, fonts);
        self.hot = self.pointer.hovered.as_ref().and_then(|hit| hit.id.clone());
    }

    pub fn resolve_hover_for_context(
        &mut self,
        ui: &Rect,
        context: RenderContext,
        fonts: &mut FontCtx,
    ) {
        self.resolve_hover(ui, Bounds::new(0, 0, context.width, context.height), fonts);
    }

    pub fn interaction(&self, id: impl Into<WidgetId>) -> WidgetInteraction {
        let id = id.into();
        WidgetInteraction {
            hovered: self.hot.as_ref() == Some(&id),
            pressed: self.active.as_ref() == Some(&id),
            clicked: self.clicked.as_ref() == Some(&id),
            focused: self.keyboard.focused.as_ref() == Some(&id),
            disabled: false,
        }
    }

    pub fn consume_click(&mut self, id: &WidgetId) -> bool {
        if self.clicked.as_ref() == Some(id) {
            self.clicked = None;
            true
        } else {
            false
        }
    }

    pub fn is_hovered(&self, id: impl Into<WidgetId>) -> bool {
        let id = id.into();
        self.hot.as_ref() == Some(&id)
    }

    pub fn is_pressed(&self, id: impl Into<WidgetId>) -> bool {
        let id = id.into();
        self.active.as_ref() == Some(&id)
    }

    pub fn was_clicked(&self, id: impl Into<WidgetId>) -> bool {
        let id = id.into();
        self.clicked.as_ref() == Some(&id)
    }
}
