use crate::{error::Fallible, traits::{FloatBits, ExactIntCast}};

use super::fill_bytes;
use ieee754::Ieee754;

pub trait SampleUniform: Sized {

    /// Returns a random sample from Uniform[0,1).
    ///
    /// This algorithm is taken from [Mironov (2012)](http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.366.5957&rep=rep1&type=pdf)
    /// and is important for making some of the guarantees in the paper.
    ///
    /// The idea behind the uniform sampling is to first sample a "precision band".
    /// Each band is a range of floating point numbers with the same level of arithmetic precision
    /// and is situated between powers of two.
    /// A band is sampled with probability relative to the unit of least precision using the Geometric distribution.
    /// That is, the uniform sampler will generate the band [1/2,1) with probability 1/2, [1/4,1/2) with probability 1/4,
    /// and so on.
    ///
    /// Once the precision band has been selected, floating numbers numbers are generated uniformly within the band
    /// by generating a 52-bit mantissa uniformly at random.
    ///
    /// # Arguments
    ///
    /// `min`: f64 minimum of uniform distribution (inclusive)
    /// `max`: f64 maximum of uniform distribution (non-inclusive)
    ///
    /// # Return
    /// Random draw from Unif[min, max).
    ///
    /// # Example
    /// ```
    /// // valid draw from Unif[0,1)
    /// use opendp::traits::samplers::SampleUniform;
    /// let unif = f64::sample_standard_uniform(false);
    /// # use opendp::error::ExplainUnwrap;
    /// # unif.unwrap_test();
    /// ```
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self>;
}

impl SampleUniform for f64 {
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self> {
        // The unbiased exponent of Uniform([0, 1)) is in [-1023, -1].
        //
        // # Lower bound of -1023:
        // Zero and subnormal numbers have a biased exponent of 0 -> an unbiased exponent of -1023
        // 
        // # Upper bound of -1:
        // A saturated mantissa is ~2, so the unbiased exponent must be <= -1, because Uniform([0, 1)) is < 1.
        //   sign     exp    mantissa
        //   (-1)^0 * 2^-1 * 1.9999... ~ 1

        // Use rejection sampling to draw ~ TruncatedGeometric(p=0.5, bounds=[0, 1022])
        // Reject samples > 1022 to redistribute the probability amongst all exponent bands
        let truncated_geometric_sample = loop {
            if let Some(e) = Self::sample_uniform_exponent(constant_time)? {
                break e
            }
        };

        // Transform [0, 1022] -> [-1023, -1]
        let exponent = -i16::exact_int_cast(truncated_geometric_sample)? - 1;

        let mantissa: u64 = Self::sample_uniform_mantissa()?;

        // Generate uniform random number from [0,1)
        Ok(Self::recompose(false, exponent, mantissa))
    }
}

impl SampleUniform for f32 {
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self> {
        // The unbiased exponent of Uniform([0, 1)) is in [-127, -1].
        //
        // # Lower bound of -127:
        // Zero and subnormal numbers have a biased exponent of 0 -> an unbiased exponent of -127
        // 
        // # Upper bound of -1:
        // A saturated mantissa is ~2, so the unbiased exponent must be <= -1, because Uniform([0, 1)) is < 1.
        //   sign     exp    mantissa
        //   (-1)^0 * 2^-1 * 1.9999... ~ 1

        // Use rejection sampling to draw ~ TruncatedGeometric(p=0.5, bounds=[0, 126])
        // Reject samples > 126 to redistribute the probability amongst all exponent bands
        let truncated_geometric_sample = loop {
            if let Some(e) = Self::sample_uniform_exponent(constant_time)? {
                break e
            }
        };

        // Transform [0, 126] -> [-127, -1]
        let exponent = -i16::exact_int_cast(truncated_geometric_sample)? - 1;

        let mantissa: u32 = Self::sample_uniform_mantissa()?;

        // Generate uniform random number from [0,1)
        Ok(Self::recompose(false, exponent, mantissa))
    }
}

pub trait SampleUniformExponent: FloatBits {
    fn sample_uniform_exponent(constant_time: bool) -> Fallible<Option<Self::Bits>>;
}

// This issue is blocking factoring these into one blanket impl:
// https://github.com/rust-lang/rust/issues/60551
// Otherwise the 128 and 16 could be supplied by an associated const on FloatBits.
impl SampleUniformExponent for f64 {
    fn sample_uniform_exponent(constant_time: bool) -> Fallible<Option<Self::Bits>> {
        // find index of the first true bit in a randomly sampled byte buffer
        Ok(sample_geometric_buffer::<128>(constant_time)?
            // cast to the bits type. This cast lossless and infallible
            .map(|v| v as Self::Bits)
            // reject success on last coin flip because last flip is reserved for inf, -inf, NaN
            .and_then(|v| (v != Self::EXPONENT_BIAS).then_some(v)))
    }
}

