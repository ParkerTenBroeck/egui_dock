use super::utils::*;
use egui::style::Margin;
use egui::*;

/// Specifies the look and feel of egui_dock.
#[derive(Clone)]
pub struct Style {
    pub padding: Option<Margin>,

    pub border_color: Color32,
    pub border_width: f32,

    /// Color used when previewing where a tab will end up.
    pub selection_color: Color32,

    pub separator_width: f32,
    pub separator_extra: f32,
    pub separator_color: Color32,

    pub tab_bar_background_color: Color32,

    pub tab_outline_color: Color32,
    pub tab_rounding: Rounding,
    pub tab_background_color: Color32,

    pub tab_text_color_unfocused: Color32,
    pub tab_text_color_focused: Color32,

    pub close_tab_color: Color32,
    pub close_tab_active_color: Color32,
    pub close_tab_background_color: Color32,
    pub show_close_buttons: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            padding: None,

            border_color: Color32::BLACK,
            border_width: Default::default(),

            selection_color: Color32::from_rgb(0, 191, 255).linear_multiply(0.5),
            separator_width: 1.0,
            separator_extra: 175.0,
            separator_color: Color32::BLACK,

            tab_bar_background_color: Color32::WHITE,

            tab_outline_color: Color32::BLACK,
            tab_rounding: Default::default(),
            tab_background_color: Color32::WHITE,

            tab_text_color_unfocused: Color32::DARK_GRAY,
            tab_text_color_focused: Color32::BLACK,

            close_tab_color: Color32::WHITE,
            close_tab_active_color: Color32::WHITE,
            close_tab_background_color: Color32::GRAY,
            show_close_buttons: true,
        }
    }
}

impl Style {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - `selection_color`
    /// - `tab_bar_background_color`
    /// - `tab_outline_color`
    /// - `tab_background_color`
    /// - `separator_color`
    /// - `border_color`
    /// - `close_tab_background_color`
    /// - `close_tab_color`
    /// - `close_tab_active_color`
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            selection_color: style.visuals.selection.bg_fill.linear_multiply(0.5),

            tab_bar_background_color: style.visuals.faint_bg_color,
            tab_outline_color: style.visuals.widgets.active.bg_fill,
            tab_background_color: style.visuals.window_fill(),

            tab_text_color_unfocused: style.visuals.text_color(),
            tab_text_color_focused: style.visuals.strong_text_color(),

            separator_color: style.visuals.widgets.active.bg_fill,
            border_color: style.visuals.widgets.active.bg_fill,

