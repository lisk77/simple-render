mod animation;
mod input;
mod memory;
mod ui;
mod wayland;
mod widgets;

pub mod prelude {
    pub use crate::{
        Align, Anchor, Border, Button, ButtonStyle, ChangeEvent, Checkbox, CheckboxStyle,
        ClickEvent, Color, Content, Context, Direction, Element, ElementBuilder, GradientDirection,
        Image, ImageFit, Key, KeyboardEvent, KeyboardEventKind, KeyboardInteractivity, Layer,
        LayerOptions, LazyFontCtx, Length, Listener, OutputTarget, Overflow, Paint, PaintTransform,
        Pixels, PixelsExt, Position, ProgressBar, ProgressBarStyle, Rect, Render, Result, RichText,
        Slider, SliderStyle, Style, Surface, Text, TextOverflow, TextRun, TextStyle, TextWrap,
        Toggle, ToggleStyle, Ui, WidgetId,
    };
}

pub use animation::{
    Animation, AnimationFrame, BoundsTransition, BoundsTransitionFrame, Easing, Edge, Offset,
    VisualEffect, VisualTransition, VisualTransitionFrame, lerp_bounds, lerp_color, lerp_f32,
    lerp_i32, lerp_inset, lerp_spacing, lerp_u32,
};
pub use input::{
    InputState, Key, KeyState, KeyboardEvent, KeyboardEventKind, KeyboardModifiers, KeyboardState,
    PointerState, WidgetId, WidgetInteraction,
};
pub use memory::{trim_free_heap_pages, tune_allocator_for_low_memory};
pub use ui::{
    Align, Border, BorderWidth, Bounds, ChangeEvent, ClickEvent, Clip, Color, ColorParseError,
    Content, Context, CornerRadius, Direction, DrawCommand, Element, ElementBuilder, FontCtx,
    FontCtxOptions, FontSource, GradientDirection, Hit, Image, ImageFilter, ImageFit, ImagePixels,
    Inset, LazyFontCtx, Length, Listener, MeasuredSize, Overflow, Paint, PaintTransform, Pixels,
    PixelsExt, Position, Rect, RectLayout, RectStyle, Render, RgbaImageSource, RichText,
    RoundedClip, Spacing, Style, Surface, Text, TextOverflow, TextRun, TextStyle, TextWrap, Ui,
};
pub use wayland::{
    Anchor, Canvas, CanvasRenderer, DEFAULT_SURFACE_ID, DamageRect, FrameAction, InputAction,
    KeyboardInteractivity, Layer, LayerOptions, Margins, OutputTarget, PointerAxis,
    PointerAxisSource, PointerButtonState, PointerEvent, PointerEventKind, RenderCommand,
    RenderContext, RenderController, RenderOutput, RenderReceiver, RenderSendError, RenderSender,
    RenderSurfaceState, Result, SurfaceId, channel, controller, run, run_surfaces,
};
pub use widgets::{
    Button, ButtonStyle, Checkbox, CheckboxStyle, ProgressBar, ProgressBarStyle, Slider,
    SliderStyle, Toggle, ToggleStyle,
};
