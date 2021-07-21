use crate::dom::{AllDomain, IntervalDomain, InherentNullDomain, InherentNull, VectorDomain, SizedDomain};
use crate::core::Domain;
use crate::dist::{AbsoluteDistance, SymmetricDistance};

// How do we get rid of I?
// I is the type that the domain is parameterized by.
// - It exists for the Subdomain bound
// - It breaks the trait
pub trait CompatiblePairing<I> {}

impl<T, D: Subdomain<AllDomain<T>>, Q> CompatiblePairing<T> for (D, AbsoluteDistance<Q>) {}
impl<DA: Domain, D: Subdomain<VectorDomain<DA>>> CompatiblePairing<DA> for (D, SymmetricDistance) {}


pub trait Subdomain<D1: Domain>: Domain {}
impl<T> Subdomain<AllDomain<T>> for AllDomain<T> {}
impl<T: PartialOrd + Clone> Subdomain<AllDomain<T>> for IntervalDomain<T> {}
impl<D: Domain> Subdomain<AllDomain<D::Carrier>> for InherentNullDomain<D>
    where D::Carrier: InherentNull {}
impl<D: Domain> Subdomain<VectorDomain<D>> for VectorDomain<D> {}
impl<D: Domain> Subdomain<VectorDomain<D>> for SizedDomain<VectorDomain<D>> {}


// This impl requires that the inner domain of VectorDomain is a Subdomain of AllDomain
// - It has the same issue that caused the I generic on CompatiblePairing to pop out
// // type parameter T is unconstrained
// impl<T, D: Subdomain<AllDomain<T>>> Subdomain<VectorDomain<D>> for VectorDomain<D> {}
// // type parameter T is unconstrained
// impl<T, D: Subdomain<AllDomain<T>>> Subdomain<VectorDomain<D>> for SizedDomain<VectorDomain<D>> {}