impl SampleUniformExponent for f32 {
    fn sample_uniform_exponent(constant_time: bool) -> Fallible<Option<Self::Bits>> {
        // find index of the first true bit in a randomly sampled byte buffer
        Ok(sample_geometric_buffer::<16>(constant_time)?
            // cast to the bits type. This cast lossless and infallible
            .map(|v| v as Self::Bits)
            // reject success on last coin flip because last flip is reserved for inf, -inf, NaN
            .and_then(|v| (v != Self::EXPONENT_BIAS).then_some(v)))
    }
}

pub trait SampleUniformMantissa: FloatBits {
    fn sample_uniform_mantissa() -> Fallible<Self::Bits>;
}

impl SampleUniformMantissa for f64 {
    fn sample_uniform_mantissa() -> Fallible<Self::Bits> {
        // Of a 64 bit buffer, we want the first 12 bits to be zero, 
        //    and the last 52 bits to be uniformly random
        let mut mantissa_buffer = [0u8; 8];
        // Fill the last 56 bits with randomness.
        fill_bytes(&mut mantissa_buffer[1..])?;
        // Clear the leftmost four bits of the second byte
        mantissa_buffer[1] &= 0b00001111;

        // convert mantissa to integer
        Ok(u64::from_be_bytes(mantissa_buffer))
    }
}

impl SampleUniformMantissa for f32 {
    fn sample_uniform_mantissa() -> Fallible<Self::Bits> {
        // Of a 32 bit buffer, we want the first 9 bits to be zero, 
        //    and the last 23 bits to be uniformly random
        let mut mantissa_buffer = [0u8; 4];
        // Fill the last 24 bits with randomness.
        fill_bytes(&mut mantissa_buffer[1..])?;
        // Clear the leftmost bit of the second byte
        mantissa_buffer[1] &= 0b01111111;

        // convert mantissa to integer
        Ok(u32::from_be_bytes(mantissa_buffer))
    }
}


/// Return sample from a Geometric distribution with parameter p=0.5.
///
/// The algorithm generates B * 8 bits at random and returns
/// - Some(index of the first set bit)
/// - None (if all bits are 0)
///
/// This is a lower-level version of the sample_geometric trait
pub fn sample_geometric_buffer<const B: usize>(constant_time: bool) -> Fallible<Option<usize>> {
    Ok(if constant_time {
        let mut buffer = [0_u8; B];
        fill_bytes(&mut buffer)?;
        buffer.iter().enumerate()
            // ignore samples that contain no events
            .filter(|(_, &sample)| sample > 0)
            // compute the index of the smallest event in the batch
            .map(|(i, sample)| 8 * i + sample.leading_zeros() as usize)
            // retrieve the smallest index
            .min()

    } else {
        // retrieve up to B bytes, each containing 8 trials
        let mut buffer = vec![0_u8; 1];
        for i in 0..B {
            fill_bytes(&mut buffer)?;

            if buffer[0] > 0 {
                return Ok(Some(i * 8 + buffer[0].leading_zeros() as usize))
            }
        }
        None
    })
}

pub trait SampleUniformInt: Sized {
    /// sample uniformly from [Self::MIN, Self::MAX]
    fn sample_uniform_int() -> Fallible<Self>;
    /// sample uniformly from [0, upper)
    fn sample_uniform_int_0_u(upper: Self) -> Fallible<Self>;
}

macro_rules! impl_sample_uniform_unsigned_int {
    ($($ty:ty),+) => ($(
        impl SampleUniformInt for $ty {
            fn sample_uniform_int() -> Fallible<Self> {
                let mut buffer = [0; core::mem::size_of::<Self>()];
                fill_bytes(&mut buffer)?;
                Ok(Self::from_be_bytes(buffer))
            }
            fn sample_uniform_int_0_u(upper: Self) -> Fallible<Self> {
                // v % upper is unbiased for any v < MAX - MAX % upper, because
                // MAX - MAX % upper evenly folds into [0, upper) RAND_MAX/upper times
                loop {
                    // algorithm is only valid when sample_uniform_int is non-negative
                    let v = Self::sample_uniform_int()?;
                    if v <= Self::MAX - Self::MAX % upper {
                        return Ok(v % upper)
                    }
                }
            }
        }
    )+)
}
impl_sample_uniform_unsigned_int!(u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod test_uniform_int {
    use super::*;
    use std::collections::HashMap;

    #[test]
    #[ignore]
    fn test_sample_uniform_int() -> Fallible<()> {
        let mut counts = HashMap::new();
        // this checks that the output distribution of each number is uniform
        (0..10000).try_for_each(|_| {
            let sample = u32::sample_uniform_int_0_u(7)?;
            *counts.entry(sample).or_insert(0) += 1;
            Fallible::Ok(())
        })?;
        println!("{:?}", counts);
        Ok(())
    }

}