#define R_NO_REMAP
#define STRICT_R_HEADERS

#include <R.h>
#include <Rinternals.h>
#include <stdio.h>

// Import C headers for rust API
#include "opendp_ffi_base.h"
#include "opendp_ffi.h"

SEXP slice_as_object__wrapper(SEXP data) {
    // TODO: unwrap data, translate into ffislice. See _convert.py for direction
    double data2 = 1.;
    uintptr_t len = 1;
    FfiSlice slice = { &data2, len};

    char *typename = malloc(4);
    strcpy(typename, "f64\0");

    FfiResult_____AnyObject result = opendp_data__slice_as_object(&slice, typename);

    // TODO: result unwrapping and return opaque AnyObject struct
    printf("Success or error: %d", result.tag);

    // the function returns the input just so that the code compiles
    return data;
}

// R boilerplate
static const R_CallMethodDef CallEntries[] = {
        // name of routine, pointer to function, number of arguments
        {"slice_as_object__wrapper", (DL_FUNC) &slice_as_object__wrapper, 1},
        // ...repeat for each function exported by opendp.so, used by the R library

        // why? This boilerplate line is everywhere
        {NULL, NULL, 0}
};

void R_init_opendp(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}
