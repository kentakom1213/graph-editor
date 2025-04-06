use graph6_rs::{GraphConversion, WriteGraph};

use super::BaseGraph;

/// graph6 形式にエンコードする
pub trait ToGraph6 {
    /// graph6 形式にエンコードする
    fn to_graph6(&self) -> String;
}

impl ToGraph6 for BaseGraph {
    fn to_graph6(&self) -> String {
        if self.is_directed {
            let dgraph = graph6_rs::DiGraph::from_adj(&self.to_adj_matrix()).unwrap();

            dgraph.write_graph()
        } else {
            let udgraph = graph6_rs::Graph::from_adj(&self.to_adj_matrix()).unwrap();

            udgraph.write_graph()
        }
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
        if g6_str.starts_with('&') {
            let dgraph =
                graph6_rs::DiGraph::from_d6(g6_str).map_err(|e| anyhow::anyhow!("{e:?}"))?;

            Ok(BaseGraph::from_adj_matrix(
                true,
                dgraph.size(),
                dgraph.bit_vec(),
            ))
        } else {
            let udgraph =
                graph6_rs::Graph::from_g6(g6_str).map_err(|e| anyhow::anyhow!("{e:?}"))?;

            Ok(BaseGraph::from_adj_matrix(
                false,
                udgraph.size(),
                udgraph.bit_vec(),
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::graph::{graph6::ToGraph6, BaseGraph};

    use super::TryFromGraph6;

    #[test]
    fn test_directed_to_g6() {
        let g1 = BaseGraph {
            is_directed: true,
            n: 2,
            edges: vec![(1, 0)],
        };
        assert_eq!(g1.to_graph6(), "&AG".to_string());

        let g2 = BaseGraph {
            is_directed: true,
            n: 12,
            edges: vec![
                (0, 2),
                (11, 0),
                (1, 3),
                (7, 3),
                (7, 4),
                (5, 6),
                (4, 8),
                (8, 10),
                (6, 3),
                (1, 11),
                (1, 4),
                (5, 4),
            ],
        };
        assert_eq!(g2.to_graph6(), "&KG?E@?????GA_C?E??A????_?".to_string());
    }

    #[test]
    fn test_undirected_to_g6() {
        let g1 = BaseGraph {
            is_directed: false,
            n: 2,
            edges: vec![(1, 0)],
        };
        assert_eq!(g1.to_graph6(), "A_".to_string());

        let g2 = BaseGraph {
            is_directed: false,
            n: 12,
            edges: vec![
                (0, 2),
                (11, 0),
                (1, 3),
                (7, 3),
                (7, 4),
                (5, 6),
                (4, 8),
                (8, 10),
                (6, 3),
                (1, 11),
                (1, 4),
                (5, 4),
            ],
        };
        assert_eq!(g2.to_graph6(), "KQOGgoG??@W?".to_string());
    }

    #[test]
    fn test_g6_to_directed() {
        let g1 = BaseGraph::try_from_graph6("&AG").unwrap();
        assert_eq!(g1.is_directed, true);
        assert_eq!(g1.n, 2);
        assert_eq!(g1.edges, vec![(1, 0)]);

        let g2 = BaseGraph::try_from_graph6("&KG?E@?????GA_C?E??A????_?").unwrap();
        assert_eq!(g2.is_directed, true);
        assert_eq!(g2.n, 12);
        assert_eq!(
            g2.edges,
            vec![
                (0, 2),
                (1, 3),
                (1, 4),
                (1, 11),
                (4, 8),
                (5, 4),
                (5, 6),
                (6, 3),
                (7, 3),
                (7, 4),
                (8, 10),
                (11, 0),
            ]
        );
    }

    #[test]
    fn test_g6_to_undirected() {
        let g1 = BaseGraph::try_from_graph6("A_").unwrap();
        assert_eq!(g1.is_directed, false);
        assert_eq!(g1.n, 2);
        assert_eq!(g1.edges, vec![(0, 1)]);

        let g2 = BaseGraph::try_from_graph6("KQOGgoG??@W?").unwrap();
        assert_eq!(g2.is_directed, false);
        assert_eq!(g2.n, 12);
        assert_eq!(
            g2.edges,
            vec![
                (0, 2),
                (0, 11),
                (1, 3),
                (1, 4),
                (1, 11),
                (3, 6),
                (3, 7),
                (4, 5),
                (4, 7),
                (4, 8),
                (5, 6),
                (8, 10),
            ]
        );
    }
}
