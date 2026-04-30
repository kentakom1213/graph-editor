use std::collections::HashMap;

use itertools::Itertools;

use super::transition_and_scale::{drag_central_panel, scale_central_panel};
use crate::{
    components::Colors,
    config::AppConfig,
    graph::Graph,
    math::{
        affine::{Affine2D, ApplyAffine},
        bezier::{
            bezier_curve, calc_bezier_control_point, calc_intersection_of_bezier_and_circle,
            d2_bezier_dt2, d_bezier_dt,
        },
        newton::newton_method,
    },
    mode::EditMode,
    state::AppState,
    view_state::GraphSnapshot,
    GraphEditorApp,
};

/// メイン領域を描画
pub fn draw_central_panel(app: &mut GraphEditorApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(app.config.bg_color))
        .show(ctx, |ui| {
            app.ui.canvas_rect = Some(ui.max_rect());

            // モード切替を行う
            change_edit_mode(app, ui);

            // ドラッグを行う
            drag_central_panel(app, ui);

            // スケールを行う
            scale_central_panel(app, ui);

            // クリックした位置に頂点を追加
            add_vertex(app, ui);

            // 入力に応じた操作を行う
            update_edge_interactions(app, ui);
            update_vertex_interactions(app, ui);

            // シミュレーションがonの場合，位置を更新
            if app.state.is_animated {
                app.current_simulator().simulate_step(&mut app.state.graph);
            }

            // 描画用スナップショットを作成
            let snapshot = app.state.graph_view.snapshot(&app.state.graph);

            // ペインターを取得
            let painter = ui.painter();

            // 辺の描画
            render_edges(&snapshot, painter, &app.config);

            // 頂点の描画
            render_vertices(&snapshot, app, ui, painter);
        });
}

/// モード切替の処理
fn change_edit_mode(app: &mut GraphEditorApp, ui: &egui::Ui) {
    // 入力中はモード切替を行わない
    if app.ui.input_has_focus {
        return;
    }

    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        // AddEdgeモードで，片方の頂点が選択済みの場合，選択状態を解除
        if let EditMode::AddEdge {
            from_vertex: ref mut from_vertex @ Some(from_vertex_id),
            ..
        } = app.state.edit_mode
        {
            if let Some(view) = app.state.graph_view.vertices.get_mut(from_vertex_id) {
                view.is_selected = false;
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
    if ui.input(|i| i.key_pressed(egui::Key::C)) {
        app.switch_colorize_mode();
    }
    if ui.input(|i| i.key_pressed(egui::Key::D)) {
        // Shift + D で無向グラフ/有向グラフを切り替え
        if ui.input(|i| i.modifiers.shift) {
            app.state.graph.is_directed ^= true;
        } else {
            app.switch_delete_mode();
        }
    }
    if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
        app.state.zero_indexed ^= true;
    }
    if ui.input(|i| i.key_pressed(egui::Key::A)) {
        // A でグラフのシミュレーションを切り替え
        app.set_animation_enabled(!app.state.is_animated);
    }
}

/// クリックした位置に頂点を追加する
fn add_vertex(app: &mut GraphEditorApp, ui: &egui::Ui) {
    // クリックした位置に頂点を追加する
    if app.state.edit_mode.is_add_vertex()
        && ui.input(|i| i.pointer.any_click())
        && !app.ui.cursor_hover.any()
    {
        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let affine = app.state.graph.affine.borrow().to_owned();
            if let Some(inv) = affine.inverse() {
                let scaled_pos = mouse_pos.applied(&inv);
                let pos = scaled_pos + affine.translation();

                let z_index = app.state.next_z_index;
                app.state.graph.add_vertex(pos);
                app.state.graph_view.add_vertex(z_index);
                app.state.next_z_index += 1;
            }
        }
    }
}

