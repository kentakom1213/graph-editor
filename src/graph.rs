use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: usize,
    pub position: egui::Pos2,
    pub drag_offset: egui::Vec2,
    pub is_pressed: bool,
    pub is_selected: bool,
    pub z_index: u32,
    pub is_deleted: bool,
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

    pub fn vertices_mut(&mut self) -> &mut Vec<Vertex> {
        &mut self.vertices
    }

    pub fn vertices_edges_mut(&mut self) -> (&mut Vec<Vertex>, &mut Vec<Edge>) {
        (&mut self.vertices, &mut self.edges)
    }

    pub fn add_vertex(&mut self, position: egui::Pos2, z_index: u32) {
        self.vertices.push(Vertex {
            id: self.vertices.len(),
            position,
            is_pressed: false,
            drag_offset: egui::Vec2::ZERO,
            is_selected: false,
            z_index,
            is_deleted: false,
        });
    }

    /// 無向グラフとみなしたとき，すでに辺が存在するか
    fn has_same_edge_undirected(edges: &[Edge], from: usize, to: usize) -> bool {
        edges
            .iter()
            .any(|edge| (edge.from, edge.to) == (from, to) || (edge.from, edge.to) == (to, from))
    }

    /// ユニークな無向辺を追加する．正常に追加された場合`true`を返す．
    pub fn add_unique_edge_undirected(edges: &mut Vec<Edge>, from: usize, to: usize) -> bool {
        if Self::has_same_edge_undirected(edges, from, to) {
            false
        } else {
            edges.push(Edge::new(from, to));
            true
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.edges.clear();
    }

    pub fn encode(&mut self, zero_indexed: bool) -> String {
        // 削除済み頂点を削除
        self.restore_graph();

        let mut res = format!("{} {}", self.vertices.len(), self.edges.len());

        for edges in &self.edges {
            res.push_str(&format!(
                "\n{} {}",
                if zero_indexed {
                    edges.from
                } else {
                    edges.from + 1
                },
                if zero_indexed { edges.to } else { edges.to + 1 }
            ));
        }

        res
    }
}

impl Default for Graph {
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
                    is_deleted: false,
                },
                Vertex {
                    id: 1,
                    position: egui::pos2(400.0, 400.0),
                    is_pressed: false,
                    drag_offset: egui::Vec2::ZERO,
                    is_selected: false,
                    z_index: 1,
                    is_deleted: false,
                },
            ],
            edges: vec![Edge {
                from: 0,
                to: 1,
                is_pressed: false,
                is_deleted: false,
            }],
        }
    }
}
