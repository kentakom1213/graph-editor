use super::BaseGraph;

/// graph6 形式にエンコードする
pub trait ToGraph6 {
    /// graph6 形式にエンコードする
    fn to_graph6(&self) -> String;
}

impl ToGraph6 for BaseGraph {
    fn to_graph6(&self) -> String {
        todo!()
    }
}

/// graph6 をデコードする
pub trait TryFromGraph6 {
    /// graph6 をデコードする
    fn try_from_graph6(graph6: &str) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl TryFromGraph6 for BaseGraph {
    fn try_from_graph6(g6_str: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
