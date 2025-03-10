use eframe::egui;
use itertools::Itertools;

use crate::graph::{Edge, Graph};
use crate::mode::EditMode;

pub struct GraphEditorApp {
    graph: Graph,
    next_z_index: u32,
    edit_mode: EditMode,
}

impl GraphEditorApp {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for GraphEditorApp {
    fn default() -> Self {
        Self {
            graph: Graph::default(),
            next_z_index: 2,
            edit_mode: EditMode::default_select(),
        }
    }
}

impl eframe::App for GraphEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(230, 230, 230)))
            .show(ctx, |ui| {
                let painter = ui.painter();
                let radius = 50.0;

                // モード切替を行う
                if ui.input(|i| i.key_pressed(egui::Key::S)) {
                    self.edit_mode = EditMode::default_select();
                }
                if ui.input(|i| i.key_pressed(egui::Key::V)) {
                    self.edit_mode = EditMode::default_add_vertex();
                }
                if ui.input(|i| i.key_pressed(egui::Key::E)) {
                    self.edit_mode = EditMode::default_add_edge();
                }

                if let EditMode::AddEdge {
                    from_vertex: ref mut from_vertex @ Some(from_vertex_id),
                    ..
                } = self.edit_mode
                {
                    // Escapeキーで選択頂点を解除
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        if let Some(from_vertex) = self
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
                if self.edit_mode == EditMode::AddVertex && ui.input(|i| i.pointer.any_click()) {
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        self.graph.add_vertex(mouse_pos, self.next_z_index);
                        self.next_z_index += 1;
                    }
                }

                // 辺の描画
                for edge in self.graph.edges() {
                    if let (Some(from_vertex), Some(to_vertex)) = (
                        self.graph.vertices().iter().find(|v| v.id == edge.from),
                        self.graph.vertices().iter().find(|v| v.id == edge.to),
                    ) {
                        painter.line_segment(
                            [from_vertex.position, to_vertex.position],
                            egui::Stroke::new(6.0, egui::Color32::from_rgb(100, 100, 100)),
                        );
                    }
                }

                // 頂点の描画
                let (vertices_mut, edges_mut) = self.graph.vertices_edges_mut();

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
                        vertex.z_index = self.next_z_index;
                        self.next_z_index += 1;
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
                    if matches!(self.edit_mode, EditMode::AddEdge { .. }) {
                        vertex.is_pressed = response.hovered();
                    }

                    // 選択時
                    if response.clicked() && !response.dragged() {
                        // 最前面に配置
                        vertex.z_index = self.next_z_index;
                        self.next_z_index += 1;

                        if let EditMode::AddEdge {
                            ref mut from_vertex,
                            ref mut confirmed,
                        } = self.edit_mode
                        {
                            if let Some(from_vertex_inner) = from_vertex {
                                if *from_vertex_inner == vertex.id {
                                    // 自分だった場合，選択を解除
                                    vertex.is_selected = false;
                                    *from_vertex = None;
                                } else {
                                    // クリックした頂点をto_vertexに設定
                                    edges_mut.push(Edge {
                                        id: edges_mut.len(),
                                        from: *from_vertex_inner,
                                        to: vertex.id,
                                        is_pressed: false,
                                        is_selected: false,
                                    });
                                    *confirmed = true;
                                }
                            } else {
                                vertex.is_selected = true;
                                vertex.z_index = self.next_z_index;
                                self.next_z_index += 1;
                                // クリックした頂点をfrom_vertexに設定
                                *from_vertex = Some(vertex.id);
                            }
                        }
                    }

                    match self.edit_mode {
                        // 始点のみ選択状態の場合，辺を描画
                        EditMode::AddEdge {
                            from_vertex: Some(from_vertex_inner),
                            confirmed: false,
                        } => {
                            if vertex.id == from_vertex_inner {
                                if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                                    painter.line_segment(
                                        [vertex.position, mouse_pos],
                                        egui::Stroke::new(
                                            6.0,
                                            egui::Color32::from_rgb(100, 100, 100),
                                        ),
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

                    painter.circle_filled(vertex.position, radius, color);
                    painter.circle_stroke(
                        vertex.position,
                        radius,
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(150, 150, 150)),
                    );
                    painter.text(
                        vertex.position,
                        egui::Align2::CENTER_CENTER,
                        vertex.id.to_string(),
                        egui::FontId::proportional(50.0),
                        egui::Color32::BLACK,
                    );
                }
            });

        egui::Window::new("Edit Mode")
            .fixed_pos(egui::pos2(10.0, 10.0))
            .fixed_size(egui::vec2(200.0, 150.0))
            .collapsible(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.radio_value(
                                &mut self.edit_mode,
                                EditMode::default_select(),
                                egui::RichText::new("Select [S]").size(20.0),
                            );
                            ui.radio_value(
                                &mut self.edit_mode,
                                EditMode::default_add_vertex(),
                                egui::RichText::new("Add Vertex [V]").size(20.0),
                            );
                            ui.radio_value(
                                &mut self.edit_mode,
                                EditMode::default_add_edge(),
                                egui::RichText::new("Add Edge [E]").size(20.0),
                            );
                        });
                    });
            });

        egui::Window::new("Action")
            .fixed_pos(egui::pos2(200.0, 10.0))
            .fixed_size(egui::vec2(150.0, 100.0))
            .collapsible(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        if ui
                            .button(egui::RichText::new("Clear All").size(20.0))
                            .clicked()
                        {
                            self.graph.clear();
                            self.next_z_index = 0;
                        }
                    });
            });
    }
}
