use super::*;

impl Rect {
    pub fn draw(self) -> wayland::Result<()> {
        let options = self.layer_options.clone();
        options.show(UiRenderer {
            root: self,
            fonts: LazyFontCtx::new(),
        })
    }

    pub fn draw_with_commands(self, receiver: wayland::RenderReceiver) -> wayland::Result<()> {
        let options = self.layer_options.clone();
        options.show_with_commands(
            UiRenderer {
                root: self,
                fonts: LazyFontCtx::new(),
            },
            receiver,
        )
    }

    pub fn commands(&self, bounds: Bounds) -> Vec<DrawCommand> {
        let mut fonts = FontCtx::new();
        self.commands_with_fonts(bounds, &mut fonts)
    }

    pub fn commands_with_fonts(&self, bounds: Bounds, fonts: &mut FontCtx) -> Vec<DrawCommand> {
        let mut commands = Vec::new();
        Self::visit_layout(
            self,
            bounds,
            Clip::rect(bounds),
            VisualState::IDENTITY,
            1.0,
            None,
            fonts,
            &mut |command, _| {
                commands.push(command.to_owned());
            },
        );
        fonts.trim_scratch();
        commands
    }

    pub fn visual_bounds(&self, bounds: Bounds) -> Option<Bounds> {
        let mut fonts = FontCtx::new();
        self.visual_bounds_with_fonts(bounds, &mut fonts)
    }

    pub fn visual_bounds_with_fonts(&self, bounds: Bounds, fonts: &mut FontCtx) -> Option<Bounds> {
        let mut visual_bounds: Option<Bounds> = None;
        Self::visit_layout(
            self,
            bounds,
            Clip::rect(bounds),
            VisualState::IDENTITY,
            1.0,
            None,
            fonts,
            &mut |command, _| {
                let bounds = command.rect();
                visual_bounds = Some(match visual_bounds {
                    Some(current) => current.union(bounds),
                    None => bounds,
                });
            },
        );
        fonts.trim_scratch();
        visual_bounds
    }

    pub fn measure(&self, available_width: u32, available_height: u32) -> MeasuredSize {
        let mut fonts = FontCtx::new();
        self.measure_with_fonts(available_width, available_height, &mut fonts)
    }

    pub fn measure_with_fonts(
        &self,
        available_width: u32,
        available_height: u32,
        fonts: &mut FontCtx,
    ) -> MeasuredSize {
        let measured = measure_element(self, fonts, available_width, available_height);
        fonts.trim_scratch();
        measured.into()
    }

    pub fn hit_test(&self, bounds: Bounds, x: f64, y: f64) -> Option<Hit> {
        let mut fonts = FontCtx::new();
        self.hit_test_with_fonts(bounds, x, y, &mut fonts)
    }

    pub fn hit_test_with_fonts(
        &self,
        bounds: Bounds,
        x: f64,
        y: f64,
        fonts: &mut FontCtx,
    ) -> Option<Hit> {
        let mut path = Vec::new();
        self.hit_test_detailed_with_fonts(bounds, x, y, fonts, &mut path)
    }

    pub fn hit_test_path(
        &self,
        bounds: Bounds,
        x: f64,
        y: f64,
        path: &mut Vec<usize>,
    ) -> Option<Bounds> {
        let mut fonts = FontCtx::new();
        self.hit_test_path_with_fonts(bounds, x, y, &mut fonts, path)
    }

    pub fn hit_test_path_with_fonts(
        &self,
        bounds: Bounds,
        x: f64,
        y: f64,
        fonts: &mut FontCtx,
        path: &mut Vec<usize>,
    ) -> Option<Bounds> {
        self.hit_test_detailed_with_fonts(bounds, x, y, fonts, path)
            .map(|hit| hit.bounds)
    }

    fn hit_test_detailed_with_fonts(
        &self,
        bounds: Bounds,
        x: f64,
        y: f64,
        fonts: &mut FontCtx,
        path: &mut Vec<usize>,
    ) -> Option<Hit> {
        path.clear();
        let Some((x, y)) = hit_point(x, y) else {
            fonts.trim_scratch();
            return None;
        };

        let mut current_path = Vec::new();
        let mut hit_id = None;
        let bounds = Self::hit_test_layout(
            self,
            bounds,
            Clip::rect(bounds),
            VisualState::IDENTITY,
            None,
            None,
            None,
            fonts,
            x,
            y,
            &mut current_path,
            path,
            &mut hit_id,
        );
        fonts.trim_scratch();
        bounds.map(|bounds| Hit {
            path: path.clone(),
            id: hit_id,
            bounds,
        })
    }
}
