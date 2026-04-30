use egui::Context;

use crate::GraphEditorApp;

fn draw_modal_background(ctx: &Context) {
    let screen_rect = ctx.screen_rect();
    let dark_color = egui::Color32::from_black_alpha(160);
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("modal_bg"),
    ));
    painter.rect_filled(screen_rect, 0.0, dark_color);
}

fn draw_modal_window(
    ctx: &Context,
    title: egui::RichText,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(egui::Frame::popup(ctx.style().as_ref()).inner_margin(10.0))
        .show(ctx, |ui| {
            add_contents(ui);
        });
}

/// エラー表示を行うモーダル画面
pub fn draw_error_modal(app: &mut GraphEditorApp, ctx: &Context) {
    let Some(message) = app.ui.error_message.to_owned() else {
        return;
    };

    draw_modal_background(ctx);

    if ctx.input(|i| {
        i.key_pressed(egui::Key::Escape) || i.pointer.any_released() && !app.ui.input_has_focus
    }) {
        app.ui.error_message = None;
        return;
    }

    let title = egui::RichText::new("Error")
        .strong()
        .size(app.config.menu_font_size_normal)
        .color(egui::Color32::from_rgb(255, 100, 80));

    draw_modal_window(ctx, title, |ui| {
        ui.label(egui::RichText::new(message).size(app.config.menu_font_size_normal));
    });
}

/// グラフの全削除を確認するモーダル画面
pub fn draw_clear_all_modal(app: &mut GraphEditorApp, ctx: &Context) {
    if !app.ui.confirm_clear_all {
        return;
    }

    draw_modal_background(ctx);

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.ui.confirm_clear_all = false;
        return;
    }

    let title = egui::RichText::new("Confirm")
        .strong()
        .size(app.config.menu_font_size_normal);

    draw_modal_window(ctx, title, |ui| {
        ui.label(
            egui::RichText::new("Clear all vertices and edges?\n")
                .size(app.config.menu_font_size_normal),
        );
        ui.horizontal(|ui| {
            let clear_button = egui::Button::new(
                egui::RichText::new("Clear")
                    .size(app.config.menu_font_size_normal)
                    .color(egui::Color32::from_rgb(200, 60, 60)),
            );
            let cancel_button = egui::Button::new(
                egui::RichText::new("Cancel").size(app.config.menu_font_size_normal),
            );

            if ui.add(clear_button).clicked() {
                app.state.graph.clear();
                app.state.graph_view.reset_for_graph(&app.state.graph);
                app.state.next_z_index = 0;
                app.ui.confirm_clear_all = false;
            }

            if ui.add(cancel_button).clicked() {
                app.ui.confirm_clear_all = false;
            }
        });
    });
}
