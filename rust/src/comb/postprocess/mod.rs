use crate::{
    core::{Domain, Metric, StabilityRelation, Transformation},
    dist::AgnosticMetric,
    error::Fallible,
};

pub fn make_erase_relation<DI, DO, MI, MO>(
    trans: Transformation<DI, DO, MI, MO>,
) -> Fallible<Transformation<DI, DO, AgnosticMetric, AgnosticMetric>>
where
    DI: Domain,
    DO: Domain,
    MI: Metric,
    MO: Metric,
{
    Ok(Transformation {
        input_domain: trans.input_domain,
        output_domain: trans.output_domain,
        function: trans.function,
        input_metric: AgnosticMetric::default(),
        output_metric: AgnosticMetric::default(),
        stability_relation: StabilityRelation::new_all(|_d_in, _d_out| Ok(true), None::<fn(&_)->_>, None::<fn(&_)->_>),
    })
}
