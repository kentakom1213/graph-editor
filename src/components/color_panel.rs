use egui::Context;

use crate::GraphEditorApp;

#[derive(Debug, PartialEq, Clone)]
pub enum Colors {
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
    pub fn to_egui_color(&self) -> egui::Color32 {
        match self {
            Colors::Default => egui::Color32::WHITE,
            Colors::Red => egui::Color32::from_rgb(255, 0, 0),
            Colors::Green => egui::Color32::from_rgb(0, 255, 0),
            Colors::Blue => egui::Color32::from_rgb(0, 0, 255),
            Colors::Yellow => egui::Color32::from_rgb(255, 255, 0),
            Colors::Orange => egui::Color32::from_rgb(255, 165, 0),
            Colors::Violet => egui::Color32::from_rgb(238, 130, 238),
            Colors::Pink => egui::Color32::from_rgb(255, 192, 203),
            Colors::Brown => egui::Color32::from_rgb(165, 42, 42),
        }
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
            app.cursor_hover
                .set_color_window(ui.rect_contains_pointer(ui.max_rect()));

            egui::Frame::new()
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // 色の選択
                        ui.radio_value(
                            &mut app.selected_color,
                            Colors::Default,
                            egui::RichText::new("Default")
                                .color(Colors::Default.to_egui_color())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.selected_color,
                            Colors::Red,
                            egui::RichText::new("Red")
                                .color(Colors::Red.to_egui_color())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.selected_color,
                            Colors::Green,
                            egui::RichText::new("Green")
                                .color(Colors::Green.to_egui_color())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.selected_color,
                            Colors::Blue,
                            egui::RichText::new("Blue")
                                .color(Colors::Blue.to_egui_color())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.selected_color,
                            Colors::Yellow,
                            egui::RichText::new("Yellow")
                                .color(Colors::Yellow.to_egui_color())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.selected_color,
                            Colors::Orange,
                            egui::RichText::new("Orange")
                                .color(Colors::Orange.to_egui_color())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.selected_color,
                            Colors::Violet,
                            egui::RichText::new("Violet")
                                .color(Colors::Violet.to_egui_color())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.selected_color,
                            Colors::Pink,
                            egui::RichText::new("Pink")
                                .color(Colors::Pink.to_egui_color())
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.selected_color,
                            Colors::Brown,
                            egui::RichText::new("Brown")
                                .color(Colors::Brown.to_egui_color())
                                .size(app.config.menu_font_size_normal),
                        );
                    });
                });
        });
}
