use egui::{text::LayoutJob, Color32, Context, FontId, TextFormat};

use crate::{
    graph::BaseGraph, project_io::import_graph_from_json, state::IoFormat, GraphEditorApp,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorTab {
    #[default]
    Graph,
    Io,
}

pub fn draw_inspector_panel(app: &mut GraphEditorApp, ctx: &Context) {
    egui::SidePanel::right("inspector_panel")
        .show_separator_line(false)
        .resizable(true)
        .min_width(196.0)
        .show(ctx, |ui| {
            app.ui
                .cursor_hover
                .set_inspector_panel(ui.rect_contains_pointer(ui.max_rect()));

            ui.columns(2, |columns| {
                draw_tab_button(
                    &mut columns[0],
                    app.ui.inspector_tab == InspectorTab::Graph,
                    "Graph",
                    app.config.tab_font_size(),
                    || app.ui.inspector_tab = InspectorTab::Graph,
                );
                draw_tab_button(
                    &mut columns[1],
                    app.ui.inspector_tab == InspectorTab::Io,
                    "I/O",
                    app.config.tab_font_size(),
                    || app.ui.inspector_tab = InspectorTab::Io,
                );
            });
            ui.separator();

            match app.ui.inspector_tab {
                InspectorTab::Graph => draw_graph_tab(app, ctx, ui),
                InspectorTab::Io => draw_io_tab(app, ctx, ui),
            }
        });
}

fn draw_graph_tab(app: &mut GraphEditorApp, ctx: &Context, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Indexing").size(app.config.section_font_size()));
    draw_toggle_button(
        ui,
        app.state.zero_indexed,
        "0-indexed",
        app.config.button_font_size(),
        || app.state.zero_indexed = true,
    );
    draw_toggle_button(
        ui,
        !app.state.zero_indexed,
        "1-indexed",
        app.config.button_font_size(),
        || app.state.zero_indexed = false,
    );

    ui.separator();
    ui.label(egui::RichText::new("Direction").size(app.config.section_font_size()));
    draw_toggle_button(
        ui,
        !app.state.graph.is_directed,
        "Undirected",
        app.config.button_font_size(),
        || app.state.graph.is_directed = false,
    );
    draw_toggle_button(
        ui,
        app.state.graph.is_directed,
        "Directed",
        app.config.button_font_size(),
        || app.state.graph.is_directed = true,
    );

    ui.separator();
    ui.label(egui::RichText::new("Operations").size(app.config.section_font_size()));

    if ui
        .add_enabled(
            !app.state.graph.is_directed,
            egui::Button::new(
                egui::RichText::new("Complement").size(app.config.button_font_size()),
            ),
        )
        .clicked()
    {
        app.rebuild_from_base_graph(ctx, app.state.graph.calc_complement());
    }

    if ui
        .add_enabled(
            app.state.graph.is_directed,
            egui::Button::new(
                egui::RichText::new("Revert Edge").size(app.config.button_font_size()),
            ),
        )
        .clicked()
    {
        app.rebuild_from_base_graph(ctx, app.state.graph.calc_reverted());
    }

    if ui
        .button(egui::RichText::new("Reset Colors").size(app.config.button_font_size()))
        .clicked()
    {
        app.state.graph_view.reset_colors();
    }

    ui.separator();
    ui.label(egui::RichText::new("Danger Zone").size(app.config.section_font_size()));
    if ui
        .button(egui::RichText::new("Clear All").size(app.config.button_font_size()))
        .clicked()
    {
        app.ui.confirm_clear_all = true;
    }
}

fn draw_io_tab(app: &mut GraphEditorApp, ctx: &Context, ui: &mut egui::Ui) {
    const GRAPH_TEXT_EDITOR_HEIGHT: f32 = 280.0;

    ui.horizontal(|ui| {
        draw_text_mode_button(
            ui,
            app.ui.io_format == IoFormat::EdgeList,
            "Edge List",
            app.config.body_font_size(),
            || {
                app.ui.io_format = IoFormat::EdgeList;
                if !app.ui.input_is_dirty {
                    app.sync_input_text_from_graph();
                }
            },
        );
        draw_text_mode_button(
            ui,
            app.ui.io_format == IoFormat::Json,
            "JSON",
            app.config.body_font_size(),
            || {
                app.ui.io_format = IoFormat::Json;
                if !app.ui.json_is_dirty {
                    app.sync_json_text_from_graph();
                }
            },
        );
    });
    ui.separator();

    match app.ui.io_format {
        IoFormat::EdgeList => draw_edge_list_io(app, ctx, ui, GRAPH_TEXT_EDITOR_HEIGHT),
        IoFormat::Json => draw_json_io(app, ctx, ui, GRAPH_TEXT_EDITOR_HEIGHT),
    }

    ui.separator();
    ui.label(egui::RichText::new("Export Image").size(app.config.section_font_size()));
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Format").size(app.config.body_font_size()));
        let mut format = app.export.format();
        egui::ComboBox::from_id_salt("inspector_export_format")
            .selected_text(match format {
                crate::export::ExportFormat::Png => {
                    egui::RichText::new("PNG").size(app.config.body_font_size())
                }
                crate::export::ExportFormat::Svg => {
                    egui::RichText::new("SVG").size(app.config.body_font_size())
                }
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut format,
                    crate::export::ExportFormat::Png,
                    egui::RichText::new("PNG").size(app.config.body_font_size()),
                );
                ui.selectable_value(
                    &mut format,
                    crate::export::ExportFormat::Svg,
                    egui::RichText::new("SVG").size(app.config.body_font_size()),
                );
            });
        app.export.set_format(format);
    });

    if ui
        .add_enabled(
            !app.export.is_busy(),
            egui::Button::new(egui::RichText::new("Export").size(app.config.button_font_size())),
        )
        .clicked()
    {
        app.request_export_image(ctx);
    }
}