            close_tab_background_color: style.visuals.widgets.active.bg_fill,
            close_tab_color: style.visuals.text_color(),
            close_tab_active_color: style.visuals.strong_text_color(),
            ..Self::default()
        }
    }

    pub(crate) fn hsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = midpoint - self.separator_width * 0.5;
        separator.max.x = midpoint + self.separator_width * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::ResizeHorizontal);

        {
            let delta = response.drag_delta().x;
            let range = rect.max.x - rect.min.x;
            let min = (self.separator_extra / range).min(1.0);
            let max = 1.0 - min;
            let (min, max) = (min.min(max), max.max(min));
            *fraction = (*fraction + delta / range).clamp(min, max);
        }

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = map_to_pixel(
            midpoint - self.separator_width * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.x = map_to_pixel(
            midpoint + self.separator_width * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            rect.intersect(Rect::everything_right_of(separator.max.x)),
            separator,
            rect.intersect(Rect::everything_left_of(separator.min.x)),
        )
    }

    pub(crate) fn vsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = midpoint - self.separator_width * 0.5;
        separator.max.y = midpoint + self.separator_width * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::ResizeVertical);

        {
            let delta = response.drag_delta().y;
            let range = rect.max.y - rect.min.y;
            let min = (self.separator_extra / range).min(1.0);
            let max = 1.0 - min;
            let (min, max) = (min.min(max), max.max(min));
            *fraction = (*fraction + delta / range).clamp(min, max);
        }

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = map_to_pixel(
            midpoint - self.separator_width * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.y = map_to_pixel(
            midpoint + self.separator_width * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            rect.intersect(Rect::everything_above(separator.min.y)),
            separator,
            rect.intersect(Rect::everything_below(separator.max.y)),
        )
    }

    /// `active` means "the tab that is opened in the parent panel".
    pub(crate) fn tab_title(
        &self,
        ui: &mut Ui,
        label: WidgetText,
        focused: bool,
        active: bool,
        is_being_dragged: bool,
        id: Id,
    ) -> (Response, bool, bool) {
        let px = ui.ctx().pixels_per_point().recip();
        let rounding = self.tab_rounding;

        let galley = label.into_galley(ui, None, f32::INFINITY, TextStyle::Button);

        let x_text_gap = 5.0;
        let x_size = Vec2::new(galley.size().y / 1.3, galley.size().y / 1.3);

        let offset = vec2(8.0, 0.0);
        let text_size = galley.size();

        let mut desired_size = text_size + offset * 2.0;
        if self.show_close_buttons {
            desired_size.x += x_size.x + x_text_gap;
        }
        desired_size.y = 24.0;

        let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
        let response = response.on_hover_cursor(CursorIcon::PointingHand);

        let (x_rect, x_res) = if (active || response.hovered()) && self.show_close_buttons {
            let mut pos = rect.left_top();
            pos.x += offset.x + text_size.x + x_text_gap + x_size.x / 2.0;
            pos.y += rect.size().y / 2.0;
            let x_rect = Rect::from_center_size(pos, x_size);
            (x_rect, Some(ui.interact(x_rect, id, Sense::click())))
        } else {
            (Rect::NOTHING, None)
        };
        match (active, is_being_dragged) {
            (true, false) => {
                let mut tab = rect;
                tab.min.x -= px;
                tab.max.x += px;
                ui.painter()
                    .rect_filled(tab, rounding, self.tab_outline_color);

                tab.min.x += px;
                tab.max.x -= px;
                tab.min.y += px;
                ui.painter()
                    .rect_filled(tab, rounding, self.tab_background_color);
            }
            (true, true) => {
                let tab = rect;

                ui.painter().rect_stroke(
                    tab,
                    self.tab_rounding,
                    Stroke::new(1.0, self.tab_outline_color),
                );
            }
            _ => (),
        }

        let pos = Align2::LEFT_TOP
            .anchor_rect(rect.shrink2(vec2(8.0, 5.0)))
            .min;

        let override_text_color = if galley.galley_has_color {
            None // respect the color the user has chosen
        } else if focused {
            Some(self.tab_text_color_focused)
        } else {
            Some(self.tab_text_color_unfocused)
        };
        ui.painter().add(epaint::TextShape {
            pos,
            galley: galley.galley,
            underline: Stroke::none(),
            override_text_color,
            angle: 0.0,
        });

        if (active || response.hovered()) && self.show_close_buttons {
            if x_res.as_ref().unwrap().hovered() {
                ui.painter().rect_filled(
                    x_rect,
                    Rounding::same(2.0),
                    self.close_tab_background_color,
                );
            }
            let x_rect = x_rect.shrink(1.75);

            let color = if focused || x_res.as_ref().unwrap().interact_pointer_pos().is_some() {
                self.close_tab_active_color
            } else {
                self.close_tab_color
            };
            ui.painter().line_segment(
                [x_rect.left_top(), x_rect.right_bottom()],
                Stroke::new(1.0, color),
            );
            ui.painter().line_segment(
                [x_rect.right_top(), x_rect.left_bottom()],
                Stroke::new(1.0, color),
            );
        }

        match x_res {
            Some(some) => (response, some.hovered(), some.clicked()),
            None => (response, false, false),
        }
    }
}

#[derive(Default)]
pub struct StyleBuilder {
    style: Style,
}

impl StyleBuilder {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `padding` to indent from the edges of the window. By `Default` it's `None`.
    #[inline(always)]
    pub fn with_padding(mut self, padding: Option<Margin>) -> Self {
        self.style.padding = padding;
        self
    }

