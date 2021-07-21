use crate::dom::{AllDomain, IntervalDomain, InherentNullDomain, InherentNull, VectorDomain, SizedDomain, OptionNullDomain};
use crate::core::Domain;
use crate::dist::{AbsoluteDistance, SymmetricDistance};

pub trait CompatiblePairing {}

impl<D: Inner + Subdomain<AllDomain<D::Inner>>, Q> CompatiblePairing for (D, AbsoluteDistance<Q>) {}
impl<D: Inner + Subdomain<VectorDomain<D::Inner>>> CompatiblePairing for (D, SymmetricDistance)
    where D::Inner: Domain {}


// identify a type shared by all domains in an equivalence class
// used to bundle a type `Inner` that would otherwise leak out from the trait
pub trait Inner { type Inner; }
impl<D: Domain> Inner for VectorDomain<D> { type Inner = D; }
impl<D: Inner + Domain> Inner for SizedDomain<D> {type Inner = D::Inner; }

impl<T> Inner for AllDomain<T> { type Inner = T; }
impl<T> Inner for IntervalDomain<T> { type Inner = T; }
impl<D: Inner + Domain> Inner for OptionNullDomain<D> { type Inner = D::Inner; }


// implement subdomain for any domain that is a subdomain of a more general domain
pub trait Subdomain<D1: Domain>: Domain {}
impl<T> Subdomain<AllDomain<T>> for AllDomain<T> {}
impl<T: PartialOrd + Clone> Subdomain<AllDomain<T>> for IntervalDomain<T> {}
impl<D: Domain> Subdomain<AllDomain<D::Carrier>> for InherentNullDomain<D>
    where D::Carrier: InherentNull {}

impl<D: Inner + Subdomain<AllDomain<D::Inner>>> Subdomain<VectorDomain<D>> for VectorDomain<D> {}
impl<D: Inner + Subdomain<AllDomain<D::Inner>>> Subdomain<VectorDomain<D>> for SizedDomain<VectorDomain<D>> {}
