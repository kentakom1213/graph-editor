// src/components/top_panel.rs
use egui::Context;

use crate::GraphEditorApp;

#[derive(Default)]
pub struct CursorHoverState {
    top_panel: bool,
    tool_bar: bool,
    inspector_panel: bool,
    footer_panel: bool,
    settings_window: bool,
    canvas_controls: bool,
    editor_window: bool,
}

impl CursorHoverState {
    pub fn set_top_panel(&mut self, hovered: bool) {
        self.top_panel = hovered;
    }
    pub fn set_tool_bar(&mut self, hovered: bool) {
        self.tool_bar = hovered;
    }
    pub fn set_inspector_panel(&mut self, hovered: bool) {
        self.inspector_panel = hovered;
    }
    pub fn set_footer_panel(&mut self, hovered: bool) {
        self.footer_panel = hovered;
    }
    pub fn set_settings_window(&mut self, hovered: bool) {
        self.settings_window = hovered;
    }
    pub fn set_canvas_controls(&mut self, hovered: bool) {
        self.canvas_controls = hovered;
    }
    pub fn set_editor_window(&mut self, hovered: bool) {
        self.editor_window = hovered;
    }
    pub fn get_top_panel(&self) -> bool {
        self.top_panel
    }
    pub fn get_tool_bar(&self) -> bool {
        self.tool_bar
    }
    pub fn get_inspector_panel(&self) -> bool {
        self.inspector_panel
    }
    pub fn get_settings_window(&self) -> bool {
        self.settings_window
    }
    /// いずれかのパネルにカーソルが乗っているか
    pub fn any(&self) -> bool {
        self.top_panel
            || self.tool_bar
            || self.inspector_panel
            || self.footer_panel
            || self.settings_window
            || self.canvas_controls
            || self.editor_window
    }
}

pub fn draw_top_panel(app: &mut GraphEditorApp, ctx: &Context) {
    app.ui.cursor_hover.set_top_panel(false);
    draw_settings_window(app, ctx);
}

fn draw_settings_window(app: &mut GraphEditorApp, ctx: &Context) {
    app.ui.cursor_hover.set_settings_window(false);

    if !app.ui.show_settings {
        return;
    }

    let mut open = app.ui.show_settings;
    egui::Window::new("Settings")
        .open(&mut open)
        .default_width(320.0)
        .resizable(true)
        .show(ctx, |ui| {
            let mut edge_length_changed = false;
            let mut reset_defaults = false;

            app.ui
                .cursor_hover
                .set_settings_window(ui.rect_contains_pointer(ui.max_rect()));

            ui.label(
                egui::RichText::new("Typography")
                    .strong()
                    .size(app.config.section_font_size()),
            );
            ui.add(egui::Slider::new(&mut app.config.ui_font_size, 10.0..=28.0).text("UI font"));
            ui.add(
                egui::Slider::new(&mut app.config.vertex_font_size, 16.0..=64.0)
                    .text("Vertex font"),
            );

            ui.separator();
            ui.label(
                egui::RichText::new("Canvas")
                    .strong()
                    .size(app.config.section_font_size()),
            );
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Background").size(app.config.body_font_size()));
                ui.color_edit_button_srgba(&mut app.config.bg_color);
            });
            ui.add(
                egui::Slider::new(&mut app.config.vertex_radius, 16.0..=72.0).text("Vertex size"),
            );
            ui.add(
                egui::Slider::new(&mut app.config.vertex_stroke, 1.0..=8.0).text("Vertex stroke"),
            );
            ui.add(egui::Slider::new(&mut app.config.edge_stroke, 1.0..=12.0).text("Edge stroke"));
            edge_length_changed |= ui
                .add(
                    egui::Slider::new(&mut app.config.simulator_config.l, 60.0..=320.0)
                        .text("Edge length"),
                )
                .changed();
            ui.add(
                egui::Slider::new(&mut app.config.edge_bezier_distance, 0.0..=120.0)
                    .text("Bezier distance"),
            );

            ui.separator();
            ui.label(
                egui::RichText::new("Interaction")
                    .strong()
                    .size(app.config.section_font_size()),
            );
            ui.add(egui::Slider::new(&mut app.config.scale_min, 0.05..=1.0).text("Min zoom"));
            ui.add(egui::Slider::new(&mut app.config.scale_max, 1.0..=6.0).text("Max zoom"));
            ui.add(
                egui::Slider::new(&mut app.config.scale_delta, 0.0005..=0.01).text("Zoom speed"),
            );

            ui.add_space(8.0);
            if ui
                .button(egui::RichText::new("Reset Defaults").size(app.config.button_font_size()))
                .clicked()
            {
                let defaults = crate::config::AppConfig::default();
                app.config = defaults;
                reset_defaults = true;
            }

            if edge_length_changed || reset_defaults {
                app.refresh_layout_edge_length_from_config(ctx);
            }
        });
    app.ui.show_settings = open;
}
