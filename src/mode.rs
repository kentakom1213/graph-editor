#[derive(Debug, PartialEq, Clone)]
pub enum EditMode {
    Select {
        selected_edges: Vec<usize>,
    },
    AddVertex,
    AddEdge {
        from_vertex: Option<usize>,
        confirmed: bool,
    },
}

impl EditMode {
    pub fn default_select() -> Self {
        Self::Select {
            selected_edges: Vec::new(),
        }
    }

    pub fn default_add_vertex() -> Self {
        Self::AddVertex
    }

    pub fn default_add_edge() -> Self {
        Self::AddEdge {
            from_vertex: None,
            confirmed: false,
        }
    }
}
