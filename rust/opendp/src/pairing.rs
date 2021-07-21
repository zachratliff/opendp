use crate::dom::{AllDomain, IntervalDomain, InherentNullDomain, InherentNull, VectorDomain, SizedDomain};
use crate::core::Domain;
use crate::dist::{AbsoluteDistance, SymmetricDistance};

pub trait CompatiblePairing {}
// type parameter T is unconstrained
impl<T, D: Subdomain<AllDomain<T>>, Q> CompatiblePairing for (D, AbsoluteDistance<Q>) {}
impl<D: Subdomain<VectorDomain<D>>> CompatiblePairing for (D, SymmetricDistance) {}


pub trait Subdomain<D1: Domain>: Domain {}
impl<T> Subdomain<AllDomain<T>> for AllDomain<T> {}
impl<T: PartialOrd + Clone> Subdomain<AllDomain<T>> for IntervalDomain<T> {}
impl<T, D: Subdomain<AllDomain<T>>> Subdomain<AllDomain<T>> for InherentNullDomain<D>
    where T: InherentNull {}
// type parameter T is unconstrained
impl<T, D: Subdomain<AllDomain<T>>> Subdomain<VectorDomain<D>> for VectorDomain<D> {}
// type parameter T is unconstrained
impl<T, D: Subdomain<AllDomain<T>>> Subdomain<VectorDomain<D>> for SizedDomain<VectorDomain<D>> {}
