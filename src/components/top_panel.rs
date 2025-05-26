// src/components/top_panel.rs
use egui::{Context, TopBottomPanel};

use crate::GraphEditorApp;

pub struct PanelTabState {
    pub edit_menu: bool,
    pub graph_io: bool,
}

impl Default for PanelTabState {
    fn default() -> Self {
        PanelTabState {
            edit_menu: true,
            graph_io: true,
        }
    }
}

pub fn draw_top_panel(app: &mut GraphEditorApp, ctx: &Context) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.checkbox(
                &mut app.panel_tab.edit_menu,
                egui::RichText::new("Menu").size(app.config.menu_font_size_normal),
            );
            ui.checkbox(
                &mut app.panel_tab.graph_io,
                egui::RichText::new("Input").size(app.config.menu_font_size_normal),
            );
        });
    });
}
