use eframe::egui;
use itertools::Itertools;

#[derive(Debug)]
struct Vertex {
    id: usize,
    position: egui::Pos2,
    is_pressed: bool,
    drag_offset: egui::Vec2,
    is_selected: bool,
    z_index: u32,
}

#[derive(Debug)]
struct Edge {
    id: usize,
    from: usize,
    to: usize,
    is_selected: bool,
}

#[derive(Debug, PartialEq)]
enum EditMode {
    Select,
    AddVertex,
    AddEdge {
        from_vertex: Option<usize>,
        confirmed: bool,
    },
}

pub struct GraphEditorApp {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
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
            vertices: vec![
                Vertex {
                    id: 0,
                    position: egui::pos2(200.0, 400.0),
                    is_pressed: false,
                    drag_offset: egui::Vec2::ZERO,
                    is_selected: false,
                    z_index: 0,
                },
                Vertex {
                    id: 1,
                    position: egui::pos2(400.0, 400.0),
                    is_pressed: false,
                    drag_offset: egui::Vec2::ZERO,
                    is_selected: false,
                    z_index: 1,
                },
            ],
            edges: vec![Edge {
                id: 0,
                from: 0,
                to: 1,
                is_selected: false,
            }],
            next_z_index: 2,
            edit_mode: EditMode::Select,
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
                    self.edit_mode = EditMode::Select;
                }
                if ui.input(|i| i.key_pressed(egui::Key::V)) {
                    self.edit_mode = EditMode::AddVertex;
                }
                if ui.input(|i| i.key_pressed(egui::Key::E)) {
                    self.edit_mode = EditMode::AddEdge {
                        from_vertex: None,
                        confirmed: false,
                    };
                }

                // クリックした位置に頂点を追加する
                if self.edit_mode == EditMode::AddVertex && ui.input(|i| i.pointer.any_click()) {
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        self.vertices.push(Vertex {
                            id: self.vertices.len(),
                            position: mouse_pos,
                            is_pressed: false,
                            drag_offset: egui::Vec2::ZERO,
                            is_selected: false,
                            z_index: self.next_z_index,
                        });
                        self.next_z_index += 1;
                    }
                }

                // 辺の描画
                for edge in &self.edges {
                    if let (Some(from_vertex), Some(to_vertex)) = (
                        self.vertices.iter().find(|v| v.id == edge.from),
                        self.vertices.iter().find(|v| v.id == edge.to),
                    ) {
                        painter.line_segment(
                            [from_vertex.position, to_vertex.position],
                            egui::Stroke::new(6.0, egui::Color32::from_rgb(100, 100, 100)),
                        );
                    }
                }

                // 頂点の描画
                for vertex in self.vertices.iter_mut().sorted_by_key(|v| v.z_index) {
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
                        match self.edit_mode {
                            EditMode::Select => {
                                // クリックで選択状態をトグル
                                vertex.is_selected = !vertex.is_selected;
                                vertex.z_index = self.next_z_index;
                                self.next_z_index += 1;
                            }
                            EditMode::AddVertex => {}
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
                                        // クリックした頂点をto_vertexに設定
                                        self.edges.push(Edge {
                                            id: self.edges.len(),
                                            from: *from_vertex_inner,
                                            to: vertex.id,
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
                    .corner_radius(5)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.radio_value(
                                &mut self.edit_mode,
                                EditMode::Select,
                                egui::RichText::new("Select [S]").size(20.0),
                            );
                            ui.radio_value(
                                &mut self.edit_mode,
                                EditMode::AddVertex,
                                egui::RichText::new("Add Vertex [V]").size(20.0),
                            );
                            ui.radio_value(
                                &mut self.edit_mode,
                                EditMode::AddEdge {
                                    from_vertex: None,
                                    confirmed: false,
                                },
                                egui::RichText::new("Add Edge [E]").size(20.0),
                            );
                        });
                    });
            });

        egui::Window::new("Actions")
            .fixed_pos(egui::pos2(220.0, 10.0))
            .fixed_size(egui::vec2(150.0, 100.0))
            .collapsible(false)
            .show(ctx, |ui| {
                if ui
                    .button(egui::RichText::new("Clear All").size(20.0))
                    .clicked()
                {
                    self.vertices.clear();
                    self.edges.clear();
                    self.next_z_index = 0;
                }
            });
    }
}
