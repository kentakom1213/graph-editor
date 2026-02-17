use std::collections::HashSet;

use num_traits::One;

use crate::{components::Colors, graph::Graph, math::affine::Affine2D};

#[derive(Debug, Clone)]
pub struct VertexViewState {
    pub is_pressed: bool,
    pub is_selected: bool,
    pub z_index: u32,
    pub drag: Affine2D,
    pub color: Colors,
}

impl Default for VertexViewState {
    fn default() -> Self {
        Self {
            is_pressed: false,
            is_selected: false,
            z_index: 0,
            drag: Affine2D::one(),
            color: Colors::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EdgeViewState {
    pub is_pressed: bool,
    pub color: Colors,
}

#[derive(Debug, Clone)]
pub struct GraphViewState {
    pub vertices: Vec<VertexViewState>,
    pub edges: Vec<EdgeViewState>,
}

impl GraphViewState {
    pub fn new_for_graph(graph: &Graph) -> Self {
        let vertices = graph
            .vertices
            .iter()
            .enumerate()
            .map(|(idx, _)| VertexViewState {
                z_index: idx as u32,
                ..VertexViewState::default()
            })
            .collect();

        let edges = graph
            .edges
            .iter()
            .map(|_| EdgeViewState::default())
            .collect();

        Self { vertices, edges }
    }

    pub fn reset_for_graph(&mut self, graph: &Graph) {
        *self = Self::new_for_graph(graph);
    }

    pub fn add_vertex(&mut self, z_index: u32) {
        self.vertices.push(VertexViewState {
            z_index,
            ..VertexViewState::default()
        });
    }

    pub fn add_edge(&mut self) {
        self.edges.push(EdgeViewState::default());
    }

    pub fn apply_deletions(&mut self, graph: &Graph) {
        let deleted_vertices: HashSet<usize> = graph
            .vertices
            .iter()
            .filter(|v| v.is_deleted)
            .map(|v| v.id)
            .collect();

        let mut next_vertices = Vec::with_capacity(self.vertices.len());
        for (idx, vertex) in self.vertices.iter().enumerate() {
            if let Some(model_vertex) = graph.vertices.get(idx) {
                if !model_vertex.is_deleted {
                    next_vertices.push(vertex.clone());
                }
            }
        }
        self.vertices = next_vertices;

        let mut next_edges = Vec::with_capacity(self.edges.len());
        for (idx, view) in self.edges.iter().enumerate() {
            if let Some(edge) = graph.edges.get(idx) {
                if edge.is_deleted
                    || deleted_vertices.contains(&edge.from)
                    || deleted_vertices.contains(&edge.to)
                {
                    continue;
                }
                next_edges.push(view.clone());
            }
        }
        self.edges = next_edges;
    }

    pub fn reset_colors(&mut self) {
        for vertex in &mut self.vertices {
            vertex.color = Colors::default();
        }
        for edge in &mut self.edges {
            edge.color = Colors::default();
        }
    }

    pub fn snapshot(&self, graph: &Graph) -> GraphSnapshot {
        let vertices: Vec<_> = graph
            .vertices
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.is_deleted)
            .filter_map(|(idx, v)| {
                let view = self.vertices.get(idx)?;
                Some(VertexSnapshot {
                    id: v.id,
                    position: v.get_position(),
                    is_pressed: view.is_pressed,
                    is_selected: view.is_selected,
                    z_index: view.z_index,
                    color: view.color,
                })
            })
            .collect();

        let vertex_ids: HashSet<_> = vertices.iter().map(|v| v.id).collect();

        let edges: Vec<_> = graph
            .edges
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.is_deleted)
            .filter_map(|(idx, e)| {
                let view = self.edges.get(idx)?;
                if !vertex_ids.contains(&e.from) || !vertex_ids.contains(&e.to) {
                    return None;
                }
                Some(EdgeSnapshot {
                    from: e.from,
                    to: e.to,
                    is_pressed: view.is_pressed,
                    color: view.color,
                })
            })
            .collect();

        GraphSnapshot {
            is_directed: graph.is_directed,
            vertices,
            edges,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VertexSnapshot {
    pub id: usize,
    pub position: egui::Pos2,
    pub is_pressed: bool,
    pub is_selected: bool,
    pub z_index: u32,
    pub color: Colors,
}

#[derive(Debug, Clone)]
pub struct EdgeSnapshot {
    pub from: usize,
    pub to: usize,
    pub is_pressed: bool,
    pub color: Colors,
}

#[derive(Debug, Clone)]
pub struct GraphSnapshot {
    pub is_directed: bool,
    pub vertices: Vec<VertexSnapshot>,
    pub edges: Vec<EdgeSnapshot>,
}
