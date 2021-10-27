Work-in-progress R bindings for OpenDP.

1. Install devtools:

    ```R   
    install.packages("devtools")
    ```

2. Install OpenDP R package:

    ```R
    # - runs src/Makevars
    # - builds libopendp_ffi.a and opendp_ffi.h
    # - copies opendp_ffi.h into src/
    # - compiles wrapper.c, which includes the two .h files and statically links with libopendp_ffi.a
    # - outputs src/opendp.so, 
    pkgbuild::compile_dll()
    devtools::document() 
    ```

3. Call into the rust OpenDP library:

    ```R
    opendp::slice_to_object("ignored data")
    ```

    This should emit:
    > Success or error: 0[1] "ignored data"

The following command currently fails:

```R
devtools::install()
```

It fails because Makevars uses a relative path to the rust directory.
When the package is installed, the directory is first lifted away to ensure that the package builds in isolation.
There are a couple solutions:
1. Add a symlink from /R/src/rust to /rust  
   - Would have to be a relative link to work with git, which similarly breaks on install  
   - Would cause cross platform issues w/ windows  
1. Copy the /rust into /R/src  
   - Slow to copy and easily gets desynchronized  
1. Only copy /rust into /R/src when deploying the package  
   - The typical dev loop is `devtools::load_all()`. 
   - Never install the dev package  
   - This option seems ideal so far
1. Something else? What is the best practice here?


### Major Components to implement
The python library is generally a pretty good model for how this can be implemented.

1. Low-level type conversions between the bindings language and Rust  
    `python/src/_convert.py` -> `R/src/wrapper.c` or `R/src/convert.c`
2. Codegen for R constructor functions  
    `rust/opendp-ffi/build/python` -> `rust/opendp-ffi/build/R`  
    Constructor functions in `meas.py`, `trans.py`, `data.py` and `comb.py`
    are all generated from bootstrap.json metadata.
    Similarly in R, `meas.R`, `trans.R`, etc. can be generated into `R/inst`
3. Tools for manipulating types idiomatically in R  
   `python/src/typing.py` -> `R/src/typing.R`
4. Adjust smoke-test.yml CI to automatically run testthat R/tests
