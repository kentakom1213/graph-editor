#[derive(Debug, PartialEq, Clone)]
pub enum EditMode {
    Normal,
    AddVertex,
    AddEdge {
        from_vertex: Option<usize>,
        confirmed: bool,
    },
    DeleteEdge,
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

    pub fn default_delete_edge() -> Self {
        Self::DeleteEdge
    }

    pub fn is_add_vertex(&self) -> bool {
        matches!(self, Self::AddVertex)
    }

    pub fn is_delete_edge(&self) -> bool {
        matches!(self, Self::DeleteEdge { .. })
    }
}
