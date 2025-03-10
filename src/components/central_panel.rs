use egui::Context;
use itertools::Itertools;

use crate::{graph::Graph, mode::EditMode, GraphEditorApp};

/// メイン領域を描画
pub fn draw_central_panel(app: &mut GraphEditorApp, ctx: &Context) {
    let bg_color = egui::Color32::from_rgb(230, 230, 230);

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(bg_color))
        .show(ctx, |ui| {
            let painter = ui.painter();
            let radius = 50.0;

            // モード切替を行う
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.edit_mode = EditMode::default_normal();
            }
            if ui.input(|i| i.key_pressed(egui::Key::V)) {
                app.edit_mode = EditMode::default_add_vertex();
            }
            if ui.input(|i| i.key_pressed(egui::Key::E)) {
                app.edit_mode = EditMode::default_add_edge();
            }
            if ui.input(|i| i.key_pressed(egui::Key::D)) {
                app.edit_mode = EditMode::default_delete_edge();
            }

            if let EditMode::AddEdge {
                from_vertex: ref mut from_vertex @ Some(from_vertex_id),
                ..
            } = app.edit_mode
            {
                // Escapeキーで選択頂点を解除
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    if let Some(from_vertex) = app
                        .graph
                        .vertices_mut()
                        .iter_mut()
                        .find(|v| v.id == from_vertex_id)
                    {
                        from_vertex.is_selected = false;
                    }
                    *from_vertex = None;
                }
            }

            // クリックした位置に頂点を追加する
            if app.edit_mode.is_add_vertex() && ui.input(|i| i.pointer.any_click()) {
                if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    app.graph.add_vertex(mouse_pos, app.next_z_index);
                    app.next_z_index += 1;
                }
            }

            let (vertices_mut, edges_mut) = app.graph.vertices_edges_mut();

            // 削除済み辺の削除
            edges_mut.retain(|edge| !edge.is_deleted);

            // エッジの描画
            for edge in edges_mut.iter_mut() {
                if let (Some(from_vertex), Some(to_vertex)) = (
                    vertices_mut.iter().find(|v| v.id == edge.from),
                    vertices_mut.iter().find(|v| v.id == edge.to),
                ) {
                    // ノーマルモードの場合，エッジの選択判定を行う
                    if app.edit_mode.is_delete_edge() {
                        let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();
                        let edge_vector = to_vertex.position - from_vertex.position;
                        let mouse_vector = mouse_pos - from_vertex.position;
                        let edge_length = edge_vector.length();

                        // エッジ上の最近接点を計算する
                        let t =
                            (edge_vector.dot(mouse_vector) / edge_length.powi(2)).clamp(0.0, 1.0);
                        let nearest_point = from_vertex.position + t * edge_vector;

                        // マウスとエッジの最近接点の距離
                        let distance = (mouse_pos - nearest_point).length();

                        // 当たり判定の閾値 (線の太さ + 余裕分)
                        let threshold = 10.0;

                        if distance < threshold {
                            edge.is_pressed = true;

                            if ui.input(|i| i.pointer.any_click()) {
                                edge.is_deleted = true;
                            }
                        } else {
                            edge.is_pressed = false;
                        }
                    }

                    let color = if edge.is_deleted {
                        bg_color
                    } else if edge.is_pressed {
                        egui::Color32::from_rgb(200, 100, 100)
                    } else {
                        egui::Color32::from_rgb(100, 100, 100)
                    };

                    painter.line_segment(
                        [from_vertex.position, to_vertex.position],
                        egui::Stroke::new(6.0, color),
                    );
                }
            }

            // 頂点の描画
            for vertex in vertices_mut.iter_mut().sorted_by_key(|v| v.z_index) {
                let rect = egui::Rect::from_center_size(
                    vertex.position,
                    egui::vec2(radius * 2.0, radius * 2.0),
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
                if app.edit_mode.is_add_edge() {
                    vertex.is_pressed = response.hovered();
                }

                // 選択時
                if response.clicked() && !response.dragged() {
                    // 最前面に配置
                    vertex.z_index = app.next_z_index;
                    app.next_z_index += 1;

                    if let EditMode::AddEdge {
                        ref mut from_vertex,
                        ref mut confirmed,
                    } = app.edit_mode
                    {
                        if let Some(from_vertex_inner) = from_vertex {
                            if *from_vertex_inner == vertex.id {
                                // 自分だった場合，選択を解除
                                vertex.is_selected = false;
                                *from_vertex = None;
                            } else {
                                // クリックした頂点をto_vertexに設定（すでに追加されている場合は無視）
                                Graph::add_unique_edge_undirected(
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
                                    egui::Stroke::new(6.0, egui::Color32::from_rgb(100, 100, 100)),
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

                let color = if vertex.is_pressed {
                    egui::Color32::from_rgb(200, 100, 100) // ドラッグ中は赤色
                } else if vertex.is_selected {
                    egui::Color32::from_rgb(100, 200, 100) // 選択状態は緑色
                } else {
                    egui::Color32::WHITE // 通常時は白色
                };

                // 0-indexed / 1-indexed の選択によってIDを変更
                let vertex_show_id = if app.zero_indexed {
                    vertex.id
                } else {
                    vertex.id + 1
                }
                .to_string();

                painter.circle_filled(vertex.position, radius, color);
                painter.circle_stroke(
                    vertex.position,
                    radius,
                    egui::Stroke::new(3.0, egui::Color32::from_rgb(150, 150, 150)),
                );
                painter.text(
                    vertex.position,
                    egui::Align2::CENTER_CENTER,
                    vertex_show_id,
                    egui::FontId::proportional(50.0),
                    egui::Color32::BLACK,
                );
            }
        });
}
