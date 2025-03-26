use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use egui::Vec2;

const DISTANCE_EPS: f32 = 1e-5;

#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: usize,
    position: egui::Pos2,
    velocity: egui::Vec2,
    pub drag_offset: egui::Vec2,
    pub is_pressed: bool,
    pub is_selected: bool,
    pub z_index: u32,
    pub is_deleted: bool,
    offset: Rc<RefCell<egui::Vec2>>,
}

impl Vertex {
    pub fn get_position(&self) -> egui::Pos2 {
        self.position + *self.offset.borrow()
    }

    pub fn update_position(&mut self, new_position: egui::Pos2) {
        self.position = new_position - *self.offset.borrow();
    }

    pub fn solve_drag_offset(&mut self) {
        self.position += self.drag_offset;
        self.drag_offset = egui::Vec2::ZERO;
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub is_pressed: bool,
    pub is_deleted: bool,
}

impl Edge {
    pub fn new(from: usize, to: usize) -> Self {
        Self {
            from,
            to,
            is_pressed: false,
            is_deleted: false,
        }
    }
}

#[derive(Debug)]
pub struct Graph {
    pub is_directed: bool,
    pub offset: Rc<RefCell<egui::Vec2>>,
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
}

impl Graph {
    /// 削除済みフラグが立っている頂点，辺を削除する
    pub fn restore_graph(&mut self) {
        // 削除済みフラグが立っている頂点を削除
        self.vertices.retain(|vertex| !vertex.is_deleted);

        // 頂点番号を振り直す
        let mut new_vertex_id = HashMap::new();

        for (i, vertex) in self.vertices.iter_mut().enumerate() {
            new_vertex_id.insert(vertex.id, i);
            vertex.id = i;
        }

        // 辺の頂点番号を振り直す
        for edge in &mut self.edges {
            if let Some((new_from, new_to)) = new_vertex_id
                .get(&edge.from)
                .zip(new_vertex_id.get(&edge.to))
            {
                edge.from = *new_from;
                edge.to = *new_to;
            } else {
                // 辺を削除
                edge.is_deleted = true;
            }
        }

        // 削除済みフラグが立っている辺を削除
        self.edges.retain(|edge| !edge.is_deleted);
    }

    pub fn edges(&self) -> &Vec<Edge> {
        &self.edges
    }

    pub fn vertices_mut(&mut self) -> &mut Vec<Vertex> {
        &mut self.vertices
    }

    pub fn edges_mut(&mut self) -> &mut Vec<Edge> {
        &mut self.edges
    }

    pub fn vertices_edges_mut(&mut self) -> (&mut Vec<Vertex>, &mut Vec<Edge>) {
        (&mut self.vertices, &mut self.edges)
    }

    pub fn add_vertex(&mut self, position: egui::Pos2, z_index: u32) {
        let position = position - *self.offset.borrow();
        let offset = self.offset.clone();

        self.vertices.push(Vertex {
            id: self.vertices.len(),
            position,
            velocity: Vec2::ZERO,
            is_pressed: false,
            drag_offset: egui::Vec2::ZERO,
            is_selected: false,
            z_index,
            is_deleted: false,
            offset,
        });
    }

    /// 始点と終点が同じ辺が存在するか
    fn has_same_edge(is_directed: bool, edges: &[Edge], from: usize, to: usize) -> bool {
        edges.iter().any(|edge| {
            (edge.from, edge.to) == (from, to) || !is_directed && (edge.from, edge.to) == (to, from)
        })
    }

    /// ユニークな辺を追加する．正常に追加された場合`true`を返す．
    pub fn add_unique_edge(
        is_directed: bool,
        edges: &mut Vec<Edge>,
        from: usize,
        to: usize,
    ) -> bool {
        if Self::has_same_edge(is_directed, edges, from, to) {
            false
        } else {
            edges.push(Edge::new(from, to));
            true
        }
    }

    /// 隣接頂点のidを列挙する
    pub fn neighbor_vertices(&self, id: usize) -> impl Iterator<Item = &Vertex> {
        self.edges
            .iter()
            .filter_map(move |v| {
                if v.from == id {
                    Some(v.to)
                } else if v.to == id {
                    Some(v.from)
                } else {
                    None
                }
            })
            .filter_map(|id| self.vertices.iter().find(|v| v.id == id))
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.edges.clear();
    }

    pub fn encode(&mut self, zero_indexed: bool) -> String {
        // 削除済み頂点を削除
        self.restore_graph();

        let mut seen_edges = HashSet::new();
        let mut res = format!("{} {}", self.vertices.len(), self.edges.len());

        for edges in &self.edges {
            // すでにみていた場合飛ばす
            if !self.is_directed && seen_edges.contains(&(edges.to, edges.from)) {
                continue;
            }
            res.push_str(&format!(
                "\n{} {}",
                if zero_indexed {
                    edges.from
                } else {
                    edges.from + 1
                },
                if zero_indexed { edges.to } else { edges.to + 1 }
            ));
            seen_edges.insert((edges.from, edges.to));
        }

        res
    }

    /// 1ステップ分シミュレーションを行う
    /// アルゴリズム: <project://memo/graph_visualization.md >
    pub fn simulate_step(&mut self, c: f32, k: f32, l: f32, h: f32, m: f32, dt: f32) {
        // ドラッグ差分を解消
        self.vertices_mut()
            .iter_mut()
            .for_each(|v| v.solve_drag_offset());

        let n = self.vertices.len();

        for i in 0..n {
            let v = self.vertices[i].clone();

            // vからxへ向かう単位ベクトル
            let r = |x: egui::Pos2| -> egui::Vec2 { (x - v.position).normalized() };

            // 頂点vに働く力
            let fv = self
                .vertices
                .iter()
                .filter(|w| w.position.distance(v.position) > DISTANCE_EPS)
                // 頂点間の斥力
                .map(|w| -r(w.position) * c / v.position.distance_sq(w.position))
                // 辺による引力
                .chain(
                    self.neighbor_vertices(v.id)
                        .map(|w| r(w.position) * (v.position.distance(w.position) - l) * k),
                )
                .fold(egui::Vec2::ZERO, |acc, f| acc + f);

            // 速度を更新
            let next_velocity = (v.velocity + fv * dt / m) * h;

            // 位置を更新
            let next_position = v.position + v.velocity * dt;

            self.vertices[i].velocity = next_velocity;
            self.vertices[i].position = next_position;
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        let offset = Rc::new(RefCell::new(egui::Vec2::ZERO));

        Self {
            is_directed: false,
            vertices: vec![
                Vertex {
                    id: 0,
                    position: egui::pos2(400.0, 400.0),
                    velocity: egui::Vec2::ZERO,
                    is_pressed: false,
                    drag_offset: egui::Vec2::ZERO,
                    is_selected: false,
                    z_index: 0,
                    is_deleted: false,
                    offset: offset.clone(),
                },
                Vertex {
                    id: 1,
                    position: egui::pos2(600.0, 400.0),
                    velocity: egui::Vec2::ZERO,
                    is_pressed: false,
                    drag_offset: egui::Vec2::ZERO,
                    is_selected: false,
                    z_index: 1,
                    is_deleted: false,
                    offset: offset.clone(),
                },
            ],
            edges: vec![Edge {
                from: 0,
                to: 1,
                is_pressed: false,
                is_deleted: false,
            }],
            offset,
        }
    }
}
