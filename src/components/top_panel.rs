// src/components/top_panel.rs
use egui::{Context, TopBottomPanel};

use crate::GraphEditorApp;

#[derive(Default)]
pub struct CursorHoverState {
    top_panel: bool,
    tool_bar: bool,
    inspector_panel: bool,
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
    pub fn get_top_panel(&self) -> bool {
        self.top_panel
    }
    pub fn get_tool_bar(&self) -> bool {
        self.tool_bar
    }
    pub fn get_inspector_panel(&self) -> bool {
        self.inspector_panel
    }
    /// いずれかのパネルにカーソルが乗っているか
    pub fn any(&self) -> bool {
        self.top_panel || self.tool_bar || self.inspector_panel
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
                    .size(app.config.menu_font_size_normal),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Settings").clicked() {
                    // TODO: settings panel
                }

                if ui
                    .add_enabled(!app.export.is_busy(), egui::Button::new("Export"))
                    .clicked()
                {
                    app.request_export_image(ctx);
                }
            });
        });
    });
}
