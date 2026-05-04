use egui::Context;

use crate::{
    components::{
        Colors, PaletteTheme, VertexPattern, COLOR_SLOTS, EDGE_LINE_STYLES, VERTEX_PATTERNS,
    },
    mode::EditMode,
    GraphEditorApp,
};

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
            ui.label(egui::RichText::new("Theme").size(app.config.section_font_size()));
            egui::ComboBox::from_id_salt("tool_bar_theme")
                .width(150.0)
                .selected_text(
                    egui::RichText::new(app.state.palette_theme.label())
                        .size(app.config.body_font_size()),
                )
                .show_ui(ui, |ui| {
                    for theme in PaletteTheme::all() {
                        let response = ui.selectable_value(
                            &mut app.state.palette_theme,
                            theme,
                            egui::RichText::new(theme.label()).size(app.config.body_font_size()),
                        );
                        response.on_hover_text(theme.description());
                    }
                });

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Color").size(app.config.section_font_size()));

            let prev_color = app.state.selected_color;
            ui.horizontal_wrapped(|ui| {
                for color in COLOR_SLOTS {
                    let fill = if color == Colors::Default {
                        egui::Color32::WHITE
                    } else {
                        color.vertex(app.state.palette_theme)
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
                        .on_hover_text(color.label());
                    if response.clicked() {
                        app.state.selected_color = color;
                    }
                }
            });

            if app.state.selected_color != prev_color {
                app.state.edit_mode = EditMode::default_colorize();
            }

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Pattern").size(app.config.section_font_size()));

            let prev_pattern = app.state.selected_pattern;
            ui.horizontal_wrapped(|ui| {
                for pattern in VERTEX_PATTERNS {
                    let stroke_color = if app.state.selected_pattern == pattern {
                        egui::Color32::BLACK
                    } else {
                        egui::Color32::from_gray(120)
                    };
                    let response = ui
                        .add(
                            egui::Button::new(pattern.label())
                                .min_size(egui::vec2(70.0, 28.0))
                                .stroke(egui::Stroke::new(2.0, stroke_color)),
                        )
                        .on_hover_text(match pattern {
                            VertexPattern::None => "No pattern overlay",
                            VertexPattern::Diagonal => "Diagonal hatch",
                            VertexPattern::Dots => "Dot pattern",
                            VertexPattern::Cross => "Cross hatch",
                        });
                    if response.clicked() {
                        app.state.selected_pattern = pattern;
                    }
                }
            });

            if app.state.selected_pattern != prev_pattern {
                app.state.edit_mode = EditMode::default_colorize();
            }

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Line").size(app.config.section_font_size()));
            egui::ComboBox::from_id_salt("tool_bar_line_style")
                .width(150.0)
                .selected_text(
                    egui::RichText::new(app.state.selected_line_style.label())
                        .size(app.config.body_font_size()),
                )
                .show_ui(ui, |ui| {
                    for style in EDGE_LINE_STYLES {
                        ui.selectable_value(
                            &mut app.state.selected_line_style,
                            style,
                            egui::RichText::new(style.label()).size(app.config.body_font_size()),
                        );
                    }
                });
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
