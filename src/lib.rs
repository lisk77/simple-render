mod animation;
mod input;
mod memory;
mod ui;
mod wayland;
mod widgets;

pub mod prelude {
    pub use crate::{
        Align, Anchor, Border, Button, ButtonStyle, Canvas, Checkbox, CheckboxStyle, Color,
        Content, Direction, FontCtx, FontCtxOptions, FrameAction, GradientDirection, Image,
        ImageFilter, ImageFit, InputAction, InputState, Key, KeyboardEvent, KeyboardEventKind,
        KeyboardInteractivity, Layer, LayerOptions, LazyFontCtx, Length, OutputTarget, Overflow,
        Paint, PaintTransform, Position, ProgressBar, ProgressBarStyle, Rect, RenderContext,
        Renderer, Result, RichText, Slider, SliderStyle, Style, Surface, Text, TextOverflow,
        TextRun, TextStyle, TextWrap, Toggle, ToggleStyle, Ui, UiContext, WidgetId,
    };
}

pub use animation::{
    Animation, AnimationFrame, BoundsTransition, BoundsTransitionFrame, Easing, Edge, Offset,
    VisualEffect, VisualTransition, VisualTransitionFrame, lerp_bounds, lerp_color, lerp_f32,
    lerp_i32, lerp_inset, lerp_spacing, lerp_u32,
};
pub use input::{
    InputState, Key, KeyState, KeyboardEvent, KeyboardEventKind, KeyboardModifiers, KeyboardState,
    PointerState, UiContext, WidgetId, WidgetInteraction,
};
pub use memory::{trim_free_heap_pages, tune_allocator_for_low_memory};
pub use ui::{
    Align, Border, BorderWidth, Bounds, Clip, Color, ColorParseError, Content, CornerRadius,
    Direction, DrawCommand, FontCtx, FontCtxOptions, FontSource, GradientDirection, Hit, Image,
    ImageFilter, ImageFit, ImagePixels, Inset, LazyFontCtx, Length, MeasuredSize, Overflow, Paint,
    PaintTransform, Position, Rect, RectLayout, RectStyle, RgbaImageSource, RichText, RoundedClip,
    Spacing, Style, Surface, Text, TextOverflow, TextRun, TextStyle, TextWrap, Ui,
};
pub use wayland::{
    Anchor, Canvas, DEFAULT_SURFACE_ID, DamageRect, FrameAction, InputAction,
    KeyboardInteractivity, Layer, LayerOptions, Margins, OutputTarget, PointerAxis,
    PointerAxisSource, PointerButtonState, PointerEvent, PointerEventKind, RenderCommand,
    RenderContext, RenderController, RenderOutput, RenderReceiver, RenderSendError, RenderSender,
    RenderSurfaceState, Renderer, Result, SurfaceId, channel, controller, run, run_surfaces,
};
pub use widgets::{
    Button, ButtonStyle, Checkbox, CheckboxStyle, ProgressBar, ProgressBarStyle, Slider,
    SliderStyle, Toggle, ToggleStyle, WidgetAction, WidgetValueAction,
};
