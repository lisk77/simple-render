use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{FontCtx, Rect};
use crate::{
    CanvasRenderer, ClickEvent, Context, Element, FrameAction, InputAction, InputState,
    PointerButtonState, PointerEvent, PointerEventKind, RenderContext,
};

pub trait Render: Sized + 'static {
    fn ui(&mut self, cx: &mut Context<Self>) -> Element;
}

pub(crate) struct ManagedRenderer<S> {
    state: Rc<RefCell<S>>,
    input: InputState,
    fonts: FontCtx,
    root: Option<Rect>,
    focused_path: Option<Vec<usize>>,
    focused_id: Option<crate::WidgetId>,
    focus_request: Rc<RefCell<Option<Option<crate::WidgetId>>>>,
    close_request: Rc<Cell<bool>>,
}

impl<S: Render> ManagedRenderer<S> {
    pub(crate) fn new(state: S) -> Self {
        Self {
            state: Rc::new(RefCell::new(state)),
            input: InputState::new(),
            fonts: FontCtx::new(),
            root: None,
            focused_path: None,
            focused_id: None,
            focus_request: Rc::new(RefCell::new(None)),
            close_request: Rc::new(Cell::new(false)),
        }
    }
}

impl<S: Render> CanvasRenderer for ManagedRenderer<S> {
    fn draw(&mut self, canvas: &mut crate::Canvas<'_>, context: RenderContext) -> FrameAction {
        let mut cx = Context::new(
            Rc::downgrade(&self.state),
            self.focus_request.clone(),
            self.close_request.clone(),
        );
        let mut root = self.state.borrow_mut().ui(&mut cx).into_rect();
        if self.close_request.get() {
            return FrameAction::Exit;
        }
        if let Some(request) = self.focus_request.borrow_mut().take() {
            self.focused_id = request;
            self.focused_path = self.focused_id.as_ref().and_then(|id| root.path_for_id(id));
        }
        if let Some(id) = &self.focused_id {
            self.focused_path = root.path_for_id(id);
        }
        self.input
            .resolve_hover_for_context(&root, context, &mut self.fonts);
        root.resolve_interaction_styles(
            self.input
                .pointer()
                .hovered
                .as_ref()
                .map(|hit| hit.path.as_slice()),
            self.input
                .pointer()
                .pressed
                .as_ref()
                .map(|hit| hit.path.as_slice()),
        );
        canvas.clear(crate::Color::TRANSPARENT.into());
        root.paint_scaled_with_fonts(
            canvas,
            &mut self.fonts,
            context.width,
            context.height,
            context.scale,
        );
        self.input.clear_frame_events();
        self.root = Some(root);
        FrameAction::Wait
    }

    fn idle(&mut self) {
        self.fonts.trim_frame_memory();
    }

    fn pointer_event(&mut self, event: PointerEvent) -> InputAction {
        let pressed_before = self.input.pointer().pressed.clone();
        let released = matches!(
            event.kind,
            PointerEventKind::Button {
                state: PointerButtonState::Released,
                ..
            }
        );
        self.input.handle_pointer_event(event);
        let dragging = matches!(
            event.kind,
            PointerEventKind::Motion
                | PointerEventKind::Button {
                    state: PointerButtonState::Pressed,
                    ..
                }
        );
        if dragging
            && let Some(hit) = self
                .input
                .pointer()
                .pressed
                .clone()
                .or_else(|| self.input.pointer().hovered.clone())
            && let Some(listener) = self
                .root
                .as_ref()
                .and_then(|root| root.drag_listener(&hit.path))
        {
            listener.call(&ClickEvent {
                x: event.x,
                y: event.y,
                bounds: hit.bounds,
            });
        }
        if released
            && let Some(hit) = self.input.pointer().hovered.clone()
            && pressed_before
                .as_ref()
                .is_some_and(|pressed| pressed.path == hit.path)
        {
            self.focused_path = Some(hit.path.clone());
            self.focused_id = self
                .root
                .as_ref()
                .and_then(|root| root.id_for_path(&hit.path));
            if let Some(listener) = self
                .root
                .as_ref()
                .and_then(|root| root.click_listener(&hit.path))
            {
                listener.call(&ClickEvent {
                    x: event.x,
                    y: event.y,
                    bounds: hit.bounds,
                });
            }
        }
        if self.close_request.get() {
            InputAction::Exit
        } else {
            InputAction::Redraw
        }
    }

    fn keyboard_event(&mut self, event: crate::KeyboardEvent) -> InputAction {
        self.input.handle_keyboard_event(event.clone());
        if let Some(listener) = self
            .focused_path
            .as_deref()
            .and_then(|path| self.root.as_ref().and_then(|root| root.key_listener(path)))
        {
            listener.call(&event);
        }
        if self.close_request.get() {
            InputAction::Exit
        } else {
            InputAction::Redraw
        }
    }
}
