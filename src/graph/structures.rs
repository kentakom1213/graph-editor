use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use egui::Vec2;
use num_traits::One;

use crate::{
    components::Colors,
    math::affine::{Affine2D, ApplyAffine},
};

use super::{visualize_methods, BaseGraph, Visualizer};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: usize,
    pub position: egui::Pos2,
    pub velocity: egui::Vec2,
    pub drag: Affine2D,
    pub is_pressed: bool,
    pub is_selected: bool,
    pub z_index: u32,
    pub is_deleted: bool,
    pub color: Colors,
    pub affine: Rc<RefCell<Affine2D>>,
}

impl Vertex {
    pub fn get_position(&self) -> egui::Pos2 {
        self.position.applied(&self.affine.borrow())
    }

    pub fn affine(&self) -> Ref<'_, Affine2D> {
        self.affine.borrow()
    }

    pub fn update_position(&mut self, new_position: egui::Pos2) {
        if let Some(inv) = self.affine.borrow().inverse() {
            self.position = new_position.applied(&inv);
        }
    }

    pub fn solve_drag_offset(&mut self) {
        self.position.apply(&self.drag);
        self.drag = Affine2D::one();
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub is_pressed: bool,
    pub is_deleted: bool,
    pub color: Colors,
}

impl Edge {
    pub fn new(from: usize, to: usize) -> Self {
        Self {
            from,
            to,
            is_pressed: false,
            is_deleted: false,
            color: Colors::default(),
        }
    }
}

#[derive(Debug)]
pub struct Graph {
    /// 有向グラフ / 無向グラフ
    pub is_directed: bool,
    /// 頂点集合に対するアフィン変換
    pub affine: Rc<RefCell<Affine2D>>,
    /// 頂点集合
    pub vertices: Vec<Vertex>,
    /// 辺集合
    pub edges: Vec<Edge>,
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
        let position = position - self.affine.borrow().translation();

        self.vertices.push(Vertex {
            id: self.vertices.len(),
            position,
            velocity: Vec2::ZERO,
            is_pressed: false,
            drag: Affine2D::one(),
            is_selected: false,
            z_index,
            is_deleted: false,
            color: Colors::default(),
            affine: self.affine.clone(),
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

    pub fn list_unique_edges(&self) -> Vec<(usize, usize)> {
        let mut seen = HashSet::new();

        self.edges
            .iter()
            .filter_map(|e| {
                if !self.is_directed && seen.contains(&(e.to, e.from)) {
                    None
                } else {
                    seen.insert((e.from, e.to));
                    Some((e.from, e.to))
                }
            })
            .collect()
    }

    pub fn encode(&mut self, zero_indexed: bool) -> String {
        // 削除済み頂点を削除
        self.restore_graph();

        let unique_edges = self.list_unique_edges();
        let mut res = format!("{} {}", self.vertices.len(), unique_edges.len());

        for (from, to) in unique_edges {
            res.push_str(&format!(
                "\n{} {}",
                if zero_indexed { from } else { from + 1 },
                if zero_indexed { to } else { to + 1 }
            ));
        }

        res
    }

    /// グラフの補グラフを求める（無向グラフの場合のみ）
    pub fn calc_complement(&self) -> BaseGraph {
        debug_assert!(!self.is_directed);

        let n = self.vertices.len();

        // 辺の存在性を反転
        let mut edge_existence = vec![vec![true; n]; n];

        for edge in &self.edges {
            let u = edge.from;
            let v = edge.to;
            edge_existence[u][v] = false;
            edge_existence[v][u] = false;
        }

        BaseGraph {
            n,
            edges: (0..n)
                .flat_map(move |i| (i + 1..n).map(move |j| (i, j)))
                .filter(|&(u, v)| edge_existence[u][v])
                .collect(),
        }
    }

    /// 逆辺を張ったグラフを求める（有向グラフの場合のみ）
    pub fn calc_reverted(&self) -> BaseGraph {
        debug_assert!(self.is_directed);

        BaseGraph {
            n: self.vertices.len(),
            edges: self.edges.iter().map(|e| (e.to, e.from)).collect(),
        }
    }

    /// グラフの入力からグラフを生成する
    pub fn rebuild_from_basegraph(
        &mut self,
        visualizer: &dyn Visualizer,
        density_threshold: f32,
        BaseGraph { n, edges }: BaseGraph,
        window_size: egui::Vec2,
    ) -> anyhow::Result<()> {
        // グラフの初期化
        self.clear();
        *self.affine.borrow_mut() = Affine2D::one();

        // 頂点座標を適切な位置に（上下左右 10% の余白をもたせる）
        let adjust_to_window = |pos: egui::Vec2| -> egui::Pos2 {
            (pos * window_size * 0.8 + window_size * 0.1).to_pos2()
        };

        // グラフの構築
        let density = if n == 0 {
            0.0
        } else {
            edges.len() as f32 / (n as f32 * n as f32)
        };

        let positions = if density > density_threshold {
            // 高密度グラフでは最適化を避けてランダム配置にする
            visualize_methods::Naive.resolve_vertex_position(n, &edges)
        } else {
            visualizer.resolve_vertex_position(n, &edges)
        };

        let new_vertices = positions.into_iter().enumerate().map(|(id, pos)| Vertex {
            id,
            position: adjust_to_window(pos),
            velocity: egui::Vec2::ZERO,
            is_pressed: false,
            drag: Affine2D::one(),
            is_selected: false,
            z_index: 0,
            is_deleted: false,
            color: Colors::default(),
            affine: self.affine.clone(),
        });

        self.vertices.extend(new_vertices);

        let new_edges = edges.into_iter().map(|(from, to)| Edge::new(from, to));

        self.edges.extend(new_edges);

        Ok(())
    }

    pub fn reset_colors(&mut self) {
        for vertex in &mut self.vertices {
            vertex.color = Colors::default();
        }
        for edge in &mut self.edges {
            edge.color = Colors::default();
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        let affine = Rc::new(RefCell::new(Affine2D::one()));

        Self {
            is_directed: false,
            vertices: vec![
                Vertex {
                    id: 0,
                    position: egui::pos2(400.0, 400.0),
                    velocity: egui::Vec2::ZERO,
                    is_pressed: false,
                    drag: Affine2D::one(),
                    is_selected: false,
                    z_index: 0,
                    is_deleted: false,
                    color: Colors::default(),
                    affine: affine.clone(),
                },
                Vertex {
                    id: 1,
                    position: egui::pos2(600.0, 400.0),
                    velocity: egui::Vec2::ZERO,
                    is_pressed: false,
                    drag: Affine2D::one(),
                    is_selected: false,
                    z_index: 1,
                    is_deleted: false,
                    color: Colors::default(),
                    affine: affine.clone(),
                },
            ],
            edges: vec![Edge {
                from: 0,
                to: 1,
                is_pressed: false,
                is_deleted: false,
                color: Colors::default(),
            }],
            affine,
        }
    }
}
