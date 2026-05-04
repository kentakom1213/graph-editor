use egui::Context;

use crate::{components::Colors, mode::EditMode, GraphEditorApp};

pub fn draw_tool_bar(app: &mut GraphEditorApp, ctx: &Context) {
    egui::SidePanel::left("tool_bar")
        .show_separator_line(false)
        .resizable(false)
        .exact_width(166.0)
        .show(ctx, |ui| {
            app.ui
                .cursor_hover
                .set_tool_bar(ui.rect_contains_pointer(ui.max_rect()));

            ui.vertical(|ui| {
                draw_mode_button(
                    ui,
                    app.state.edit_mode == EditMode::default_normal(),
                    "Normal [Esc]",
                    app.config.button_font_size(),
                    || app.switch_normal_mode(),
                );
                draw_mode_button(
                    ui,
                    app.state.edit_mode == EditMode::default_add_vertex(),
                    "Add Vertex [V]",
                    app.config.button_font_size(),
                    || app.switch_add_vertex_mode(),
                );
                draw_mode_button(
                    ui,
                    app.state.edit_mode.is_add_edge(),
                    "Add Edge [E]",
                    app.config.button_font_size(),
                    || app.switch_add_edge_mode(),
                );
                draw_mode_button(
                    ui,
                    app.state.edit_mode.is_colorize(),
                    "Colorize [C]",
                    app.config.button_font_size(),
                    || app.switch_colorize_mode(),
                );
                draw_mode_button(
                    ui,
                    app.state.edit_mode.is_delete(),
                    "Delete [D]",
                    app.config.button_font_size(),
                    || app.switch_delete_mode(),
                );
            });

            ui.separator();
            ui.label(egui::RichText::new("Color").size(app.config.section_font_size()));

            let prev_color = app.state.selected_color;
            ui.horizontal_wrapped(|ui| {
                for (label, color) in [
                    ("Default", Colors::Default),
                    ("Red", Colors::Red),
                    ("Green", Colors::Green),
                    ("Blue", Colors::Blue),
                    ("Yellow", Colors::Yellow),
                    ("Orange", Colors::Orange),
                    ("Violet", Colors::Violet),
                    ("Pink", Colors::Pink),
                    ("Brown", Colors::Brown),
                    ("Cyan", Colors::Cyan),
                    ("Indigo", Colors::Indigo),
                    ("Gray", Colors::Gray),
                ] {
                    let fill = if color == Colors::Default {
                        egui::Color32::WHITE
                    } else {
                        color.vertex()
                    };
                    let stroke_color = if app.state.selected_color == color {
                        egui::Color32::BLACK
                    } else {
                        egui::Color32::from_gray(120)
                    };
                    let response = ui
                        .add(
                            egui::Button::new("")
                                .min_size(egui::vec2(28.0, 28.0))
                                .fill(fill)
                                .stroke(egui::Stroke::new(2.0, stroke_color)),
                        )
                        .on_hover_text(label);
                    if response.clicked() {
                        app.state.selected_color = color;
                    }
                }
            });

            if app.state.selected_color != prev_color {
                app.state.edit_mode = EditMode::default_colorize();
            }
        });
}

fn draw_mode_button(
    ui: &mut egui::Ui,
    selected: bool,
    label: &str,
    font_size: f32,
    on_click: impl FnOnce(),
) {
    if ui
        .add_sized(
            [150.0, 32.0],
            egui::SelectableLabel::new(selected, egui::RichText::new(label).size(font_size)),
        )
        .clicked()
    {
        on_click();
    }
}
