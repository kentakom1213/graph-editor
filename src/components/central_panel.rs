use egui::Context;
use itertools::Itertools;

use crate::{graph::Graph, mode::EditMode, GraphEditorApp};

/// メイン領域を描画
pub fn draw_central_panel(app: &mut GraphEditorApp, ctx: &Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(app.config.bg_color))
        .show(ctx, |ui| {
            let painter = ui.painter();

            // モード切替を行う
            change_edit_mode(app, ui);

            // Indexing切替を行う
            change_indexing(app, ui);

            // クリックした位置に頂点を追加
            add_vertex(app, ui);

            // 辺の描画
            draw_edges(app, ui, painter);

            // 頂点の描画
            draw_vertices(app, ui, painter);
        });
}

/// モード切替の処理
fn change_edit_mode(app: &mut GraphEditorApp, ui: &egui::Ui) {
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        // AddEdgeモードで，片方の頂点が選択済みの場合，選択状態を解除
        if let EditMode::AddEdge {
            from_vertex: ref mut from_vertex @ Some(from_vertex_id),
            ..
        } = app.edit_mode
        {
            if let Some(from_vertex) = app
                .graph
                .vertices_mut()
                .iter_mut()
                .find(|v| v.id == from_vertex_id)
            {
                from_vertex.is_selected = false;
            }
            *from_vertex = None;
        } else {
            app.edit_mode = EditMode::default_normal();
        }
    }
    if ui.input(|i| i.key_pressed(egui::Key::V)) {
        app.edit_mode = EditMode::default_add_vertex();
    }
    if ui.input(|i| i.key_pressed(egui::Key::E)) {
        app.edit_mode = EditMode::default_add_edge();
    }
    if ui.input(|i| i.key_pressed(egui::Key::D)) {
        app.edit_mode = EditMode::default_delete();
    }
}

fn change_indexing(app: &mut GraphEditorApp, ui: &egui::Ui) {
    if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
        app.zero_indexed ^= true;
    }
}

/// クリックした位置に頂点を追加する
fn add_vertex(app: &mut GraphEditorApp, ui: &egui::Ui) {
    // クリックした位置に頂点を追加する
    if app.edit_mode.is_add_vertex()
        && ui.input(|i| i.pointer.any_click())
        && !app.hovered_on_menu_window
        && !app.hovered_on_input_window
    {
        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
            app.graph.add_vertex(mouse_pos, app.next_z_index);
            app.next_z_index += 1;
        }
    }
}

/// central_panel に辺を描画する
fn draw_edges(app: &mut GraphEditorApp, ui: &egui::Ui, painter: &egui::Painter) {
    app.graph.restore_graph();

    let (vertices_mut, edges_mut) = app.graph.vertices_edges_mut();

    for edge in edges_mut.iter_mut() {
        if let (Some(from_vertex), Some(to_vertex)) = (
            vertices_mut.iter().find(|v| v.id == edge.from),
            vertices_mut.iter().find(|v| v.id == edge.to),
        ) {
            // ノーマルモードの場合，エッジの選択判定を行う
            if app.edit_mode.is_delete() {
                let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();
                let edge_vector = to_vertex.position - from_vertex.position;
                let mouse_vector = mouse_pos - from_vertex.position;
                let edge_length = edge_vector.length();

                // 端点との距離
                let distance_from_vertex = (mouse_pos - from_vertex.position)
                    .length()
                    .min((mouse_pos - to_vertex.position).length());

                // カーソルが頂点上にあるかどうか
                let is_on_vertex = distance_from_vertex < app.config.vertex_radius;

                // エッジ上の最近接点を計算する
                let t = (edge_vector.dot(mouse_vector) / edge_length.powi(2)).clamp(0.0, 1.0);
                let nearest_point = from_vertex.position + t * edge_vector;

                // マウスとエッジの最近接点の距離
                let distance = (mouse_pos - nearest_point).length();

                // 当たり判定の閾値 (線の太さ + 余裕分)
                let threshold = 10.0;

                // カーソルが辺上にあるかどうか
                let is_on_edge = distance < threshold;

                if is_on_edge && !is_on_vertex {
                    edge.is_pressed = true;

                    if ui.input(|i| i.pointer.any_click()) {
                        edge.is_deleted = true;
                    }
                } else {
                    edge.is_pressed = false;
                }
            }

            let color = if edge.is_pressed {
                app.config.edge_color_hover
            } else {
                app.config.edge_color_normal
            };

            painter.line_segment(
                [from_vertex.position, to_vertex.position],
                egui::Stroke::new(app.config.edge_stroke, color),
            );
        }
    }
}

