import opendp


gaussian = opendp.make_base_gaussian("<f64>", 1.)


# mean = opendp.make_bounded_mean("<HammingDistance, L1Sensitivity<f64>>", 0., 10., 5)

# print(opendp.opendp_core__bootstrap())
# count = opendp.make_count("<HammingDistance, L1Sensitivity<f64>>")

