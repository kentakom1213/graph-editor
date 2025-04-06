#[derive(Debug)]
pub struct BaseGraph {
    pub is_directed: bool,
    pub n: usize,
    pub edges: Vec<(usize, usize)>,
}

impl BaseGraph {
    /// 文字列からグラフの基本構造を生成する．
    ///
    /// ### 入力形式
    /// ```text
    /// N M
    /// u_1 v_1
    /// ...
    /// u_M v_M
    /// ```
    pub fn parse(input_text: &str, zero_indexed: bool) -> anyhow::Result<Self> {
        let mut source = input_text
            .split_ascii_whitespace()
            .map(|s| s.parse::<usize>());

        let n = source
            .next()
            .ok_or_else(|| anyhow::anyhow!("Insufficient input"))??;
        let m = source
            .next()
            .ok_or_else(|| anyhow::anyhow!("Insufficient input"))??;

        let edges = (0..m)
            .map(|_| {
                let mut from = source
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Insufficient input"))??;
                let mut to = source
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Insufficient input"))??;

                if !zero_indexed {
                    from = from
                        .checked_sub(1)
                        .ok_or_else(|| anyhow::anyhow!("Invalid edge: {} {}", from, to))?;
                    to = to
                        .checked_sub(1)
                        .ok_or_else(|| anyhow::anyhow!("Invalid edge: {} {}", from, to))?;
                }

                if from > n || to > n {
                    return Err(anyhow::anyhow!("Invalid edge: {} {}", from, to));
                }

                anyhow::Ok((from, to))
            })
            .collect::<anyhow::Result<_>>()?;

        if source.next().is_some() {
            return Err(anyhow::anyhow!("Excessive input"));
        }

        Ok(Self {
            is_directed: false,
            n,
            edges,
        })
    }

    /// 1次元形式の隣接行列を生成する
    ///
    /// - 空間計算量: O(n^2) bits
    pub fn to_adj_matrix(&self) -> Vec<usize> {
        self.edges
            .iter()
            .fold(vec![0; self.n * self.n], |mut mat, &(u, v)| {
                mat[u * self.n + v] = 1;
                mat
            })
    }

    /// 1次元形式の隣接行列からBaseGraphを生成する
    ///
    /// - 時間計算量: O(n^2) 時間
    pub fn from_adj_matrix(is_directed: bool, n: usize, adj_matrix: &[usize]) -> Self {
        let mut edges = vec![];

        for i in 0..n {
            for j in 0..n {
                if !is_directed && i > j {
                    continue;
                }
                if adj_matrix[i * n + j] == 1 {
                    edges.push((i, j));
                }
            }
        }

        Self {
            is_directed,
            n,
            edges,
        }
    }
}
