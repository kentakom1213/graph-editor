use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use egui::Vec2;
use num_traits::One;

use crate::{
    config::SimulateConfig,
    math::affine::{Affine2D, ApplyAffine},
};

use super::{BaseGraph, Visualize};

const DISTANCE_EPS: f32 = 1e-5;

#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: usize,
    position: egui::Pos2,
    velocity: egui::Vec2,
    pub drag: Affine2D,
    pub is_pressed: bool,
    pub is_selected: bool,
    pub z_index: u32,
    pub is_deleted: bool,
    affine: Rc<RefCell<Affine2D>>,
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
    /// 有向グラフ / 無向グラフ
    pub is_directed: bool,
    /// 頂点集合に対するアフィン変換
    pub affine: Rc<RefCell<Affine2D>>,
    /// 頂点集合
    vertices: Vec<Vertex>,
    /// 辺集合
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

    /// グラフの入力からグラフを生成する
    pub fn apply_input(
        &mut self,
        vizualizer: &dyn Visualize,
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
        let new_vertices = vizualizer
            .resolve_vertex_position(n, &edges)
            .into_iter()
            .enumerate()
            .map(|(id, pos)| Vertex {
                id,
                position: adjust_to_window(pos),
                velocity: egui::Vec2::ZERO,
                is_pressed: false,
                drag: Affine2D::one(),
                is_selected: false,
                z_index: 0,
                is_deleted: false,
                affine: self.affine.clone(),
            });

        self.vertices.extend(new_vertices);

        let new_edges = edges.into_iter().map(|(from, to)| Edge::new(from, to));

        self.edges.extend(new_edges);

        Ok(())
    }

    /// 1ステップ分シミュレーションを行う
    /// アルゴリズム: <project://memo/graph_visualization.md >
    ///
    /// ### Args
    /// - `c`: クーロン定数（頂点同士の反発力）
    /// - `k`: ばね定数
    /// - `l`: ばねの自然長
    /// - `h`: 力の減衰率
    /// - `m`: 頂点の重さ
    /// - `dt`: 微小時間
    pub fn simulate_step(
        &mut self,
        &SimulateConfig {
            c,
            k,
            l,
            h,
            m,
            max_v,
            dt,
        }: &SimulateConfig,
    ) {
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
            let mut next_velocity = (v.velocity + fv * dt / m) * h;

            if next_velocity.length() > max_v {
                next_velocity = next_velocity.normalized() * max_v;
            }

            // 位置を更新
            let next_position = v.position + v.velocity * dt;

            self.vertices[i].velocity = next_velocity;
            self.vertices[i].position = next_position;
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
                    affine: affine.clone(),
                },
            ],
            edges: vec![Edge {
                from: 0,
                to: 1,
                is_pressed: false,
                is_deleted: false,
            }],
            affine,
        }
    }
}
