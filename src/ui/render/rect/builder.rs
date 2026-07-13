use super::*;

impl Rect {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn layout(layout: RectLayout) -> Self {
        Self::default().with_layout(layout)
    }

    pub fn styled(style: RectStyle) -> Self {
        Self::empty().style(style)
    }

    pub fn with_layout(mut self, layout: RectLayout) -> Self {
        self.id = layout.id;
        if let Some(surface) = layout.surface {
            self.layer_options = surface.into();
        }
        self.width = layout.width;
        self.height = layout.height;
        self.min_width = layout.min_width;
        self.min_height = layout.min_height;
        self.max_width = layout.max_width;
        self.max_height = layout.max_height;
        self.fill = layout.fill;
        self.direction = layout.direction;
        self.align = layout.align;
        self.justify = layout.justify;
        self.overflow = layout.overflow;
        self.position = layout.position;
        self.inset = layout.inset;
        self.padding = layout.padding;
        self.gap = layout.gap;
        self.style = layout.style;
        if !layout.transform.is_identity() {
            self.style.transform = layout.transform;
        }
        self.content = layout.content;
        self
    }

    pub fn style(mut self, style: RectStyle) -> Self {
        self.style = style;
        self
    }

    pub fn id(mut self, id: impl Into<WidgetId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn on_click(mut self, listener: Listener<ClickEvent>) -> Self {
        self.on_click = Some(listener);
        self
    }

    pub fn on_drag(mut self, listener: Listener<ClickEvent>) -> Self {
        self.on_drag = Some(listener);
        self
    }

    pub fn on_key(mut self, listener: Listener<KeyboardEvent>) -> Self {
        self.on_key = Some(listener);
        self
    }

    pub(crate) fn click_listener(&self, path: &[usize]) -> Option<Listener<ClickEvent>> {
        let mut element = self;
        let mut listener = element.on_click.clone();
        for &index in path {
            let Some(child) = element.children.get(index) else {
                break;
            };
            element = child;
            if element.on_click.is_some() {
                listener = element.on_click.clone();
            }
        }
        listener
    }

    pub(crate) fn drag_listener(&self, path: &[usize]) -> Option<Listener<ClickEvent>> {
        let mut element = self;
        let mut listener = element.on_drag.clone();
        for &index in path {
            let Some(child) = element.children.get(index) else {
                break;
            };
            element = child;
            if element.on_drag.is_some() {
                listener = element.on_drag.clone();
            }
        }
        listener
    }

    pub(crate) fn key_listener(&self, path: &[usize]) -> Option<Listener<KeyboardEvent>> {
        let mut element = self;
        let mut listener = element.on_key.clone();
        for &index in path {
            let Some(child) = element.children.get(index) else {
                break;
            };
            element = child;
            if element.on_key.is_some() {
                listener = element.on_key.clone();
            }
        }
        listener
    }

    pub(crate) fn id_for_path(&self, path: &[usize]) -> Option<WidgetId> {
        let mut element = self;
        let mut id = element.id.clone();
        for &index in path {
            let Some(child) = element.children.get(index) else {
                break;
            };
            element = child;
            if element.id.is_some() {
                id = element.id.clone();
            }
        }
        id
    }

    pub(crate) fn path_for_id(&self, id: &WidgetId) -> Option<Vec<usize>> {
        fn find(element: &Rect, id: &WidgetId, path: &mut Vec<usize>) -> bool {
            if element.id.as_ref() == Some(id) {
                return true;
            }
            for (index, child) in element.children.iter().enumerate() {
                path.push(index);
                if find(child, id, path) {
                    return true;
                }
                path.pop();
            }
            false
        }
        let mut path = Vec::new();
        find(self, id, &mut path).then_some(path)
    }

    pub(crate) fn interaction_styles(
        mut self,
        hovered: Style,
        pressed: Style,
        disabled: Style,
    ) -> Self {
        self.interaction_styles = Some((hovered, pressed, disabled));
        self
    }

    pub(crate) fn interaction_child_styles(
        mut self,
        child: usize,
        hovered: Style,
        pressed: Style,
        disabled: Style,
    ) -> Self {
        self.interaction_styles = Some((hovered, pressed, disabled));
        self.interaction_style_child = Some(child);
        self
    }

    pub(crate) fn resolve_interaction_styles(
        &mut self,
        hovered: Option<&[usize]>,
        pressed: Option<&[usize]>,
    ) {
        fn visit(
            element: &mut Rect,
            path: &mut Vec<usize>,
            hovered: Option<&[usize]>,
            pressed: Option<&[usize]>,
        ) {
            if let Some((hovered_style, pressed_style, _)) = &element.interaction_styles {
                let style = if pressed.is_some_and(|target| target.starts_with(path)) {
                    Some(pressed_style.clone())
                } else if hovered.is_some_and(|target| target.starts_with(path)) {
                    Some(hovered_style.clone())
                } else {
                    None
                };
                if let Some(style) = style {
                    if let Some(index) = element.interaction_style_child {
                        if let Some(child) = element.children.get_mut(index) {
                            child.style = style;
                        }
                    } else {
                        element.style = style;
                    }
                }
            }
            for (index, child) in element.children.iter_mut().enumerate() {
                path.push(index);
                visit(child, path, hovered, pressed);
                path.pop();
            }
        }
        visit(self, &mut Vec::new(), hovered, pressed);
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn size(mut self, width: impl Into<Length>, height: impl Into<Length>) -> Self {
        self.width = width.into();
        self.height = height.into();
        self
    }

    pub fn min_width(mut self, min_width: impl Into<Pixels>) -> Self {
        self.min_width = min_width.into().get();
        self
    }

    pub fn min_height(mut self, min_height: impl Into<Pixels>) -> Self {
        self.min_height = min_height.into().get();
        self
    }

    pub fn max_width(mut self, max_width: impl Into<Option<u32>>) -> Self {
        self.max_width = max_width.into();
        self
    }

    pub fn max_height(mut self, max_height: impl Into<Option<u32>>) -> Self {
        self.max_height = max_height.into();
        self
    }

    pub fn fill(mut self, fill: u32) -> Self {
        self.fill = fill;
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn justify(mut self, justify: Align) -> Self {
        self.justify = justify;
        self
    }

    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    pub fn inset(mut self, inset: Inset) -> Self {
        self.inset = inset;
        self
    }

    pub fn inset_all(mut self, value: impl Into<Pixels>) -> Self {
        self.inset = Inset::all(value.into().get());
        self
    }

    pub fn inset_axis(
        mut self,
        horizontal: impl Into<Pixels>,
        vertical: impl Into<Pixels>,
    ) -> Self {
        self.inset = Inset::axis(horizontal.into().get(), vertical.into().get());
        self
    }

    pub fn inset_top(mut self, value: impl Into<Pixels>) -> Self {
        self.inset.top = Some(value.into().get());
        self
    }

    pub fn inset_right(mut self, value: impl Into<Pixels>) -> Self {
        self.inset.right = Some(value.into().get());
        self
    }

    pub fn inset_bottom(mut self, value: impl Into<Pixels>) -> Self {
        self.inset.bottom = Some(value.into().get());
        self
    }

    pub fn inset_left(mut self, value: impl Into<Pixels>) -> Self {
        self.inset.left = Some(value.into().get());
        self
    }

    pub fn absolute(mut self, inset: Inset) -> Self {
        self.position = Position::Absolute;
        self.inset = inset;
        self
    }

    pub fn absolute_all(mut self, value: impl Into<Pixels>) -> Self {
        self.position = Position::Absolute;
        self.inset = Inset::all(value.into().get());
        self
    }

    pub fn absolute_axis(
        mut self,
        horizontal: impl Into<Pixels>,
        vertical: impl Into<Pixels>,
    ) -> Self {
        self.position = Position::Absolute;
        self.inset = Inset::axis(horizontal.into().get(), vertical.into().get());
        self
    }

    pub fn top(mut self, value: impl Into<Pixels>) -> Self {
        self.position = Position::Absolute;
        self.inset.top = Some(value.into().get());
        self
    }

    pub fn right(mut self, value: impl Into<Pixels>) -> Self {
        self.position = Position::Absolute;
        self.inset.right = Some(value.into().get());
        self
    }

    pub fn bottom(mut self, value: impl Into<Pixels>) -> Self {
        self.position = Position::Absolute;
        self.inset.bottom = Some(value.into().get());
        self
    }

    pub fn left(mut self, value: impl Into<Pixels>) -> Self {
        self.position = Position::Absolute;
        self.inset.left = Some(value.into().get());
        self
    }

    pub fn padding(mut self, padding: Spacing) -> Self {
        self.padding = padding;
        self
    }

    pub fn padding_top(mut self, value: impl Into<Pixels>) -> Self {
        self.padding.top = value.into().get();
        self
    }

    pub fn padding_right(mut self, value: impl Into<Pixels>) -> Self {
        self.padding.right = value.into().get();
        self
    }

    pub fn padding_bottom(mut self, value: impl Into<Pixels>) -> Self {
        self.padding.bottom = value.into().get();
        self
    }

    pub fn padding_left(mut self, value: impl Into<Pixels>) -> Self {
        self.padding.left = value.into().get();
        self
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into().get();
        self
    }

    pub fn content(mut self, content: impl Into<Content>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn with_surface(mut self, surface: Surface) -> Self {
        self.layer_options = surface.into();
        self
    }

    pub fn surface(self, surface: Surface) -> Self {
        self.with_surface(surface)
    }
}
