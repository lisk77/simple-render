use super::*;

impl Layer {
    pub(in crate::wayland) fn into_sctk(self) -> SctkLayer {
        match self {
            Self::Background => SctkLayer::Background,
            Self::Bottom => SctkLayer::Bottom,
            Self::Top => SctkLayer::Top,
            Self::Overlay => SctkLayer::Overlay,
        }
    }
}

impl KeyboardInteractivity {
    pub(in crate::wayland) fn into_sctk(self) -> SctkKeyboardInteractivity {
        match self {
            Self::None => SctkKeyboardInteractivity::None,
            Self::Exclusive => SctkKeyboardInteractivity::Exclusive,
            Self::OnDemand => SctkKeyboardInteractivity::OnDemand,
        }
    }
}

pub(in crate::wayland) fn apply_placement(layer: &LayerSurface, anchor: Anchor, margins: Margins) {
    let (anchor, margins) = placement(anchor, margins);
    layer.set_anchor(anchor);
    layer.set_margin(margins.top, margins.right, margins.bottom, margins.left);
}

fn placement(anchor: Anchor, margins: Margins) -> (SctkAnchor, Margins) {
    match anchor {
        Anchor::Position { x, y } => (
            SctkAnchor::TOP | SctkAnchor::LEFT,
            Margins {
                top: y,
                right: 0,
                bottom: 0,
                left: x,
            },
        ),
        anchor => (anchor.into_sctk(), margins),
    }
}

impl Anchor {
    fn into_sctk(self) -> SctkAnchor {
        match self {
            Self::Center => SctkAnchor::empty(),
            Self::Top => SctkAnchor::TOP,
            Self::TopLeft => SctkAnchor::TOP | SctkAnchor::LEFT,
            Self::TopRight => SctkAnchor::TOP | SctkAnchor::RIGHT,
            Self::Bottom => SctkAnchor::BOTTOM,
            Self::BottomLeft => SctkAnchor::BOTTOM | SctkAnchor::LEFT,
            Self::BottomRight => SctkAnchor::BOTTOM | SctkAnchor::RIGHT,
            Self::Left => SctkAnchor::LEFT,
            Self::Right => SctkAnchor::RIGHT,
            Self::Fill => {
                SctkAnchor::TOP | SctkAnchor::RIGHT | SctkAnchor::BOTTOM | SctkAnchor::LEFT
            }
            Self::Position { .. } => SctkAnchor::TOP | SctkAnchor::LEFT,
        }
    }
}
