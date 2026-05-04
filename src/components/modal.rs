use egui::Context;

use crate::{
    components::{
        default_vertex_text_color, Colors, EdgeLineStyle, VertexPattern, COLOR_SLOTS,
        EDGE_LINE_STYLES, VERTEX_PATTERNS,
    },
    mode::EditMode,
    state::EditTarget,
    GraphEditorApp,
};

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

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.ui.error_message = None;
        return;
    }

    let title = egui::RichText::new("Error")
        .strong()
        .size(app.config.title_font_size())
        .color(egui::Color32::from_rgb(255, 100, 80));

    draw_modal_window(ctx, title, |ui| {
        ui.label(egui::RichText::new(message).size(app.config.body_font_size()));
        ui.separator();
        if ui
            .button(egui::RichText::new("Close").size(app.config.button_font_size()))
            .clicked()
        {
            app.ui.error_message = None;
        }
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
        .size(app.config.title_font_size());

    draw_modal_window(ctx, title, |ui| {
        ui.label(
            egui::RichText::new("Clear all vertices and edges?\n")
                .size(app.config.body_font_size()),
        );
        ui.horizontal(|ui| {
            let clear_button = egui::Button::new(
                egui::RichText::new("Clear")
                    .size(app.config.button_font_size())
                    .color(egui::Color32::from_rgb(200, 60, 60)),
            );
            let cancel_button = egui::Button::new(
                egui::RichText::new("Cancel").size(app.config.button_font_size()),
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

pub fn draw_entity_editor(app: &mut GraphEditorApp, ctx: &Context) {
    app.ui.cursor_hover.set_editor_window(false);

    if app.state.edit_mode != EditMode::Normal {
        app.close_entity_editor();
        return;
    }

    let Some(target) = app.ui.edit_target else {
        return;
    };

    let mut open = true;
    let pos = app
        .ui
        .edit_window_pos
        .unwrap_or_else(|| ctx.screen_rect().center());
    egui::Window::new(match target {
        EditTarget::Vertex(_) => "Vertex",
        EditTarget::Edge(_) => "Edge",
    })
    .open(&mut open)
    .default_pos(pos)
    .resizable(false)
    .show(ctx, |ui| {
        app.ui
            .cursor_hover
            .set_editor_window(ui.rect_contains_pointer(ui.max_rect()));

        match target {
            EditTarget::Vertex(index) => draw_vertex_editor(app, ui, index),
            EditTarget::Edge(index) => draw_edge_editor(app, ui, index),
        }
    });

    if !open {
        app.ui.edit_target = None;
        app.ui.edit_window_pos = None;
    }
}

fn draw_vertex_editor(app: &mut GraphEditorApp, ui: &mut egui::Ui, index: usize) {
    let Some(view) = app.state.graph_view.vertices.get_mut(index) else {
        app.ui.edit_target = None;
        return;
    };
    let Some(vertex) = app.state.graph.vertices.get(index) else {
        app.ui.edit_target = None;
        return;
    };

    ui.label(format!("id: {}", vertex.id));
    ui.separator();

    ui.label(
        egui::RichText::new("Label")
            .strong()
            .size(app.config.section_font_size()),
    );
    let default_label = if app.state.zero_indexed {
        vertex.id.to_string()
    } else {
        (vertex.id + 1).to_string()
    };
    let label = view.label.get_or_insert(default_label);
    ui.text_edit_singleline(label);

    ui.separator();
    ui.label(
        egui::RichText::new("Fill")
            .strong()
            .size(app.config.section_font_size()),
    );
    draw_color_palette(ui, &mut view.color, app.state.palette_theme);

    ui.separator();
    ui.label(
        egui::RichText::new("Pattern")
            .strong()
            .size(app.config.section_font_size()),
    );
    draw_pattern_palette(ui, &mut view.pattern);

    ui.separator();
    ui.label(
        egui::RichText::new("Text")
            .strong()
            .size(app.config.section_font_size()),
    );
    let mut text_color = view
        .text_color
        .unwrap_or_else(|| default_vertex_text_color(view.color.vertex(app.state.palette_theme)));
    if ui.color_edit_button_srgba(&mut text_color).changed() {
        view.text_color = Some(text_color);
    }

    ui.separator();
    ui.label(
        egui::RichText::new("Geometry")
            .strong()
            .size(app.config.section_font_size()),
    );
    let mut use_default_radius = view.radius.is_none();
    if ui
        .checkbox(&mut use_default_radius, "Use default size")
        .changed()
        && use_default_radius
    {
        view.radius = None;
    }
    if !use_default_radius {
        let radius = view.radius.get_or_insert(app.config.vertex_radius);
        ui.add(egui::DragValue::new(radius).speed(0.5).prefix("radius: "));
    }

    let mut use_default_stroke = view.stroke_width.is_none();
    if ui
        .checkbox(&mut use_default_stroke, "Use default stroke")
        .changed()
        && use_default_stroke
    {
        view.stroke_width = None;
    }
    if !use_default_stroke {
        let stroke = view.stroke_width.get_or_insert(app.config.vertex_stroke);
        ui.add(egui::DragValue::new(stroke).speed(0.25).prefix("stroke: "));
    }
}

fn draw_edge_editor(app: &mut GraphEditorApp, ui: &mut egui::Ui, index: usize) {
    let Some(edge) = app.state.graph.edges.get(index) else {
        app.ui.edit_target = None;
        return;
    };
    let Some(view) = app.state.graph_view.edges.get_mut(index) else {
        app.ui.edit_target = None;
        return;
    };

    ui.label(format!("from: {}", edge.from));
    ui.label(format!("to: {}", edge.to));
    ui.separator();

    ui.label(
        egui::RichText::new("Stroke")
            .strong()
            .size(app.config.section_font_size()),
    );
    draw_color_palette(ui, &mut view.color, app.state.palette_theme);

    ui.separator();
    ui.label(
        egui::RichText::new("Line")
            .strong()
            .size(app.config.section_font_size()),
    );
    draw_line_style_palette(ui, &mut view.line_style);

    let mut use_default_stroke = view.stroke_width.is_none();
    if ui
        .checkbox(&mut use_default_stroke, "Use default width")
        .changed()
        && use_default_stroke
    {
        view.stroke_width = None;
    }
    if !use_default_stroke {
        let stroke = view.stroke_width.get_or_insert(app.config.edge_stroke);
        ui.add(egui::DragValue::new(stroke).speed(0.25).prefix("width: "));
    }
}

fn draw_color_palette(
    ui: &mut egui::Ui,
    color: &mut Colors,
    palette_theme: crate::components::PaletteTheme,
) {
    ui.horizontal_wrapped(|ui| {
        for candidate in COLOR_SLOTS {
            let fill = if candidate == Colors::Default {
                egui::Color32::WHITE
            } else {
                candidate.vertex(palette_theme)
            };
            let stroke_color = if *color == candidate {
                egui::Color32::BLACK
            } else {
                egui::Color32::from_gray(120)
            };
            let response = ui
                .add(
                    egui::Button::new("")
                        .min_size(egui::vec2(24.0, 24.0))
                        .fill(fill)
                        .stroke(egui::Stroke::new(2.0, stroke_color)),
                )
                .on_hover_text(candidate.label());
            if response.clicked() {
                *color = candidate;
            }
        }
    });
}

fn draw_pattern_palette(ui: &mut egui::Ui, pattern: &mut VertexPattern) {
    ui.horizontal_wrapped(|ui| {
        for candidate in VERTEX_PATTERNS {
            let stroke_color = if *pattern == candidate {
                egui::Color32::BLACK
            } else {
                egui::Color32::from_gray(120)
            };
            let response = ui
                .add(
                    egui::Button::new(candidate.label())
                        .min_size(egui::vec2(64.0, 24.0))
                        .stroke(egui::Stroke::new(2.0, stroke_color)),
                )
                .on_hover_text(candidate.label());
            if response.clicked() {
                *pattern = candidate;
            }
        }
    });
}

fn draw_line_style_palette(ui: &mut egui::Ui, line_style: &mut EdgeLineStyle) {
    ui.horizontal_wrapped(|ui| {
        for candidate in EDGE_LINE_STYLES {
            let stroke_color = if *line_style == candidate {
                egui::Color32::BLACK
            } else {
                egui::Color32::from_gray(120)
            };
            let response = ui
                .add(
                    egui::Button::new(candidate.label())
                        .min_size(egui::vec2(64.0, 24.0))
                        .stroke(egui::Stroke::new(2.0, stroke_color)),
                )
                .on_hover_text(candidate.label());
            if response.clicked() {
                *line_style = candidate;
            }
        }
    });
}
