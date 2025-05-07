use std::collections::HashMap;

use itertools::Itertools;

use super::transition_and_scale::{drag_central_panel, scale_central_panel};
use crate::{
    config::AppConfig,
    graph::Graph,
    math::{
        affine::ApplyAffine,
        bezier::{
            bezier_curve, calc_bezier_control_point, calc_intersection_of_bezier_and_circle,
            d2_bezier_dt2, d_bezier_dt,
        },
        newton::newton_method,
    },
    mode::EditMode,
    GraphEditorApp,
};

/// メイン領域を描画
pub fn draw_central_panel(app: &mut GraphEditorApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(app.config.bg_color))
        .show(ctx, |ui| {
            // モード切替を行う
            change_edit_mode(app, ui);

            // ドラッグを行う
            drag_central_panel(app, ui);

            // スケールを行う
            scale_central_panel(app, ui);

            // クリックした位置に頂点を追加
            add_vertex(app, ui);

            // ペインターを取得
            let painter = ui.painter();

            // 辺の描画
            draw_edges(app, ui, painter);

            // 頂点の描画
            draw_vertices(app, ui, painter);
        });
}

/// モード切替の処理
fn change_edit_mode(app: &mut GraphEditorApp, ui: &egui::Ui) {
    // 入力中はモード切替を行わない
    if app.hovered_on_input_window {
        return;
    }

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
            app.switch_normal_mode();
        }
    }
    if ui.input(|i| i.key_pressed(egui::Key::V)) {
        app.switch_add_vertex_mode();
    }
    if ui.input(|i| i.key_pressed(egui::Key::E)) {
        app.switch_add_edge_mode();
    }
    if ui.input(|i| i.key_pressed(egui::Key::D)) {
        // Shift + D で無向グラフ/有向グラフを切り替え
        if ui.input(|i| i.modifiers.shift) {
            app.graph.is_directed ^= true;
        } else {
            app.switch_delete_mode();
        }
    }
    if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
        app.zero_indexed ^= true;
    }
    if ui.input(|i| i.key_pressed(egui::Key::A)) {
        // A でグラフのシミュレーションを切り替え
        app.is_animated ^= true;
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

    // 辺をカウントする
    let edge_count = app
        .graph
        .edges()
        .iter()
        .fold(HashMap::new(), |mut map, edge| {
            *map.entry((edge.from, edge.to)).or_insert(0) += 1;
            *map.entry((edge.to, edge.from)).or_insert(0) += 1;
            map
        });

    let is_directed = app.graph.is_directed;
    let (vertices_mut, edges_mut) = app.graph.vertices_edges_mut();

    for edge in edges_mut.iter_mut() {
        if let (Some(from_vertex), Some(to_vertex)) = (
            vertices_mut.iter().find(|v| v.id == edge.from),
            vertices_mut.iter().find(|v| v.id == edge.to),
        ) {
            // ノーマルモードの場合，エッジの選択判定を行う
            if app.edit_mode.is_delete() {
                let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();

                // 端点との距離
                let distance_from_vertex = (mouse_pos - from_vertex.get_position())
                    .length()
                    .min((mouse_pos - to_vertex.get_position()).length());

                // カーソルが頂点上にあるかどうか
                let is_on_vertex = distance_from_vertex < app.config.vertex_radius;

                // マウスと辺の最近接点の距離
                let distance = if !is_directed || edge_count.get(&(edge.from, edge.to)) == Some(&1)
                {
                    distance_from_edge_line(
                        from_vertex.get_position(),
                        to_vertex.get_position(),
                        mouse_pos,
                    )
                } else {
                    distance_from_edge_bezier(
                        from_vertex.get_position(),
                        to_vertex.get_position(),
                        app.config.edge_bezier_distance,
                        mouse_pos,
                    )
                };

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

            let edge_color = if edge.is_pressed {
                app.config.edge_color_hover
            } else {
                app.config.edge_color_normal
            };

            if is_directed {
                if edge_count.get(&(edge.from, edge.to)) == Some(&1) {
                    draw_edge_directed(
                        painter,
                        from_vertex.get_position(),
                        to_vertex.get_position(),
                        edge_color,
                        &app.config,
                    );
                } else {
                    draw_edge_directed_curved(
                        painter,
                        from_vertex.get_position(),
                        to_vertex.get_position(),
                        edge_color,
                        &app.config,
                    );
                }
            } else {
                draw_edge_undirected(
                    painter,
                    from_vertex.get_position(),
                    to_vertex.get_position(),
                    app.config.edge_stroke,
                    edge_color,
                );
            }
        }
    }
}

fn distance_from_edge_line(from_pos: egui::Pos2, to_pos: egui::Pos2, mouse_pos: egui::Pos2) -> f32 {
    let edge_vector = to_pos - from_pos;
    let mouse_vector = mouse_pos - from_pos;
    let edge_length = edge_vector.length();

    let t_ast = (edge_vector.dot(mouse_vector) / edge_length.powi(2)).clamp(0.0, 1.0);
    let nearest_point = from_pos + t_ast * edge_vector;

    (mouse_pos - nearest_point).length()
}

fn distance_from_edge_bezier(
    from_pos: egui::Pos2,
    to_pos: egui::Pos2,
    bezier_distance: f32,
    mouse_pos: egui::Pos2,
) -> f32 {
    let control = calc_bezier_control_point(from_pos, to_pos, bezier_distance, false);

    let bezier = |t: f32| -> egui::Pos2 { bezier_curve(from_pos, control, to_pos, t) };
    let d_bezier = |t: f32| -> egui::Vec2 { d_bezier_dt(from_pos, control, to_pos, t) };
    let dd_bezier = d2_bezier_dt2(from_pos, control, to_pos);

    let d_dist_sq_dt = |t: f32| -> f32 {
        let pt = bezier(t);
        let d_pos = d_bezier(t);
        2.0 * (pt - mouse_pos).dot(d_pos)
    };
    let d2_sqr_dist_dt2 = |t: f32| -> f32 {
        let pt = bezier(t); // (x, y)
        let dp = d_bezier(t); // (dx/dt, dy/dt)
        let ddp = dd_bezier; // (d^2x/dt^2, d^2y/dt^2) for quadratic is constant

        // 2( (dx/dt)^2 + (dy/dt)^2 ) + 2( (x - Mx)*d^2x/dt^2 + (y - My)*d^2y/dt^2 )
        2.0 * dp.length_sq() + 2.0 * (pt - mouse_pos).dot(ddp)
    };

    let t_ast = newton_method(d_dist_sq_dt, d2_sqr_dist_dt2, 0.5, 1e-6, 10);

    t_ast
        .filter(|&t| (0.0..=1.0).contains(&t))
        .map(|t| bezier(t).distance(mouse_pos))
        .unwrap_or(f32::INFINITY)
}

fn draw_edge_undirected(
    painter: &egui::Painter,
    from_pos: egui::Pos2,
    to_pos: egui::Pos2,
    stroke: f32,
    color: egui::Color32,
) {
    painter.line_segment([from_pos, to_pos], egui::Stroke::new(stroke, color));
}

fn draw_edge_directed(
    painter: &egui::Painter,
    from_pos: egui::Pos2,
    to_pos: egui::Pos2,
    color: egui::Color32,
    config: &AppConfig,
) {
    // 矢印の方向を取得
    let dir = (to_pos - from_pos).normalized();
    let arrowhead = to_pos - dir * config.vertex_radius;
    let endpoint = arrowhead - dir * config.edge_arrow_length;

    // 矢印のヘッド（三角形）の3つの頂点を計算
    let dir = dir * config.edge_arrow_length;
    let left = egui::Pos2::new(
        arrowhead.x - dir.x - dir.y * (config.edge_arrow_width / config.edge_arrow_length),
        arrowhead.y - dir.y + dir.x * (config.edge_arrow_width / config.edge_arrow_length),
    );
    let right = egui::Pos2::new(
        arrowhead.x - dir.x + dir.y * (config.edge_arrow_width / config.edge_arrow_length),
        arrowhead.y - dir.y - dir.x * (config.edge_arrow_width / config.edge_arrow_length),
    );

    // 三角形を描画
    painter.add(egui::Shape::convex_polygon(
        vec![arrowhead, left, right],
        color,
        egui::Stroke::NONE,
    ));

    // 線を描画
    painter.line_segment(
        [from_pos, endpoint],
        egui::Stroke::new(config.edge_stroke, color),
    );
}

/// 曲線付きの矢印を描画する関数
fn draw_edge_directed_curved(
    painter: &egui::Painter,
    from_pos: egui::Pos2,
    to_pos: egui::Pos2,
    color: egui::Color32,
    config: &AppConfig,
) -> Option<()> {
    let control = calc_bezier_control_point(from_pos, to_pos, config.edge_bezier_distance, false);

    // ベジェ曲線と円の交点を計算
    let (arrowhead, dir) = calc_intersection_of_bezier_and_circle(
        from_pos,
        control,
        to_pos,
        to_pos,
        config.vertex_radius,
    )?;

    // 2次ベジェ曲線を描画
    let bezier = epaint::QuadraticBezierShape {
        points: [from_pos, control, to_pos], // 始点・制御点・終点
        closed: false,
        fill: egui::Color32::TRANSPARENT,
        stroke: epaint::PathStroke::new(config.edge_stroke, color),
    };
    painter.add(bezier);

    // 矢印のヘッドに曲線が重ならないよう，マスクを作成
    painter.line_segment(
        [
            arrowhead - dir.normalized() * config.edge_arrow_length / 2.0,
            arrowhead,
        ],
        egui::Stroke::new(config.edge_stroke, config.bg_color),
    );

    // 矢印のヘッド（三角形）の3つの頂点を計算
    let dir = dir.normalized() * config.edge_arrow_length;
    let left = egui::Pos2::new(
        arrowhead.x - dir.x - dir.y * (config.edge_arrow_width / config.edge_arrow_length),
        arrowhead.y - dir.y + dir.x * (config.edge_arrow_width / config.edge_arrow_length),
    );
    let right = egui::Pos2::new(
        arrowhead.x - dir.x + dir.y * (config.edge_arrow_width / config.edge_arrow_length),
        arrowhead.y - dir.y - dir.x * (config.edge_arrow_width / config.edge_arrow_length),
    );

    // 三角形を描画
    painter.add(egui::Shape::convex_polygon(
        vec![arrowhead, left, right],
        color,
        egui::Stroke::NONE,
    ));

    Some(())
}

/// central_panel に頂点を描画する
fn draw_vertices(app: &mut GraphEditorApp, ui: &egui::Ui, painter: &egui::Painter) {
    // グラフの更新
    app.graph.restore_graph();

    // シミュレーションがonの場合，位置を更新
    if app.is_animated {
        app.graph.simulate_step(&app.config.simulate_config);
    }

    let is_directed = app.graph.is_directed;
    let (vertices_mut, edges_mut) = app.graph.vertices_edges_mut();

    for vertex in vertices_mut.iter_mut().sorted_by_key(|v| v.z_index) {
        let rect = egui::Rect::from_center_size(
            vertex.get_position(),
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
                let scale = vertex.affine().scale_x();
                let delta = (mouse_pos - vertex.get_position()) / scale;
                vertex.drag_offset = delta;
            }
        } else if response.dragged() {
            // ドラッグ中の位置更新
            if let Some(mouse_pos) = response.hover_pos() {
                vertex.update_position(mouse_pos - vertex.drag_offset);
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
                            [vertex.get_position(), mouse_pos],
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

        painter.circle_filled(vertex.get_position(), app.config.vertex_radius, color);
        painter.circle_stroke(
            vertex.get_position(),
            app.config.vertex_radius,
            egui::Stroke::new(app.config.vertex_stroke, app.config.vertex_color_outline),
        );
        painter.text(
            vertex.get_position(),
            egui::Align2::CENTER_CENTER,
            vertex_show_id,
            egui::FontId::proportional(app.config.vertex_font_size),
            app.config.vertex_font_color,
        );
    }
}
