use num::{Float, One};
use std::cmp::Ordering;

use crate::{
    core::{Function, StabilityRelation, Transformation},
    dist::{AbsoluteDistance, IntDistance, SubstituteDistance, SupDistance, SymmetricDistance},
    dom::{AllDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{CheckNull, DistanceConstant, ExactIntCast},
};

pub fn make_quantile_scorer_de<TI, TO>(
    candidates: Vec<TI>,
    alpha: TO,
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TI>>,
        VectorDomain<AllDomain<TO>>,
        SymmetricDistance,
        SupDistance<AbsoluteDistance<TO>>,
    >,
>
where
    TI: 'static + CheckNull + Clone + PartialOrd,
    TO: CheckNull + DistanceConstant<IntDistance> + Float + ExactIntCast<usize>,
    IntDistance: DistanceConstant<TO>,
{
    // distances between candidate scores on neighboring datasets
    //    max d_abs(s, s')    (where s is a candidate score)
    //  = max |s - s'| = alpha * (1 - alpha)
    let abs_dist_const = alpha.max(TO::one() - alpha);
    // distance between score vectors on neighboring datasets
    //    max d_sup(s, s')    (where s is a score vector)
    //  = max_{ij} |d_abs(s_i, s_j) - d_abs(s'_i, s'_j)|
    //  = max_{ij} |(s_i - s_j) - (s'_i - s'_j)|
    //  = max_{ij} |(s_i - s'_i) - (s_j - s'_j)|
    // <= 2 * max_i |s_i - s'_i|    (scorer is not monotonic, so signs on terms can disagree)
    //  = 2 * max_i d_abs(s_i, s'_i)
    //  = 2 * alpha * (1 - alpha)   (by abs_dist_const)
    let sup_dist_const = (TO::one() + TO::one()) * abs_dist_const;
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<TI>| score(arg.clone(), &candidates, alpha.clone())),
        SymmetricDistance::default(),
        SupDistance::default(),
        StabilityRelation::new_from_constant(sup_dist_const),
    ))
}

pub fn make_sized_quantile_scorer_de<TI, TO>(
    size: usize,
    candidates: Vec<TI>,
    alpha: TO,
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<AllDomain<TI>>>,
        VectorDomain<AllDomain<TO>>,
        SubstituteDistance,
        SupDistance<AbsoluteDistance<TO>>,
    >,
>
where
    TI: 'static + CheckNull + Clone + PartialOrd,
    TO: CheckNull + DistanceConstant<IntDistance> + One + Float + ExactIntCast<usize>,
    IntDistance: DistanceConstant<TO>,
{
    if candidates.windows(2).any(|w| w[0] >= w[1]) {
        return fallible!(MakeTransformation, "candidates must be increasing");
    }
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new_all(), size),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<TI>| score(arg.clone(), &candidates, alpha.clone())),
        SubstituteDistance::default(),
        SupDistance::default(),
        StabilityRelation::new_from_constant((TO::one() + TO::one())),
    ))
}

/// Compute score of each candidates on a dataset
/// Formula is -|#(`x` <= c) - alpha * n| for each c in `candidates`.
///
/// # Arguments
/// * `x` - dataset to score against. Must be non-null
/// * `candidates` - values to be scored. Must be sorted
/// * `alpha` - parameter for quantile. {0: min, 0.5: median, 1: max, ...}
///
/// # Returns
/// Score of each candidate
fn score<TI, TO>(mut x: Vec<TI>, candidates: &Vec<TI>, alpha: TO) -> Fallible<Vec<TO>>
where
    TI: PartialOrd,
    TO: Float + ExactIntCast<usize>,
{
    // x must be sorted because counts are done via binary search
    x.sort_by(|a, b| a.partial_cmp(&b).unwrap_or(Ordering::Equal));

    // compute #(`x` <= c) for each c in candidates
    let mut num_lte = vec![0; candidates.len()];
    count_lte_recursive(
        num_lte.as_mut_slice(),
        candidates.as_slice(),
        x.as_slice(),
        0,
    );

    // now that we have num_lte, score all candidates
    let center = alpha * TO::exact_int_cast(x.len())?;
    num_lte
        .into_iter()
        .map(|v| TO::exact_int_cast(v).map(|v| -(v - center).abs()))
        .collect()
}

