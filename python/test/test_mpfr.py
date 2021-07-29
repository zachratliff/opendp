import matplotlib.pyplot as plt
samples = {"-2.0": 616, "5.0e-1": 1266, "1.0": 1565, "-4.9e-4": 2, "3.9e-3": 10, "-3.1e-2": 99, "3.1e-2": 96, "-2.5e-1": 719, "1.6e-2": 49, "2.5e-1": 735, "-5.0e-1": 1327, "7.8e-3": 25, "-3.9e-3": 13, "2.0e-3": 10, "4.0": 12, "2.0": 641, "-7.8e-3": 27, "1.2e-1": 407, "-1.2e-4": 1, "-4.0": 20, "-6.2e-2": 174, "6.2e-2": 184, "-1.0": 1567, "-9.8e-4": 1, "-2.0e-3": 9, "-1.6e-2": 49, "4.9e-4": 1, "9.8e-4": 2, "-1.2e-1": 372, "1.2e-4": 1}
scaled_samples = {"3.9e-3": 10, "9.8e-4": 1, "1.6e-2": 25, "-2.4e-4": 1, "6.2e-2": 96, "7.8e-3": 10, "-5.0e-1": 719, "-7.8e-3": 13, "-8.0": 20, "-2.0e-3": 1, "1.2e-1": 184, "-3.9e-3": 9, "2.5e-1": 407, "2.0": 1565, "-1.6e-2": 27, "-2.5e-1": 372, "5.0e-1": 735, "8.0": 12, "3.1e-2": 49, "2.4e-4": 1, "-6.2e-2": 99, "-1.2e-1": 174, "-2.0": 1567, "1.0": 1266, "-9.8e-4": 2, "-3.1e-2": 49, "-4.0": 616, "4.0": 641, "-1.0": 1327, "2.0e-3": 2}


def parse_samples(samples_):
    return {float(s): c for s, c in samples_.items()}


samples = parse_samples(samples)
scaled_samples = parse_samples(scaled_samples)

# plt.bar(samples.keys(), samples.values())
# plt.show()

filtered = {s: scaled_samples[s] for s, c in samples.items() if s in scaled_samples}
plt.bar(filtered.keys(), filtered.values())
plt.show()
