use egui::Context;

use crate::{mode::EditMode, GraphEditorApp};

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum Colors {
    #[default]
    Default,
    Red,
    Green,
    Blue,
    Yellow,
    Orange,
    Violet,
    Pink,
    Brown,
}

impl Colors {
    fn to_egui_color(self) -> Option<egui::Color32> {
        match self {
            Colors::Default => None,
            Colors::Red => Some(egui::Color32::from_rgb(255, 70, 70)),
            Colors::Green => Some(egui::Color32::from_rgb(70, 255, 70)),
            Colors::Blue => Some(egui::Color32::from_rgb(70, 70, 255)),
            Colors::Yellow => Some(egui::Color32::from_rgb(255, 255, 0)),
            Colors::Orange => Some(egui::Color32::from_rgb(255, 165, 0)),
            Colors::Violet => Some(egui::Color32::from_rgb(238, 130, 238)),
            Colors::Pink => Some(egui::Color32::from_rgb(255, 192, 203)),
            Colors::Brown => Some(egui::Color32::from_rgb(181, 101, 29)),
        }
    }

    pub fn vertex(&self) -> egui::Color32 {
        self.to_egui_color().unwrap_or(egui::Color32::WHITE)
    }

    pub fn edge(&self) -> egui::Color32 {
        self.to_egui_color()
            .unwrap_or(egui::Color32::from_rgb(100, 100, 100))
    }
}

/// グラフのエンコードを表示する
pub fn draw_color_settings(app: &mut GraphEditorApp, ctx: &Context) {
    // テキストの表示
    egui::Window::new("Color")
        .collapsible(false)
        .default_width(150.0)
        .show(ctx, |ui| {
            // カーソルがあるか判定
            app.ui
                .cursor_hover
                .set_color_window(ui.rect_contains_pointer(ui.max_rect()));

            egui::Frame::new()
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        let prev_color = app.state.selected_color;

                        // 色の選択
                        ui.radio_value(
                            &mut app.state.selected_color,
                            Colors::Default,
                            egui::RichText::new("Default").size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.state.selected_color,
                            Colors::Red,
                            egui::RichText::new("Red")
                                .color(Colors::Red.vertex())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.state.selected_color,
                            Colors::Green,
                            egui::RichText::new("Green")
                                .color(Colors::Green.vertex())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.state.selected_color,
                            Colors::Blue,
                            egui::RichText::new("Blue")
                                .color(Colors::Blue.vertex())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.state.selected_color,
                            Colors::Yellow,
                            egui::RichText::new("Yellow")
                                .color(Colors::Yellow.vertex())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.state.selected_color,
                            Colors::Orange,
                            egui::RichText::new("Orange")
                                .color(Colors::Orange.vertex())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.state.selected_color,
                            Colors::Violet,
                            egui::RichText::new("Violet")
                                .color(Colors::Violet.vertex())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.state.selected_color,
                            Colors::Pink,
                            egui::RichText::new("Pink")
                                .color(Colors::Pink.vertex())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.state.selected_color,
                            Colors::Brown,
                            egui::RichText::new("Brown")
                                .color(Colors::Brown.vertex())
                                .size(app.config.menu_font_size_normal),
                        );

                        // 色が変わっていたらモードを切り替え
                        if app.state.selected_color != prev_color {
                            app.state.edit_mode = EditMode::default_colorize();
                        }
                    });
                });
        });
}
