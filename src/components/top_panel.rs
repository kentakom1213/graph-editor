// src/components/top_panel.rs
use egui::{Context, TopBottomPanel};

use crate::GraphEditorApp;

pub struct PanelTabState {
    pub edit_menu: bool,
    pub color_settings: bool,
    pub graph_io: bool,
}

impl Default for PanelTabState {
    fn default() -> Self {
        Self {
            edit_menu: true,
            color_settings: true,
            graph_io: true,
        }
    }
}

#[derive(Default)]
pub struct CursorHoverState {
    top_panel: bool,
    color_window: bool,
    menu_window: bool,
    input_window: bool,
}

impl CursorHoverState {
    pub fn set_top_panel(&mut self, hovered: bool) {
        self.top_panel = hovered;
    }
    pub fn set_color_window(&mut self, hovered: bool) {
        self.color_window = hovered;
    }
    pub fn set_menu_window(&mut self, hovered: bool) {
        self.menu_window = hovered;
    }
    pub fn set_input_window(&mut self, hovered: bool) {
        self.input_window = hovered;
    }
    pub fn get_top_panel(&self) -> bool {
        self.top_panel
    }
    pub fn get_color_window(&self) -> bool {
        self.color_window
    }
    pub fn get_menu_window(&self) -> bool {
        self.menu_window
    }
    pub fn get_input_window(&self) -> bool {
        self.input_window
    }
    /// いずれかのパネルにカーソルが乗っているか
    pub fn any(&self) -> bool {
        self.top_panel || self.color_window || self.menu_window || self.input_window
    }
}

pub fn draw_top_panel(app: &mut GraphEditorApp, ctx: &Context) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // カーソルがあるか判定
        app.ui.cursor_hover
            .set_top_panel(ui.rect_contains_pointer(ui.max_rect()));

        egui::menu::bar(ui, |ui| {
            ui.toggle_value(
                &mut app.ui.panel_tab.edit_menu,
                egui::RichText::new("Menu").size(app.config.menu_font_size_normal),
            );
            ui.toggle_value(
                &mut app.ui.panel_tab.color_settings,
                egui::RichText::new("Color").size(app.config.menu_font_size_normal),
            );
            ui.toggle_value(
                &mut app.ui.panel_tab.graph_io,
                egui::RichText::new("Input").size(app.config.menu_font_size_normal),
            );
        });
    });
}
