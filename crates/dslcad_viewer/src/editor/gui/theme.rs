//! A dark, low-contrast egui theme tuned to match the CAD viewport. Colors are
//! aligned with the viewport [`Palette`](crate::editor::Palette) so the panels
//! frame the model instead of competing with it.

use bevy_egui::egui::style::{HandleShape, ScrollStyle, WidgetVisuals};
use bevy_egui::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, Shadow, Stroke, Style, TextStyle,
    Vec2, Visuals,
};

/// Panel, widget and signal colors used by the theme.
struct Colors;

impl Colors {
    /// Background of the side panels, menus and toolbars.
    fn panel() -> Color32 {
        Color32::from_rgb(0x2B, 0x30, 0x38)
    }

    /// Background of an interactive widget at rest.
    fn widget() -> Color32 {
        Color32::from_rgb(0x33, 0x3A, 0x44)
    }

    /// Background of a hovered widget.
    fn widget_hovered() -> Color32 {
        Color32::from_rgb(0x3D, 0x45, 0x50)
    }

    /// Background of a pressed or selected widget.
    fn widget_active() -> Color32 {
        Color32::from_rgb(0x4A, 0x54, 0x61)
    }

    /// Background of text edits and other recessed controls.
    fn recessed() -> Color32 {
        Color32::from_rgb(0x1E, 0x22, 0x29)
    }

    /// Hairline borders between panels and around widgets.
    fn border() -> Color32 {
        Color32::from_rgb(0x14, 0x1A, 0x23)
    }

    /// Primary text.
    fn text() -> Color32 {
        Color32::from_rgb(0xD5, 0xDB, 0xE5)
    }

    /// Secondary text and inactive labels.
    fn text_weak() -> Color32 {
        Color32::from_rgb(0x9A, 0xA3, 0xB0)
    }

    /// Interactive accent, matching the positive z axis in the viewport.
    fn accent() -> Color32 {
        Color32::from_rgb(0x3E, 0x8B, 0xFF)
    }

    /// Warning text, matching the annotation color in the viewport.
    fn warn() -> Color32 {
        Color32::from_rgb(0xF5, 0xC2, 0x4B)
    }

    /// Error text, matching the positive x axis in the viewport.
    fn error() -> Color32 {
        Color32::from_rgb(0xE5, 0x48, 0x4D)
    }
}

/// Applies the CAD theme. It forces the dark theme so the look does not depend
/// on the operating system setting.
pub fn apply(context: &egui::Context) {
    context.set_theme(egui::ThemePreference::Dark);
    context.set_style_of(egui::Theme::Dark, style());
}

fn style() -> Style {
    Style {
        visuals: visuals(),
        spacing: spacing(),
        text_styles: text_styles(),
        ..Default::default()
    }
}

fn visuals() -> Visuals {
    let mut visuals = Visuals::dark();
    let corner_radius = CornerRadius::same(4);
    let border = Stroke::new(1.0_f32, Colors::border());

    visuals.dark_mode = true;
    visuals.override_text_color = None;

    // Frames and surfaces.
    visuals.panel_fill = Colors::panel();
    visuals.window_fill = Colors::panel();
    visuals.window_stroke = border;
    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.menu_corner_radius = CornerRadius::same(4);
    visuals.faint_bg_color = Colors::widget();
    visuals.extreme_bg_color = Colors::recessed();
    visuals.code_bg_color = Colors::recessed();
    visuals.window_shadow = Shadow {
        offset: [0, 6],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(180),
    };
    visuals.popup_shadow = visuals.window_shadow;

    // Signals.
    visuals.hyperlink_color = Colors::accent();
    visuals.warn_fg_color = Colors::warn();
    visuals.error_fg_color = Colors::error();
    visuals.selection.bg_fill = Color32::from_rgb(0x2E, 0x5A, 0x9E);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);

    // Widgets.
    visuals.widgets.noninteractive = WidgetVisuals {
        bg_fill: Colors::panel(),
        weak_bg_fill: Colors::panel(),
        bg_stroke: border,
        corner_radius,
        fg_stroke: Stroke::new(1.0_f32, Colors::text()),
        expansion: 0.0,
    };
    visuals.widgets.inactive = WidgetVisuals {
        bg_fill: Colors::widget(),
        weak_bg_fill: Colors::widget(),
        bg_stroke: border,
        corner_radius,
        fg_stroke: Stroke::new(1.0_f32, Colors::text()),
        expansion: 0.0,
    };
    visuals.widgets.hovered = WidgetVisuals {
        bg_fill: Colors::widget_hovered(),
        weak_bg_fill: Colors::widget_hovered(),
        bg_stroke: Stroke::new(1.0_f32, Colors::accent().gamma_multiply(0.7)),
        corner_radius,
        fg_stroke: Stroke::new(1.0_f32, Color32::WHITE),
        expansion: 0.0,
    };
    visuals.widgets.active = WidgetVisuals {
        bg_fill: Colors::widget_active(),
        weak_bg_fill: Colors::widget_active(),
        bg_stroke: Stroke::new(1.0_f32, Colors::accent()),
        corner_radius,
        fg_stroke: Stroke::new(1.0_f32, Color32::WHITE),
        expansion: 0.0,
    };
    visuals.widgets.open = visuals.widgets.active;

    // A flat, technical look: thin rails and rectangular slider handles.
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline = false;
    visuals.striped = true;
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = HandleShape::Rect { aspect_ratio: 1.5 };
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

    visuals
}

fn spacing() -> egui::Spacing {
    egui::Spacing {
        item_spacing: Vec2::new(8.0, 6.0),
        button_padding: Vec2::new(8.0, 4.0),
        menu_margin: Margin::same(6),
        window_margin: Margin::same(10),
        indent: 16.0,
        slider_width: 150.0,
        scroll: ScrollStyle {
            bar_width: 10.0,
            floating: true,
            bar_inner_margin: 2.0,
            bar_outer_margin: 2.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn text_styles() -> std::collections::BTreeMap<TextStyle, FontId> {
    [
        (
            TextStyle::Heading,
            FontId::new(16.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(13.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(12.0, FontFamily::Monospace),
        ),
    ]
    .into()
}

/// The muted color used for the panel headings, so they read as labels rather
/// than content.
pub fn heading_color() -> Color32 {
    Colors::text_weak()
}
