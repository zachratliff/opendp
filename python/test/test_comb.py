from opendp.mod import enable_features
from opendp.typing import *

enable_features("floating-point")


def test_amplification():
    from opendp.trans import make_bounded_mean
    from opendp.meas import make_base_laplace
    from opendp.comb import make_population_amplification

    meas = make_bounded_mean(0., 10., 10) >> make_base_laplace(scale=0.5)

    amplified = make_population_amplification(meas, n_population=100, DIA=IntervalDomain[float], MO=MaxDivergence[float])
    print("amplified base laplace:", amplified([1.] * 10))
    assert meas.check(1, 1.)
    assert not meas.check(1, 0.999)
    assert amplified.check(1, 0.159)
    assert not amplified.check(1, 0.158)