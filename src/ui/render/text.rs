use super::geometry::{
    multiply_coverage, rounded_rect_bounds_coverage_with_antialias,
    rounded_rect_coverage_with_antialias,
};
use super::*;

mod decorations;
mod draw;
mod measure;

pub(super) use decorations::*;
pub(super) use draw::*;
pub(super) use measure::*;

#[derive(Clone, Copy)]
pub(super) enum TextContent<'a> {
    Plain(&'a Text),
    Rich(&'a RichText),
}

impl TextContent<'_> {
    fn is_empty(self) -> bool {
        match self {
            Self::Plain(text) => text.content.is_empty(),
            Self::Rich(text) => text.runs.iter().all(|run| run.content.is_empty()),
        }
    }

    fn align(self) -> Align {
        match self {
            Self::Plain(text) => text.align,
            Self::Rich(text) => text.align,
        }
    }

    fn vertical_align(self) -> Align {
        match self {
            Self::Plain(text) => text.vertical_align,
            Self::Rich(text) => text.vertical_align,
        }
    }

    fn wrap(self) -> TextWrap {
        match self {
            Self::Plain(text) => text.wrap,
            Self::Rich(text) => text.wrap,
        }
    }

    fn overflow(self) -> TextOverflow {
        match self {
            Self::Plain(text) => text.overflow,
            Self::Rich(text) => text.overflow,
        }
    }

    fn max_lines(self) -> Option<u32> {
        match self {
            Self::Plain(text) => text.max_lines,
            Self::Rich(text) => text.max_lines,
        }
    }

    fn default_style(&self) -> &TextStyle {
        match self {
            Self::Plain(text) => &text.style,
            Self::Rich(text) => text
                .runs
                .first()
                .map(|run| &run.style)
                .expect("rich text has at least one non-empty run"),
        }
    }
}
