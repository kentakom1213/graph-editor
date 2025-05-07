#[derive(Debug)]
pub struct BaseGraph {
    pub n: usize,
    pub edges: Vec<(usize, usize)>,
}

impl BaseGraph {
    /// 文字列からグラフの基本構造を生成する．
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

        Ok(Self { n, edges })
    }
}
