//! The topological sort of sclang (`SynthDef:topologicalSort`).

use std::collections::BTreeSet;

/// The order in which sclang writes the UGens, as indices of creation.
///
/// `antecedents[u]` holds the UGens that UGen `u` waits for: the sources of
/// its inputs, and the width-first UGens created before it. Each holds a
/// lower index than `u`.
pub(crate) fn sort(antecedents: &[Vec<usize>]) -> Vec<usize> {
    let mut waiting: Vec<BTreeSet<usize>> = antecedents
        .iter()
        .map(|ugens| ugens.iter().copied().collect())
        .collect();
    let mut descendants = vec![Vec::new(); waiting.len()];
    for (ugen, sources) in waiting.iter().enumerate() {
        for &source in sources {
            descendants[source].push(ugen);
        }
    }
    let mut available: Vec<usize> = (0..waiting.len())
        .rev()
        .filter(|&ugen| waiting[ugen].is_empty())
        .collect();
    let mut order = Vec::with_capacity(waiting.len());
    while let Some(ugen) = available.pop() {
        for &descendant in descendants[ugen].iter().rev() {
            waiting[descendant].remove(&ugen);
            if waiting[descendant].is_empty() {
                available.push(descendant);
            }
        }
        order.push(ugen);
    }
    order
}

#[cfg(test)]
mod tests {
    use super::sort;

    #[test]
    fn independent_ugens_keep_their_order() {
        assert_eq!(sort(&[vec![], vec![], vec![]]), [0, 1, 2]);
    }

    #[test]
    fn a_chain_runs_before_the_next_root() {
        // 0 and 1 are roots; 2 reads 0, 3 reads 1, 4 reads 2 and 3.
        let antecedents = [vec![], vec![], vec![0], vec![1], vec![2, 3]];
        assert_eq!(sort(&antecedents), [0, 2, 1, 3, 4]);
    }

    #[test]
    fn a_fan_out_runs_its_descendants_in_creation_order() {
        // 1 and 2 read 0; 3 reads both.
        let antecedents = [vec![], vec![0], vec![0], vec![1, 2]];
        assert_eq!(sort(&antecedents), [0, 1, 2, 3]);
    }

    #[test]
    fn a_source_read_twice_counts_once() {
        let antecedents = [vec![], vec![0, 0], vec![1]];
        assert_eq!(sort(&antecedents), [0, 1, 2]);
    }

    #[test]
    fn a_ugen_waits_for_every_width_first_ugen_before_it() {
        // 0 and 1 are roots, 2 reads 1, and 3 reads 0.
        assert_eq!(sort(&[vec![], vec![], vec![1], vec![0]]), [0, 3, 1, 2]);
        // When 2 is width-first, 3 waits for it.
        assert_eq!(sort(&[vec![], vec![], vec![1], vec![0, 2]]), [0, 1, 2, 3]);
    }
}
