use num::{Float, One};
use std::cmp::Ordering;

use crate::{
    core::{Function, StabilityRelation, Transformation},
    dist::{AbsoluteDistance, IntDistance, SupDistance, SymmetricDistance},
    dom::{AllDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{CheckNull, DistanceConstant, ExactIntCast, InfSub},
};

/// Makes a [Transformation] that scores how similar each candidate is to the given `alpha`-quantile on the input dataset.
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
    TO: CheckNull + DistanceConstant<IntDistance> + Float + ExactIntCast<usize> + InfSub,
    IntDistance: DistanceConstant<TO>,
{
    // distances between candidate scores on neighboring datasets
    //    max d_abs(s, s')    (where s is a candidate score)
    //  = max |s - s'| 
    //  = max | -|#(x <= c) - alpha * n| - -|#(x' <= c) - alpha * n||
    //  assume x' is equal to x, but with some x_i <= c removed
    //      = max | -|#(x <= c) - alpha * n| - -|#(x' <= c) - alpha * (n - 1)||
    //      = max | -|#(x <= c) - alpha * n| - -|#(x <= c) - 1 - alpha * n + alpha||  (by the assumption)
    //      = max | -|#(x <= c) - alpha * n| - -|#(x <= c) - alpha * n + (alpha - 1)||
    //     <= max | -|#(x <= c) - alpha * n| - -(|#(x <= c) - alpha * n| + |alpha - 1|)|
    //      = max | a - a - |alpha - 1|| (where a = -|#(x <= c) - alpha * n|)
    //      = 1 - alpha (since alpha <= 1)
    //  assume x' is equal to x, but with some x_i > c removed
    //      = max | -|#(x <= c) - alpha * n| - -|#(x' <= c) - alpha * (n - 1)||
    //      = max | -|#(x <= c) - alpha * n| - -|#(x <= c) - alpha * n + alpha|| (by the assumption)
    //      = max | -|#(x <= c) - alpha * n| - -|#(x <= c) - alpha * n + alpha||
    //     <= max | -|#(x <= c) - alpha * n| - -(|#(x <= c) - alpha * n| + |alpha|)|
    //      = max | a - a - |alpha|| (where a = -|#(x <= c) - alpha * n|)
    //      = alpha (since alpha >= 0)
    //  assume x' is equal to x, but with some x'_i <= c added
    //      = max | -|#(x <= c) - alpha * n| - -|#(x' <= c) - alpha * (n + 1)||
    //      = max | -|#(x <= c) - alpha * n| - -|#(x <= c) + 1 - alpha * (n + 1)|| (by the assumption)
    //      = max | -|#(x <= c) - alpha * n| - -|#(x <= c) - alpha * n + (1 - alpha)||
    //     <= max | -|#(x <= c) - alpha * n| - -(|#(x <= c) - alpha * n| + |1 - alpha|)|
    //      = max | a - a - |1 - alpha|| (where a = -|#(x <= c) - alpha * n|)
    //      = 1 - alpha (since alpha <= 1)
    //  assume x' is equal to x, but with some x'_i > c added
    //      = max | -|#(x <= c) - alpha * n| - -|#(x' <= c) - alpha * (n + 1)||
    //      = max | -|#(x <= c) - alpha * n| - -|#(x <= c) - alpha * (n + 1)|| (by the assumption)
    //      = max | -|#(x <= c) - alpha * n| - -|#(x <= c) - alpha * n - alpha||
    //     <= max | -|#(x <= c) - alpha * n| - -(|#(x <= c) - alpha * n| + |-alpha|)|
    //      = max | a - a - |alpha|| (where a = -|#(x <= c) - alpha * n|)
    //      = alpha (since alpha >= 0)
    //  via union bound, in all four cases, for any addition or removal, sensitivity bounded above by max(alpha, 1 - alpha)
    //  therefore max d_abs(s, s') == max(alpha, 1 - alpha)
    let abs_dist_const = alpha.max(TO::one().inf_sub(&alpha)?);

    // distance between score vectors on neighboring datasets
    //    max d_sup(sv, sv')    (where sv is a score vector)
    //  = max_{ij} |d(sv_i, sv_j) - d(sv'_i, sv'_j)|
    //  = max_{ij} ||sv_i - sv_j| - |sv'_i - sv'_j||
    // <= max_{ij} |(sv_i - sv_j) - (sv'_i - sv'_j)| (by reverse triangle inequality)
    //  = max_{ij} |(sv_i - sv'_i) - (sv_j - sv'_j)|
    // <= max_i |sv_i - sv'_i| + max_j |sv_j - sv'_j| (by triangle inequality)
    //  = 2 * max_i |sv_i - sv'_i|
    //  = 2 * abs_dist_const
    let sup_dist_const = (TO::one() + TO::one()).inf_mul(&abs_dist_const)?;

    // for comparison, if we were to assume monotonicity:
    //    max d_sup(sv, sv')    (where sv is a score vector)
    //  = max_{ij} |d(sv_i, sv_j) - d(sv'_i, sv'_j)|
    //  = max_{ij} ||sv_i - sv_j| - |sv'_i - sv'_j||
    // <= max_{ij} |(sv_i - sv_j) - (sv'_i - sv'_j)| (by reverse triangle inequality)
    //  = max_{ij} |(sv_i - sv'_i) - (sv_j - sv'_j)|
    //  = max_{ij} ||sv_i - sv'_i| - |sv_j - sv'_j|| (since sv'_k >= sv_k for all k)
    // <= max_{ij} ||sv_i - sv'_i| - 0| (maximized when second term is zero)
    //  = max_i |sv_i - sv'_i|

    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<TI>| score(arg.clone(), &candidates, alpha.clone())),
        SymmetricDistance::default(),
        SupDistance::default(),
        StabilityRelation::new_from_constant(sup_dist_const),
    ))
}