/// central_panel に頂点を描画する
fn draw_vertices(app: &mut GraphEditorApp, ui: &egui::Ui, painter: &egui::Painter) {
    app.graph.restore_graph();

    let is_directed = app.graph.is_directed;
    let (vertices_mut, edges_mut) = app.graph.vertices_edges_mut();

    for vertex in vertices_mut.iter_mut().sorted_by_key(|v| v.z_index) {
        let rect = egui::Rect::from_center_size(
            vertex.position,
            egui::vec2(
                app.config.vertex_radius * 2.0,
                app.config.vertex_radius * 2.0,
            ),
        );
        let response = ui.interact(
            rect,
            egui::Id::new(vertex.id),
            egui::Sense::click_and_drag(),
        );

        // ドラッグ開始時の処理
        if response.drag_started() {
            vertex.is_pressed = true;
            vertex.z_index = app.next_z_index;
            app.next_z_index += 1;
            if let Some(mouse_pos) = response.hover_pos() {
                vertex.drag_offset = mouse_pos - vertex.position;
            }
        } else if response.dragged() {
            // ドラッグ中の位置更新
            if let Some(mouse_pos) = response.hover_pos() {
                vertex.position = mouse_pos - vertex.drag_offset;
            }
        } else {
            vertex.is_pressed = false;
        }

        // ホバー時
        if app.edit_mode.is_add_edge() || app.edit_mode.is_delete() {
            vertex.is_pressed = response.hovered();
        }

        // 選択時
        if response.clicked() && !response.dragged() {
            // 最前面に配置
            vertex.z_index = app.next_z_index;
            app.next_z_index += 1;

            match app.edit_mode {
                EditMode::AddEdge {
                    ref mut from_vertex,
                    ref mut confirmed,
                } => {
                    if let Some(from_vertex_inner) = from_vertex {
                        if *from_vertex_inner == vertex.id {
                            // 自分だった場合，選択を解除
                            vertex.is_selected = false;
                            *from_vertex = None;
                        } else {
                            // クリックした頂点をto_vertexに設定（すでに追加されている場合は無視）
                            Graph::add_unique_edge(
                                is_directed,
                                edges_mut,
                                *from_vertex_inner,
                                vertex.id,
                            );
                            *confirmed = true;
                        }
                    } else {
                        vertex.is_selected = true;
                        vertex.z_index = app.next_z_index;
                        app.next_z_index += 1;
                        // クリックした頂点をfrom_vertexに設定
                        *from_vertex = Some(vertex.id);
                    }
                }
                EditMode::Delete => {
                    vertex.is_deleted = true;
                }
                _ => {}
            }
        }

        match app.edit_mode {
            // 始点のみ選択状態の場合，辺を描画
            EditMode::AddEdge {
                from_vertex: Some(from_vertex_inner),
                confirmed: false,
            } => {
                if vertex.id == from_vertex_inner {
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        painter.line_segment(
                            [vertex.position, mouse_pos],
                            egui::Stroke::new(app.config.edge_stroke, app.config.edge_color_normal),
                        );
                    }
                }
            }
            // 辺を選択し終えた場合，状態をリセット
            EditMode::AddEdge {
                from_vertex: ref mut from_vertex @ Some(from_vertex_inner),
                confirmed: ref mut confirmed @ true,
            } => {
                if vertex.id == from_vertex_inner {
                    vertex.is_selected = false;
                    *from_vertex = None;
                    *confirmed = false;
                }
            }
            _ => {}
        }

        // 頂点の色
        let color = if vertex.is_selected {
            app.config.vertex_color_selected
        } else if vertex.is_pressed {
            app.config.vertex_color_dragged
        } else {
            app.config.vertex_color_normal
        };

        // 0-indexed / 1-indexed の選択によってIDを変更
        let vertex_show_id = if app.zero_indexed {
            vertex.id
        } else {
            vertex.id + 1
        }
        .to_string();

        painter.circle_filled(vertex.position, app.config.vertex_radius, color);
        painter.circle_stroke(
            vertex.position,
            app.config.vertex_radius,
            egui::Stroke::new(app.config.vertex_stroke, app.config.vertex_color_outline),
        );
        painter.text(
            vertex.position,
            egui::Align2::CENTER_CENTER,
            vertex_show_id,
            egui::FontId::proportional(app.config.vertex_font_size),
            app.config.vertex_font_color,
        );
    }
}