/// Compute number of elements less than each edge
/// Formula is #(`x` <= e) for each e in `edges`.
///
/// # Arguments
/// * `counts` - location to write the result
/// * `edges` - edges to collect counts for. Must be sorted
/// * `x` - dataset to count against
/// * `x_start_idx` - value to add to the count. Useful for recursion on subslices
fn count_lte_recursive<TI: PartialOrd>(
    counts: &mut [usize],
    edges: &[TI],
    x: &[TI],
    x_start_idx: usize,
) {
    if edges.is_empty() {
        return;
    }
    if edges.len() == 1 {
        counts[0] = x_start_idx + count_lte(x, &edges[0]);
        return;
    }
    // use binary search to find |{i; x[i] < middle edge}|
    let mid_edge_idx = (edges.len() + 1) / 2;
    let mid_edge = &edges[mid_edge_idx];
    let mid_x_idx = count_lte(x, mid_edge);
    counts[mid_edge_idx] = x_start_idx + mid_x_idx;

    count_lte_recursive(
        &mut counts[..mid_edge_idx],
        &edges[..mid_edge_idx],
        &x[..mid_x_idx],
        x_start_idx,
    );

    count_lte_recursive(
        &mut counts[mid_edge_idx + 1..],
        &edges[mid_edge_idx + 1..],
        &x[mid_x_idx..],
        x_start_idx + mid_x_idx,
    );
}

/// Find the number of elements in `x` lte `target`.
/// Formula is #(`x` <= `target`)
///
/// # Arguments
/// * `x` - dataset to count against
/// * `target` - value to compare against
fn count_lte<TI: PartialOrd>(x: &[TI], target: &TI) -> usize {
    if x.is_empty() {
        return 0;
    }
    let (mut lower, mut upper) = (0, x.len());

    while upper - lower > 1 {
        let middle = lower + (upper - lower) / 2;

        if &x[middle] <= target {
            lower = middle;
        } else {
            upper = middle;
        }
    }
    if &x[lower] <= target {
        upper
    } else {
        lower
    }
}

#[cfg(test)]
mod test_scorer {
    use super::*;

    #[test]
    fn test_count_lte() {
        let x = (5..20).collect::<Vec<i32>>();
        let edges = vec![2, 4, 7, 12, 22];
        let mut counts = vec![0; edges.len()];
        count_lte_recursive(counts.as_mut_slice(), edges.as_slice(), x.as_slice(), 0);
        println!("{:?}", counts);
        assert_eq!(counts, vec![0, 0, 3, 8, 15]);
    }

    #[test]
    fn test_count_lte_repetition() {
        let x = vec![0, 2, 2, 3, 5, 7, 7, 7];
        let edges = vec![-1, 2, 4, 7, 12, 22];
        let mut counts = vec![0; edges.len()];
        count_lte_recursive(counts.as_mut_slice(), edges.as_slice(), x.as_slice(), 0);
        println!("{:?}", counts);
        assert_eq!(counts, vec![0, 3, 4, 8, 8, 8]);
    }

    #[test]
    fn test_scorer() -> Fallible<()> {
        let edges = vec![-1, 2, 4, 7, 12, 22];

        let x = vec![0, 2, 2, 3, 5, 7, 7, 7];
        let scores = score(x, &edges, 0.5)?;
        println!("{:?}", scores);

        let x = vec![0, 2, 2, 3, 4, 7, 7, 7];
        let scores = score(x, &edges, 0.5)?;
        println!("{:?}", scores);
        Ok(())
    }
}

#[cfg(test)]
mod test_trans {
    use crate::meas::make_base_discrete_exponential;

    use super::*;

    #[test]
    fn test_int() -> Fallible<()> {
        let candidates = vec![7, 12, 14, 72, 76];
        let trans = make_quantile_scorer_de(candidates.clone(), 0.75)?;
        let trans_sized = make_sized_quantile_scorer_de(100, candidates, 0.75)?;
        let exp_mech = make_base_discrete_exponential(1., false)?;

        let quantile_meas = (trans >> exp_mech.clone())?;
        let idx = quantile_meas.invoke(&(0..100).collect())?;
        println!("idx {:?}", idx);
        println!("{:?}", quantile_meas.check(&2, &2.)?);

        let quantile_sized_meas = (trans_sized >> exp_mech)?;
        let idx = quantile_sized_meas.invoke(&(0..100).collect())?;
        println!("idx sized {:?}", idx);
        println!("{:?}", quantile_sized_meas.check(&2, &2.)?);

        Ok(())
    }
}
