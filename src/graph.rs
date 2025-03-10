#![warn(dead_code)]

#[derive(Debug)]
pub struct Vertex {
    pub id: usize,
    pub position: egui::Pos2,
    pub is_pressed: bool,
    pub drag_offset: egui::Vec2,
    pub is_selected: bool,
    pub z_index: u32,
}

#[derive(Debug)]
pub struct Edge {
    pub id: usize,
    pub from: usize,
    pub to: usize,
    pub is_selected: bool,
}

#[derive(Debug)]
pub struct Graph {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
}

impl Graph {
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn vertices_mut(&mut self) -> &mut Vec<Vertex> {
        &mut self.vertices
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn edges_mut(&mut self) -> &mut Vec<Edge> {
        &mut self.edges
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
        });
    }

    pub fn add_edge(&mut self, from_vertex_id: usize, to_vertex_id: usize) {
        self.edges.push(Edge {
            id: self.edges.len(),
            from: from_vertex_id,
            to: to_vertex_id,
            is_selected: false,
        });
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.edges.clear();
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
        }
    }
}
