# CCS'24 Artifact

Paper: "A Framework for Differential Privacy Against Timing Attacks"

### Installation
1. You will need to install the Rust toolchain. See https://www.rust-lang.org/tools/install for quickly setting up Rust on your OS. 
2. Make sure you have the latest version by running `rustup update`
3. Clone the OpenDP fork and checkout the correct branch:
`git clone git@github.com:zachratliff/opendp.git && git checkout 931-timing && cd opendp/rust`
4. **Running the tests**: Run the Laplace timing delay tests from the `opendp/rust` directory by running the following command from your terminal. 
`cargo test --package opendp --lib --features untrusted --features bindings -- --nocapture -- combinators::laplace_delay::test::test_laplace_delay --exact --show-output`

For more detailed instructions on installing Rust and OpenDP you can refer to the official OpenDP user guide: https://docs.opendp.org/en/stable/contributing/development-environment.html

#### Expected output

When you run the timing delay tests you should expect to see output like the following:

```
running 1 test
release: 5
Epsilon budget spent on DP count: 2.0
Combined Epsilon, Delta budget spent on DP count and Laplace Delay: (2.1, 0.00010034936411235076)
test combinators::laplace_delay::test::test_laplace_delay ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 216 filtered out; finished in 0.01s
```

The output corresponds to a DP release of a private count on a dataset of size 5. 
The program takes a **non**-timing private measurement (computing a DP count with epsilon = 2.0) and chains it with an implementation of the Discrete Laplace timing-private delay program discussed in Section 4 of the paper. The test assumes that the counting mechanism is 10-_OC-timing stable_ (Definition 3.3), where the time units are nanoseconds, and parameterize the Discrete Laplace distribution with a scale of 100ns and a shift of 1,000ns. The output gives the combined (epsilon, delta) privacy budget used on privatizing both the output and the running-time. 

The relevant source files are:
- `/rust/src/combinators/laplace_delay/mod.rs`
- `/rust/src/combinators/laplace_delay/test.rs`

If you want to modify the privacy budget spent on making the runtime private, change the parameters in line 98 of `test.rs`:

`let m_count_delayed = make_laplace_delay(&m_count_ts, 1_000, 100.0)?;`

The above line indicates a shift = 1_000 and scale = 100.0. 

**Note**: The OC-timing stability constant of 10 nanoseconds is only an estimate on an upper bound on the mechanism's timing stability in our execution environment, and a 
more careful analysis of timing stability would be required to ensure that the program actually achieves timing privacy. As mentioned in the paper, the purpose of the implementation is to illustrate the compatibility of our framework with OpenDP (and other DP libraries) **not to claim that the implementation provides timing privacy for physical executions**. We leave for future work the problem of instantiating our framework for physical timing channels, which would involve constraining the execution environment, identifying the appropriate units to measure timing, and finding realistic upper bounds on actual timing-stability constants.
