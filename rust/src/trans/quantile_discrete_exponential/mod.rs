use num::{Float, One};
use std::cmp::Ordering;

use crate::{
    core::{Function, StabilityRelation, Transformation},
    dist::{InfDistance, IntDistance, SubstituteDistance, SymmetricDistance},
    dom::{AllDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{CheckNull, DistanceConstant, ExactIntCast},
};

pub fn make_quantile_discrete_exponential<TI, TO>(
    candidates: Vec<TI>,
    alpha: TO,
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TI>>,
        VectorDomain<AllDomain<TO>>,
        SymmetricDistance,
        InfDistance<TO>,
    >,
>
where
    TI: 'static + CheckNull + Clone + PartialOrd,
    TO: CheckNull + Float + DistanceConstant<IntDistance> + ExactIntCast<usize>,
    IntDistance: DistanceConstant<TO>,
{
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<TI>| {
            score(arg.clone(), candidates.clone(), alpha.clone())
        }),
        SymmetricDistance::default(),
        InfDistance::default(),
        StabilityRelation::new_from_constant(alpha.max(TO::one() - alpha)),
    ))
}

pub fn make_sized_quantile_discrete_exponential<TI, TO>(
    size: usize,
    candidates: Vec<TI>,
    alpha: TO,
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<AllDomain<TI>>>,
        VectorDomain<AllDomain<TO>>,
        SubstituteDistance,
        InfDistance<TO>,
    >,
>
where
    TI: 'static + CheckNull + Clone + PartialOrd,
    TO: CheckNull + Clone + One + Float + DistanceConstant<IntDistance> + ExactIntCast<usize>,
    IntDistance: DistanceConstant<TO>,
{
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new_all(), size),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<TI>| {
            score(arg.clone(), candidates.clone(), alpha.clone())
        }),
        SubstituteDistance::default(),
        InfDistance::default(),
        StabilityRelation::new_from_constant(TO::one()),
    ))
}

/// Compute score of each candidates on a dataset
/// Formula is n * max(alpha, 1 - alpha) - |(1 - alpha) * #(Z < r) - alpha * #(Z > r)|
///
/// # Arguments
/// * `x` - dataset to score against
/// * `candidates` - values to be scored
/// * `alpha` - parameter for quantile. {0: min, 0.5: median, 1: max, ...}
///
/// # Returns
/// Utility for each candidate
fn score<TI, TO>(mut x: Vec<TI>, candidates: Vec<TI>, alpha: TO) -> Fallible<Vec<TO>>
where
    TI: PartialOrd + Clone,
    TO: Float + ExactIntCast<usize>,
{
    // sort candidates but preserve original ordering
    let mut candidates = candidates
        .into_iter()
        .enumerate()
        .collect::<Vec<(usize, TI)>>();
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
    x.sort_by(|a, b| a.partial_cmp(&b).unwrap_or(Ordering::Equal));

    let mut x_idx: usize = 0;
    let mut cand_idx: usize = 0;
    let mut utilities = Vec::with_capacity(candidates.len());

    // prepend utilities for candidates less than smallest value of the dataset
    if let Some(v) = x.first() {
        let candidate_score = score_candidate(x_idx, x.len() - x_idx, alpha)?;
        while cand_idx < candidates.len() && candidates[cand_idx].1 < *v {
            utilities.push(candidate_score);
            cand_idx += 1;
        }
    }

    while cand_idx < candidates.len() && x_idx < x.len() {
        match x[x_idx].partial_cmp(&candidates[cand_idx].1) {
            None => (),
            Some(Ordering::Less) => x_idx += 1,
            // if ith value is equal, then there are
            //   i values smaller than the current candidate
            //   loop to find number of values larger than current candidate
            Some(Ordering::Equal) => {
                let num_lt = x_idx;
                let num_gt = loop {
                    x_idx += 1;
                    // if all elements are lte, then num_lte == n, so num_gt must be 0
                    if x_idx == x.len() {
                        break x.len() - x_idx;
                    }
                    // if next value is greater than candidate,
                    //  then num_gt is n - num_lte
                    if x[x_idx] > candidates[cand_idx].1 {
                        break x.len() - x_idx;
                    }
                };
                // score the candidate
                let candidate_score = score_candidate(num_lt, num_gt, alpha)?;
                // reuse the score for all equivalent candidates
                while cand_idx < candidates.len() && candidates[cand_idx].1 == x[num_lt] {
                    utilities.push(candidate_score);
                    cand_idx += 1;
                }
            }
            // if the ith value is larger, then there are
            //  i values smaller than the current candidate
            //  n - i values larger
            Some(Ordering::Greater) => {
                utilities.push(score_candidate(x_idx, x.len() - x_idx, alpha)?);
                cand_idx += 1;
            }
        }
    }

    // append utilities for candidates greater than the maximum value of the dataset
    let candidate_score = score_candidate(x.len(), 0, alpha)?;
    utilities.extend((0..candidates.len() - utilities.len()).map(|_| candidate_score));

    // order the utilities by the order of the candidates before they were sorted, and shift the utility
    let constant = alpha.max(TO::one() - alpha);
    let x_len = TO::exact_int_cast(x.len())?;
    Ok(candidates
        .into_iter()
        .map(|(idx, _)| constant * x_len - utilities[idx])
        .collect())
}

fn score_candidate<TO: Float + One + ExactIntCast<usize>>(
    num_lt: usize,
    num_gt: usize,
    alpha: TO,
) -> Fallible<TO> {
    Ok(
        ((TO::one() - alpha) * TO::exact_int_cast(num_lt)? - alpha * TO::exact_int_cast(num_gt)?)
            .abs(),
    )
}
