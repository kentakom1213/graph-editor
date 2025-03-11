#[derive(Debug, PartialEq, Clone)]
pub enum EditMode {
    Normal,
    AddVertex,
    AddEdge {
        from_vertex: Option<usize>,
        confirmed: bool,
    },
    Delete,
}

impl EditMode {
    pub fn default_normal() -> Self {
        Self::Normal
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

    pub fn default_delete() -> Self {
        Self::Delete
    }

    pub fn is_add_vertex(&self) -> bool {
        matches!(self, Self::AddVertex)
    }

    pub fn is_add_edge(&self) -> bool {
        matches!(self, Self::AddEdge { .. })
    }

    pub fn is_delete(&self) -> bool {
        matches!(self, Self::Delete)
    }
}
