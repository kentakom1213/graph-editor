#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: usize,
    pub position: egui::Pos2,
    pub drag_offset: egui::Vec2,
    pub is_pressed: bool,
    pub is_selected: bool,
    pub z_index: u32,
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
        });
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.edges.clear();
    }

    pub fn encode(&mut self, zero_indexed: bool) -> String {
        self.edges.retain(|edge| !edge.is_deleted);

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
                from: 0,
                to: 1,
                is_pressed: false,
                is_deleted: false,
            }],
        }
    }
}
