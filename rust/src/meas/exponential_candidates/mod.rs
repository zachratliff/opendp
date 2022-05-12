use num::Float;

use crate::{
    core::{Function, Measurement, PrivacyRelation},
    dist::{InfDifferenceDistance, MaxDivergence},
    dom::{AllDomain, VectorDomain},
    error::Fallible,
    samplers::SampleUniform,
    traits::{CheckNull, InfMul, InfDiv},
};

pub fn make_base_exponential_candidates<TI>(
    temperature: TI,
    constant_time: bool,
) -> Fallible<
    Measurement<
        VectorDomain<AllDomain<TI>>,
        AllDomain<usize>,
        InfDifferenceDistance<TI>,
        MaxDivergence<TI>,
    >,
>
where
    TI: 'static + CheckNull + Float + SampleUniform + InfMul + InfDiv,
{
    Ok(Measurement::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new_fallible(move |arg: &Vec<TI>| {
            arg.iter()
                .copied()
                .map(|v| v / temperature)
                // enumerate before sampling so that indexes are inside the result
                .enumerate()
                // using the gumbel sampling trick: https://lips.cs.princeton.edu/the-gumbel-max-trick-for-discrete-distributions/
                .map(|(i, llik)| {
                    TI::sample_standard_uniform(constant_time)
                        .map(|u| (i, llik - u.ln().neg().ln()))
                })
                // retrieve the highest noisy likelihood pair
                .try_fold(
                    (arg.len(), TI::neg_infinity()),
                    |acc: (usize, TI), res| {
                        res.map(|v| if acc.1 > v.1 { acc } else { v })
                    },
                )
                // only return the index
                .map(|v| v.0)
        }),
        InfDifferenceDistance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_all(
            move |d_in: &TI, d_out: &TI| {
                if d_in.is_sign_negative() {
                    return fallible!(InvalidDistance, "sensitivity must be non-negative");
                }
                if d_out.is_sign_negative() {
                    return fallible!(InvalidDistance, "epsilon must be non-negative");
                }
                // d_out >= d_in / temperature
                Ok(d_out.neg_inf_mul(&temperature)? >= d_in.clone())
            },
            Some(move |d_out: &TI| d_out.neg_inf_mul(&temperature)),
        ),
    ))
}

#[cfg(test)]
pub mod test_exponential {
    use super::*;
    
    #[test]
    fn test_exponential() -> Fallible<()> {
        let de = make_base_exponential_candidates(1., false)?;
        let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
        println!("{:?}", release);

        Ok(())
    }
}
