use egui::Context;

use crate::{components::Colors, mode::EditMode, GraphEditorApp};

pub fn draw_tool_bar(app: &mut GraphEditorApp, ctx: &Context) {
    egui::SidePanel::left("tool_bar")
        .resizable(false)
        .exact_width(190.0)
        .show(ctx, |ui| {
            app.ui
                .cursor_hover
                .set_tool_bar(ui.rect_contains_pointer(ui.max_rect()));

            ui.vertical(|ui| {
                draw_mode_button(
                    ui,
                    app.state.edit_mode == EditMode::default_normal(),
                    "Normal [Esc]",
                    || app.switch_normal_mode(),
                );
                draw_mode_button(
                    ui,
                    app.state.edit_mode == EditMode::default_add_vertex(),
                    "Add Vertex [V]",
                    || app.switch_add_vertex_mode(),
                );
                draw_mode_button(
                    ui,
                    app.state.edit_mode.is_add_edge(),
                    "Add Edge [E]",
                    || app.switch_add_edge_mode(),
                );
                draw_mode_button(
                    ui,
                    app.state.edit_mode.is_colorize(),
                    "Colorize [C]",
                    || app.switch_colorize_mode(),
                );
                draw_mode_button(ui, app.state.edit_mode.is_delete(), "Delete [D]", || {
                    app.switch_delete_mode()
                });
            });

            ui.separator();
            ui.label(egui::RichText::new("Color").size(app.config.menu_font_size_mini));

            let prev_color = app.state.selected_color;
            for (label, color) in [
                ("Def", Colors::Default),
                ("Red", Colors::Red),
                ("Grn", Colors::Green),
                ("Blu", Colors::Blue),
                ("Yel", Colors::Yellow),
                ("Org", Colors::Orange),
                ("Vio", Colors::Violet),
                ("Pnk", Colors::Pink),
                ("Brn", Colors::Brown),
            ] {
                let text = if color == Colors::Default {
                    egui::RichText::new(label)
                } else {
                    egui::RichText::new(label).color(color.vertex())
                };
                if ui
                    .selectable_label(app.state.selected_color == color, text)
                    .on_hover_text(color_label(color))
                    .clicked()
                {
                    app.state.selected_color = color;
                }
            }

            if app.state.selected_color != prev_color {
                app.state.edit_mode = EditMode::default_colorize();
            }
        });
}

fn draw_mode_button(ui: &mut egui::Ui, selected: bool, label: &str, on_click: impl FnOnce()) {
    if ui
        .add_sized([170.0, 28.0], egui::SelectableLabel::new(selected, label))
        .clicked()
    {
        on_click();
    }
}

fn color_label(color: Colors) -> &'static str {
    match color {
        Colors::Default => "Default",
        Colors::Red => "Red",
        Colors::Green => "Green",
        Colors::Blue => "Blue",
        Colors::Yellow => "Yellow",
        Colors::Orange => "Orange",
        Colors::Violet => "Violet",
        Colors::Pink => "Pink",
        Colors::Brown => "Brown",
    }
}