/// 辺の操作を更新する
fn update_edge_interactions(app: &mut GraphEditorApp, ui: &egui::Ui) {
    if !app.state.edit_mode.is_delete() && !app.state.edit_mode.is_colorize() {
        return;
    }

    let is_directed = app.state.graph.is_directed;
    let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();
    let vertex_radius = app
        .config
        .effective_vertex_radius(app.state.graph.vertices.len());

    let vertex_positions: HashMap<usize, egui::Pos2> = app
        .state
        .graph
        .vertices
        .iter()
        .filter(|v| !v.is_deleted)
        .map(|v| (v.id, v.get_position()))
        .collect();

    let edge_count = app
        .state
        .graph
        .edges()
        .iter()
        .filter(|edge| !edge.is_deleted)
        .filter(|edge| vertex_positions.contains_key(&edge.from))
        .filter(|edge| vertex_positions.contains_key(&edge.to))
        .fold(HashMap::new(), |mut map, edge| {
            *map.entry((edge.from, edge.to)).or_insert(0) += 1;
            *map.entry((edge.to, edge.from)).or_insert(0) += 1;
            map
        });

    for (index, edge) in app.state.graph.edges_mut().iter_mut().enumerate() {
        let Some(view) = app.state.graph_view.edges.get_mut(index) else {
            continue;
        };
        if edge.is_deleted {
            continue;
        }

        let (Some(&from_pos), Some(&to_pos)) = (
            vertex_positions.get(&edge.from),
            vertex_positions.get(&edge.to),
        ) else {
            view.is_pressed = false;
            continue;
        };

        let distance_from_vertex = (mouse_pos - from_pos)
            .length()
            .min((mouse_pos - to_pos).length());
        let is_on_vertex = distance_from_vertex < vertex_radius;

        let distance = if !is_directed || edge_count.get(&(edge.from, edge.to)) == Some(&1) {
            distance_from_edge_line(from_pos, to_pos, mouse_pos)
        } else {
            distance_from_edge_bezier(from_pos, to_pos, app.config.edge_bezier_distance, mouse_pos)
        };

        let threshold = 10.0;
        let is_on_edge = distance < threshold;

        if is_on_edge && !is_on_vertex {
            view.is_pressed = true;

            if ui.input(|i| i.pointer.any_click()) {
                if app.state.edit_mode.is_colorize() {
                    view.color = app.state.selected_color;
                } else if app.state.edit_mode.is_delete() {
                    edge.is_deleted = true;
                }
            }
        } else {
            view.is_pressed = false;
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

/// 頂点の操作を更新する
fn update_vertex_interactions(app: &mut GraphEditorApp, ui: &egui::Ui) {
    let vertex_radius = app
        .config
        .effective_vertex_radius(app.state.graph.vertices.len());
    let AppState {
        graph,
        graph_view,
        edit_mode,
        selected_color,
        next_z_index,
        ..
    } = &mut app.state;
    let is_directed = graph.is_directed;

    {
        let mut indices: Vec<usize> = graph
            .vertices
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.is_deleted)
            .map(|(idx, _)| idx)
            .collect();
        indices.sort_by_key(|idx| {
            graph_view
                .vertices
                .get(*idx)
                .map(|v| v.z_index)
                .unwrap_or(0)
        });

        let (vertices_mut, edges_mut) = graph.vertices_edges_mut();

        for idx in indices {
            let vertex = &mut vertices_mut[idx];
            let Some(view) = graph_view.vertices.get_mut(idx) else {
                continue;
            };
            let rect = egui::Rect::from_center_size(
                vertex.get_position(),
                egui::vec2(vertex_radius * 2.0, vertex_radius * 2.0),
            );
            let response = ui.interact(
                rect,
                egui::Id::new(vertex.id),
                egui::Sense::click_and_drag(),
            );

            if response.drag_started() {
                view.is_pressed = true;
                view.z_index = *next_z_index;
                *next_z_index += 1;
                if let Some(mouse_pos) = response.hover_pos() {
                    let delta = Affine2D::from_transition(mouse_pos - vertex.get_position());
                    view.drag = delta;
                }
            } else if response.dragged() {
                if let Some(mouse_pos) = response.hover_pos() {
                    vertex.update_position(mouse_pos.applied(&view.drag));
                }
            } else {
                view.is_pressed = false;
            }

            if edit_mode.is_add_edge() || edit_mode.is_delete() {
                view.is_pressed = response.hovered();
            }

            if response.clicked() && !response.dragged() {
                view.z_index = *next_z_index;
                *next_z_index += 1;

                match edit_mode {
                    EditMode::AddEdge {
                        ref mut from_vertex,
                        ref mut confirmed,
                    } => {
                        if let Some(from_vertex_inner) = from_vertex {
                            if *from_vertex_inner == vertex.id {
                                view.is_selected = false;
                                *from_vertex = None;
                            } else {
                                let added = Graph::add_unique_edge(
                                    is_directed,
                                    edges_mut,
                                    *from_vertex_inner,
                                    vertex.id,
                                );
                                if added {
                                    graph_view.add_edge();
                                }
                                *confirmed = true;
                            }
                        } else {
                            view.is_selected = true;
                            view.z_index = *next_z_index;
                            *next_z_index += 1;
                            *from_vertex = Some(vertex.id);
                        }
                    }
                    EditMode::Colorize => {
                        view.color = *selected_color;
                    }
                    EditMode::Delete => {
                        vertex.is_deleted = true;
                    }
                    _ => {}
                }
            }
        }
    }

    if let EditMode::AddEdge {
        from_vertex: ref mut from_vertex @ Some(from_vertex_inner),
        confirmed: ref mut confirmed @ true,
    } = app.state.edit_mode
    {
        if let Some(vertex) = app.state.graph_view.vertices.get_mut(from_vertex_inner) {
            vertex.is_selected = false;
        }
        *from_vertex = None;
        *confirmed = false;
    }
}

/// central_panel に辺を描画する
fn render_edges(snapshot: &GraphSnapshot, painter: &egui::Painter, config: &AppConfig) {
    let vertex_positions: HashMap<usize, egui::Pos2> = snapshot
        .vertices
        .iter()
        .map(|v| (v.id, v.position))
        .collect();

    let edge_count = snapshot.edges.iter().fold(HashMap::new(), |mut map, edge| {
        *map.entry((edge.from, edge.to)).or_insert(0) += 1;
        *map.entry((edge.to, edge.from)).or_insert(0) += 1;
        map
    });

    for edge in snapshot.edges.iter() {
        let (Some(&from_pos), Some(&to_pos)) = (
            vertex_positions.get(&edge.from),
            vertex_positions.get(&edge.to),
        ) else {
            continue;
        };

        let edge_color = if edge.is_pressed {
            config.edge_color_hover
        } else {
            edge.color.edge()
        };

        if snapshot.is_directed {
            if edge_count.get(&(edge.from, edge.to)) == Some(&1) {
                draw_edge_directed(painter, from_pos, to_pos, edge_color, config);
            } else {
                draw_edge_directed_curved(painter, from_pos, to_pos, edge_color, config);
            }
        } else {
            draw_edge_undirected(painter, from_pos, to_pos, config.edge_stroke, edge_color);
        }
    }
}

/// central_panel に頂点を描画する
fn render_vertices(
    snapshot: &GraphSnapshot,
    app: &GraphEditorApp,
    ui: &egui::Ui,
    painter: &egui::Painter,
) {
    let vertex_radius = app.config.effective_vertex_radius(snapshot.vertices.len());
    let vertex_font_size = app
        .config
        .effective_vertex_font_size(snapshot.vertices.len());

    // 設置途中の辺を描画
    if let EditMode::AddEdge {
        from_vertex: Some(from_vertex_inner),
        confirmed: false,
    } = app.state.edit_mode
    {
        let from_pos = snapshot
            .vertices
            .iter()
            .find(|v| v.id == from_vertex_inner)
            .map(|v| v.position);

        if let (Some(from_pos), Some(mouse_pos)) = (from_pos, ui.input(|i| i.pointer.hover_pos())) {
            painter.line_segment(
                [from_pos, mouse_pos],
                egui::Stroke::new(app.config.edge_stroke, Colors::Default.edge()),
            );
        }
    }

    for vertex in snapshot.vertices.iter().sorted_by_key(|v| v.z_index) {
        let color = if vertex.is_selected {
            app.config.vertex_color_selected
        } else if vertex.is_pressed {
            app.config.vertex_color_dragged
        } else {
            vertex.color.vertex()
        };

        painter.circle_filled(vertex.position, vertex_radius, color);
        painter.circle_stroke(
            vertex.position,
            vertex_radius,
            egui::Stroke::new(app.config.vertex_stroke, app.config.vertex_color_outline),
        );
        if app.state.show_number {
            let vertex_show_id = if app.state.zero_indexed {
                vertex.id
            } else {
                vertex.id + 1
            }
            .to_string();
            painter.text(
                vertex.position,
                egui::Align2::CENTER_CENTER,
                vertex_show_id,
                egui::FontId::proportional(vertex_font_size),
                app.config.vertex_font_color,
            );
        }
    }
}
