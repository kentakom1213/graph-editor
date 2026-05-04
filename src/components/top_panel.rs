// src/components/top_panel.rs
use egui::{Context, TopBottomPanel};

use crate::GraphEditorApp;

#[derive(Default)]
pub struct CursorHoverState {
    top_panel: bool,
    tool_bar: bool,
    inspector_panel: bool,
    settings_window: bool,
    canvas_controls: bool,
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
    pub fn set_settings_window(&mut self, hovered: bool) {
        self.settings_window = hovered;
    }
    pub fn set_canvas_controls(&mut self, hovered: bool) {
        self.canvas_controls = hovered;
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
            || self.settings_window
            || self.canvas_controls
    }
}

pub fn draw_top_panel(app: &mut GraphEditorApp, ctx: &Context) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // カーソルがあるか判定
        app.ui
            .cursor_hover
            .set_top_panel(ui.rect_contains_pointer(ui.max_rect()));

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Graph Editor")
                    .strong()
                    .size(app.config.title_font_size),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(egui::RichText::new("Settings").size(app.config.button_font_size()))
                    .clicked()
                {
                    app.ui.show_settings = true;
                }
            });
        });
    });

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
            app.ui
                .cursor_hover
                .set_settings_window(ui.rect_contains_pointer(ui.max_rect()));

            ui.label(egui::RichText::new("Typography").size(app.config.section_font_size()));
            ui.add(
                egui::Slider::new(&mut app.config.title_font_size, 12.0..=32.0).text("Title font"),
            );
            ui.add(egui::Slider::new(&mut app.config.ui_font_size, 10.0..=28.0).text("UI font"));
            ui.add(
                egui::Slider::new(&mut app.config.vertex_font_size, 16.0..=64.0)
                    .text("Vertex font"),
            );

            ui.separator();
            ui.label(egui::RichText::new("Canvas").size(app.config.section_font_size()));
            ui.add(
                egui::Slider::new(&mut app.config.vertex_radius, 16.0..=72.0).text("Vertex size"),
            );
            ui.add(
                egui::Slider::new(&mut app.config.vertex_stroke, 1.0..=8.0).text("Vertex stroke"),
            );
            ui.add(egui::Slider::new(&mut app.config.edge_stroke, 1.0..=12.0).text("Edge stroke"));
            ui.add(
                egui::Slider::new(&mut app.config.edge_bezier_distance, 0.0..=120.0)
                    .text("Bezier distance"),
            );

            ui.separator();
            ui.label(egui::RichText::new("Interaction").size(app.config.section_font_size()));
            ui.add(egui::Slider::new(&mut app.config.scale_min, 0.05..=1.0).text("Min zoom"));
            ui.add(egui::Slider::new(&mut app.config.scale_max, 1.0..=6.0).text("Max zoom"));
            ui.add(
                egui::Slider::new(&mut app.config.scale_delta, 0.0005..=0.01).text("Zoom speed"),
            );

            if ui
                .button(egui::RichText::new("Reset Defaults").size(app.config.button_font_size()))
                .clicked()
            {
                let defaults = crate::config::AppConfig::default();
                app.config = defaults;
            }
        });
    app.ui.show_settings = open;
}