fn draw_edge_list_io(
    app: &mut GraphEditorApp,
    ctx: &Context,
    ui: &mut egui::Ui,
    editor_height: f32,
) {
    if !app.ui.input_has_focus && !app.ui.input_is_dirty {
        app.sync_input_text_from_graph();
    }

    ui.label(egui::RichText::new("Graph Text").size(app.config.section_font_size()));
    ui.horizontal(|ui| {
        if ui
            .button(egui::RichText::new("Copy").size(app.config.button_font_size()))
            .clicked()
        {
            ctx.copy_text(app.ui.input_text.clone());
        }

        if ui
            .button(egui::RichText::new("Apply").size(app.config.button_font_size()))
            .clicked()
        {
            let new_graph = BaseGraph::parse(&app.ui.input_text, app.state.zero_indexed);
            match new_graph {
                Ok(base_graph) => app.rebuild_from_base_graph(ctx, base_graph),
                Err(err) => app.ui.error_message = Some(err.to_string()),
            }
        }
    });

    ui.separator();
    let editor = egui::TextEdit::multiline(&mut app.ui.input_text)
        .font(egui::FontId::monospace(app.config.input_font_size()))
        .desired_rows(10)
        .desired_width(f32::INFINITY);
    let response = egui::ScrollArea::vertical()
        .id_salt("graph_text_editor_scroll")
        .max_height(editor_height)
        .show(ui, |ui| {
            ui.add_sized([ui.available_width(), editor_height], editor)
        })
        .inner;
    app.ui.input_has_focus = response.has_focus();
    app.ui.input_is_dirty = app.ui.input_text != app.ui.input_synced_text;
}

fn draw_json_io(app: &mut GraphEditorApp, ctx: &Context, ui: &mut egui::Ui, editor_height: f32) {
    if !app.ui.input_has_focus && !app.ui.json_is_dirty {
        app.sync_json_text_from_graph();
    }

    ui.label(egui::RichText::new("Graph JSON").size(app.config.section_font_size()));
    ui.checkbox(
        &mut app.ui.save_vertex_position,
        egui::RichText::new("Save vertex positions").size(app.config.body_font_size()),
    );
    ui.checkbox(
        &mut app.ui.save_vertex_style,
        egui::RichText::new("Save vertex colors").size(app.config.body_font_size()),
    );
    ui.checkbox(
        &mut app.ui.save_edge_style,
        egui::RichText::new("Save edge colors").size(app.config.body_font_size()),
    );

    if !app.ui.json_is_dirty {
        app.sync_json_text_from_graph();
    }

    ui.horizontal_wrapped(|ui| {
        if ui
            .button(egui::RichText::new("Copy").size(app.config.button_font_size()))
            .clicked()
        {
            ctx.copy_text(app.ui.json_text.clone());
        }

        if ui
            .button(egui::RichText::new("Apply").size(app.config.button_font_size()))
            .clicked()
        {
            match import_graph_from_json(&app.ui.json_text) {
                Ok(imported) => app.apply_imported_graph(ctx, imported),
                Err(err) => app.ui.error_message = Some(err.to_string()),
            }
        }
    });

    ui.separator();
    let font_size = app.config.input_font_size();
    let mut layouter = move |ui: &egui::Ui, string: &str, wrap_width: f32| {
        let mut layout_job = json_highlight_layout_job(string, font_size);
        let _ = wrap_width;
        layout_job.wrap.max_width = f32::INFINITY;
        ui.fonts(|fonts| fonts.layout_job(layout_job))
    };
    let editor = egui::TextEdit::multiline(&mut app.ui.json_text)
        .font(egui::FontId::monospace(app.config.input_font_size()))
        .desired_rows(10)
        .desired_width(f32::INFINITY)
        .layouter(&mut layouter);
    let response = egui::ScrollArea::vertical()
        .id_salt("graph_json_editor_scroll")
        .max_height(editor_height)
        .show(ui, |ui| {
            ui.add_sized([ui.available_width(), editor_height], editor)
        })
        .inner;
    app.ui.input_has_focus = response.has_focus();
    app.ui.json_is_dirty = app.ui.json_text != app.ui.json_synced_text;
}

