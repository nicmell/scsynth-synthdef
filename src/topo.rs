//! The topological sort of sclang (`SynthDef:topologicalSort`).

/// The order in which sclang writes the UGens, as indices of creation.
///
/// `antecedents[u]` holds the UGens that UGen `u` waits for: the sources of
/// its inputs, and the width-first UGens created before it.
pub(crate) fn sort(antecedents: &[Vec<usize>]) -> Vec<usize> {
    let mut waiting: Vec<usize> = antecedents.iter().map(Vec::len).collect();
    let mut descendants = vec![Vec::new(); antecedents.len()];
    for (ugen, sources) in antecedents.iter().enumerate() {
        for &source in sources {
            descendants[source].push(ugen);
        }
    }
    let mut available: Vec<usize> = (0..waiting.len())
        .rev()
        .filter(|&ugen| waiting[ugen] == 0)
        .collect();
    let mut order = Vec::with_capacity(waiting.len());
    while let Some(ugen) = available.pop() {
        for &descendant in descendants[ugen].iter().rev() {
            waiting[descendant] -= 1;
            if waiting[descendant] == 0 {
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
        let antecedents = [vec![], vec![], vec![0], vec![1], vec![2, 3]];
        assert_eq!(sort(&antecedents), [0, 2, 1, 3, 4]);
    }

    #[test]
    fn a_fan_out_runs_its_descendants_in_creation_order() {
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
        assert_eq!(sort(&[vec![], vec![], vec![1], vec![0]]), [0, 3, 1, 2]);
        // When 2 is width-first, 3 waits for it.
        assert_eq!(sort(&[vec![], vec![], vec![1], vec![0, 2]]), [0, 1, 2, 3]);
    }
}