    /// Sets `border_color` for the window of "working area". By `Default` it's [`egui::Color32::BLACK`].
    #[inline(always)]
    pub fn with_border_color(mut self, border_color: Color32) -> Self {
        self.style.border_color = border_color;
        self
    }

    /// Sets `border_width` for the border. By `Default` it's `0.0`.
    #[inline(always)]
    pub fn with_border_width(mut self, border_width: f32) -> Self {
        self.style.border_width = border_width;
        self
    }

    /// Sets `selection color` for the placing area of the tab where this tab targeted on it. By `Default` it's `(0, 191, 255)` (light blue) with `0.5` capacity.
    #[inline(always)]
    pub fn with_selection_color(mut self, selection_color: Color32) -> Self {
        self.style.selection_color = selection_color;
        self
    }

    /// Sets `separator_size` for the rectangle separator between nodes. By `Default` it's `1.0`.
    #[inline(always)]
    pub fn with_separator_width(mut self, separator_width: f32) -> Self {
        self.style.separator_width = separator_width;
        self
    }

    /// Sets `separator_extra` it sets limit for the allowed area for the separator offset. By `Default` it's `175.0`.
    /// `bigger value > less allowed offset` for the current window size.
    #[inline(always)]
    pub fn with_separator_extra(mut self, separator_extra: f32) -> Self {
        self.style.separator_extra = separator_extra;
        self
    }

    /// Sets `separator_color`for the rectangle separator. By `Default` it's [`Color32::BLACK`].
    #[inline(always)]
    pub fn with_separator_color(mut self, separator_color: Color32) -> Self {
        self.style.separator_color = separator_color;
        self
    }

    /// Sets `tab_bar_background_color` for the color of tab bar. By `Default` it's [`Color32::WHITE`].
    #[inline(always)]
    pub fn with_tab_bar_background(mut self, tab_bar_background_color: Color32) -> Self {
        self.style.tab_bar_background_color = tab_bar_background_color;
        self
    }

    /// Sets `tab_outline_color` for the outline color of tabs. By `Default` it's [`Color32::BLACK`].
    #[inline(always)]
    pub fn with_tab_outline_color(mut self, tab_outline_color: Color32) -> Self {
        self.style.tab_outline_color = tab_outline_color;
        self
    }

    /// Sets `tab_rounding` for the tab rounding.
    #[inline(always)]
    pub fn with_tab_rounding(mut self, tab_rounding: Rounding) -> Self {
        self.style.tab_rounding = tab_rounding;
        self
    }

    /// Sets `tab_background_color` for the current tab background color.
    #[inline(always)]
    pub fn with_tab_background_color(mut self, tab_background: Color32) -> Self {
        self.style.tab_background_color = tab_background;
        self
    }

    /// Sets `close_tab_color` for the close tab button color.
    #[inline(always)]
    pub fn with_close_tab_color(mut self, close_tab_color: Color32) -> Self {
        self.style.close_tab_color = close_tab_color;
        self
    }

    /// Sets `close_tab_active_color` for the active close tab button color.
    #[inline(always)]
    pub fn with_close_tab_active_color_color(mut self, close_tab_active_color: Color32) -> Self {
        self.style.close_tab_active_color = close_tab_active_color;
        self
    }

    /// Sets `close_tab_background_color` for the background close tab button color.
    #[inline(always)]
    pub fn with_close_tab_background_color_color(
        mut self,
        close_tab_background_color: Color32,
    ) -> Self {
        self.style.close_tab_background_color = close_tab_background_color;
        self
    }

    /// Shows / Hides the tab close buttons.
    #[inline(always)]
    pub fn show_close_buttons(mut self, show_close_buttons: bool) -> Self {
        self.style.show_close_buttons = show_close_buttons;
        self
    }

    /// Returns `Style` with set values.
    #[inline(always)]
    pub fn build(self) -> Style {
        self.style
    }
}