fn draw_toggle_button(
    ui: &mut egui::Ui,
    selected: bool,
    label: &str,
    font_size: f32,
    on_click: impl FnOnce(),
) {
    if ui
        .add_sized(
            [170.0, 32.0],
            egui::SelectableLabel::new(selected, egui::RichText::new(label).size(font_size)),
        )
        .clicked()
    {
        on_click();
    }
}

fn draw_tab_button(
    ui: &mut egui::Ui,
    selected: bool,
    label: &str,
    font_size: f32,
    on_click: impl FnOnce(),
) {
    if ui
        .add_sized(
            [ui.available_width(), 30.0],
            egui::SelectableLabel::new(selected, egui::RichText::new(label).size(font_size)),
        )
        .clicked()
    {
        on_click();
    }
}

fn draw_text_mode_button(
    ui: &mut egui::Ui,
    selected: bool,
    label: &str,
    font_size: f32,
    on_click: impl FnOnce(),
) {
    if ui
        .add_sized(
            [80.0, 28.0],
            egui::SelectableLabel::new(selected, egui::RichText::new(label).size(font_size)),
        )
        .clicked()
    {
        on_click();
    }
}

fn json_highlight_layout_job(text: &str, font_size: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut chars = text.chars().peekable();
    let default = TextFormat {
        font_id: FontId::monospace(font_size),
        color: Color32::from_gray(220),
        ..Default::default()
    };
    let key = TextFormat {
        font_id: FontId::monospace(font_size),
        color: Color32::from_rgb(120, 190, 255),
        ..Default::default()
    };
    let string = TextFormat {
        font_id: FontId::monospace(font_size),
        color: Color32::from_rgb(170, 220, 140),
        ..Default::default()
    };
    let number = TextFormat {
        font_id: FontId::monospace(font_size),
        color: Color32::from_rgb(255, 200, 120),
        ..Default::default()
    };
    let keyword = TextFormat {
        font_id: FontId::monospace(font_size),
        color: Color32::from_rgb(255, 140, 140),
        ..Default::default()
    };
    let punctuation = TextFormat {
        font_id: FontId::monospace(font_size),
        color: Color32::from_rgb(180, 180, 180),
        ..Default::default()
    };

    while let Some(ch) = chars.next() {
        if ch == '"' {
            let mut token = String::from(ch);
            let mut escaped = false;
            for next in chars.by_ref() {
                token.push(next);
                if escaped {
                    escaped = false;
                    continue;
                }
                if next == '\\' {
                    escaped = true;
                    continue;
                }
                if next == '"' {
                    break;
                }
            }

            let format = if matches!(chars.peek(), Some(':')) {
                key.clone()
            } else {
                string.clone()
            };
            job.append(&token, 0.0, format);
            continue;
        }

        if ch.is_ascii_digit() || ch == '-' {
            let mut token = String::from(ch);
            while let Some(next) = chars.peek() {
                if next.is_ascii_digit() || matches!(next, '.' | 'e' | 'E' | '+' | '-') {
                    token.push(*next);
                    chars.next();
                } else {
                    break;
                }
            }
            job.append(&token, 0.0, number.clone());
            continue;
        }

        if ch.is_ascii_alphabetic() {
            let mut token = String::from(ch);
            while let Some(next) = chars.peek() {
                if next.is_ascii_alphabetic() {
                    token.push(*next);
                    chars.next();
                } else {
                    break;
                }
            }
            let format = match token.as_str() {
                "true" | "false" | "null" => keyword.clone(),
                _ => default.clone(),
            };
            job.append(&token, 0.0, format);
            continue;
        }

        let format = if matches!(ch, '{' | '}' | '[' | ']' | ':' | ',') {
            punctuation.clone()
        } else {
            default.clone()
        };
        let mut token = String::new();
        token.push(ch);
        job.append(&token, 0.0, format);
    }

    job
}
