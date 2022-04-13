use std::ops::Neg;

use num::Float;

use crate::{
    core::{Function, Measurement, PrivacyRelation, SensitivityMetric},
    dist::{SupDistance, MaxDivergence},
    dom::{AllDomain, VectorDomain},
    error::Fallible,
    samplers::SampleUniform,
    traits::{CheckNull, InfMul},
};

pub fn make_base_discrete_exponential<MI>(
    scale: MI::Distance,
    constant_time: bool,
) -> Fallible<
    Measurement<
        VectorDomain<AllDomain<MI::Distance>>,
        AllDomain<usize>,
        SupDistance<MI>,
        MaxDivergence<MI::Distance>,
    >,
>
where
    MI: SensitivityMetric,
    MI::Distance: 'static + CheckNull + Float + SampleUniform + InfMul,
{
    Ok(Measurement::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new_fallible(move |arg: &Vec<MI::Distance>| {
            arg.iter()
                .copied()
                .map(|v| scale * v)
                // enumerate before sampling so that indexes are inside the result
                .enumerate()
                .map(|(i, llik)| {
                    MI::Distance::sample_standard_uniform(constant_time)
                        .map(|v| (i, llik - v.ln().neg().ln()))
                })
                // retrieve the highest noisy likelihood pair
                .try_fold(
                    (arg.len(), MI::Distance::neg_infinity()),
                    |acc: (usize, MI::Distance), res| {
                        res.map(|v| if acc.1 > v.1 { acc } else { v })
                    },
                )
                // only return the index
                .map(|v| v.0)
        }),
        SupDistance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_all(
            move |d_in: &MI::Distance, d_out: &MI::Distance| {
                if d_in.is_sign_negative() {
                    return fallible!(InvalidDistance, "sensitivity must be non-negative");
                }
                if d_out.is_sign_negative() {
                    return fallible!(InvalidDistance, "epsilon must be non-negative");
                }
                // d_out * scale >= d_in
                Ok(d_out.neg_inf_mul(&scale)? >= d_in.clone())
            },
            Some(move |d_out: &MI::Distance| d_out.neg_inf_mul(&scale)),
        ),
    ))
}

#[cfg(test)]
pub mod test_exponential {
    use crate::dist::AbsoluteDistance;

    use super::*;
    #[test]
    fn test_exponential() -> Fallible<()> {
        let de = make_base_discrete_exponential::<AbsoluteDistance<_>>(1., false)?;
        let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
        println!("{:?}", release);

        Ok(())
    }
}
