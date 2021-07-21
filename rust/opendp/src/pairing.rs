use crate::dom::{AllDomain, IntervalDomain, InherentNullDomain, InherentNull, VectorDomain, SizedDomain};
use crate::core::Domain;
use crate::dist::{AbsoluteDistance, SymmetricDistance};

pub trait CompatiblePairing {}

impl<D: Inner + Subdomain<AllDomain<D::Inner>>, Q> CompatiblePairing for (D, AbsoluteDistance<Q>) {}
impl<D: Inner + Subdomain<VectorDomain<D::Inner>>> CompatiblePairing for (D, SymmetricDistance)
    where D::Inner: Domain {}


pub trait Inner { type Inner; }
impl<D: Domain> Inner for VectorDomain<D> { type Inner = D; }
impl<T> Inner for AllDomain<T> { type Inner = T; }
impl<D: Domain> Inner for SizedDomain<VectorDomain<D>> { type Inner = D; }


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