/// Makes a [Transformation] that scores how similar each candidate is to the given `alpha`-quantile on the input dataset.
pub fn make_sized_quantile_scorer_de<TI, TO>(
    size: usize,
    candidates: Vec<TI>,
    alpha: TO,
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<AllDomain<TI>>>,
        VectorDomain<AllDomain<TO>>,
        SymmetricDistance,
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
    // distances between candidate scores on neighboring datasets
    //    max d_abs(s, s')    (where s is a candidate score)
    //  = max |s - s'| 
    //  = max | -|#(x <= c) - alpha * n| - -|#(x' <= c) - alpha * n||
    //  assume x' is equal to x, but with some x_i <= c replaced with x'_i > c
    //      = max | -|#(x <= c) - alpha * n| - -|#(x' <= c) - alpha * n||
    //      = max | -|#(x <= c) - alpha * n| - -|#(x <= c) - 1 - alpha * n||  (by the assumption)
    //     <= max | -|#(x <= c) - alpha * n| - -(|#(x <= c) - alpha * n| + 1)|  (by triangle inequality)
    //      = max | -a - -(a + 1)|  (where a = |#(x <= c) - alpha * n|)
    //      = 1
    //  assume x' is equal to x, but with some x_i > c replaced with x'_i <= c
    //     by symmetry, distance also <= 1
    //  assume x' is equal to x, but with some x_i <= c replaced with x'_i <= c
    //      = max | -|#(x <= c) - alpha * n| - -|#(x' <= c) - alpha * n||
    //      = max | -|#(x <= c) - alpha * n| - -|#(x <= c) - alpha * n|| (because #(x <= c) == #(x' <= c))
    //      = max | a - a | (where a = -|#(x <= c) - alpha * n|)
    //      = 0
    //  assume x' is equal to x, but with some x_i > c replaced with x'_i > c
    //     by symmetry, distance also == 0
    //  via union bound, in all four cases, for any addition and removal, sensitivity bounded above by 1
    //  therefore max d_abs(s, s') == 1 / 2 (because it takes two changes to get a difference of one)
    
    // distance between score vectors on neighboring datasets
    //    max d_sup(sv, sv')    (where sv is a score vector)
    //  = max_{ij} |d(sv_i, sv_j) - d(sv'_i, sv'_j)|
    //  = max_{ij} ||sv_i - sv_j| - |sv'_i - sv'_j||
    // <= max_{ij} |(sv_i - sv_j) - (sv'_i - sv'_j)| (by reverse triangle inequality)
    //  = max_{ij} |(sv_i - sv'_i) - (sv_j - sv'_j)|
    // <= max_i |sv_i - sv'_i| + max_j |sv_j - sv'_j| (by triangle inequality)
    //  = 2 * max_i |sv_i - sv'_i|
    // <= 2 * (1 / 2)  (by the "distances between candidate scores on neighboring datasets" proof)

    
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new_all(), size),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<TI>| score(arg.clone(), &candidates, alpha.clone())),
        SymmetricDistance::default(),
        SupDistance::default(),
        StabilityRelation::new_from_constant(TO::one()),
    ))
}

/// Compute score of each candidate on a dataset
/// Formula is -|#(x <= c) - alpha * n| for each c in `candidates`.
/// Can be understood as -|observed_value - ideal_value|. 
///     We want greater scores when observed value is near ideal value.
///     The further away the observed value is from the ideal value, the more negative it gets
/// TODO: why is the scorer the same as in the continuous case?
///     http://cs-people.bu.edu/ads22/pubs/2011/stoc194-smith.pdf#page=7
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
    let ideal_value = alpha * TO::exact_int_cast(x.len())?;
    num_lte
        .into_iter()
        .map(|v| TO::exact_int_cast(v).map(|v| -(v - ideal_value).abs()))
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
