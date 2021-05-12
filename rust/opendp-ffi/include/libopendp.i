%module libopendp

//
// %include "typemaps.i"

// convert from rust representation to PyObject
// %typemap(out) FfiResult<FfiTransformation*>* {
//
// }


%typemap(in) (void const*) {
    // TODO: how to pass type annotation down?
    //     wrap in tuple, and parse the tuple?

    // https://docs.python.org/3/c-api/long.html
    if PyLong_Check($input) {
        // don't need to incref as it is not a python object
        long temp = PyLong_AsLong($input);
        $1 = (void*) &temp;
    }
    else if PyFloat_Check($input) {
        double temp = PyFloat_AsDouble($input);
        $1 = (void*) &temp;
    }
    else if PyBool_Check($input) {
        bool temp = PyObject_IsTrue($input) == 1;
        $1 = (void*) &temp;
    }
//     else if (PyObject_IsInstance($input, (PyObject *)&MyType)) Py_RETURN_TRUE;


//     if PyTuple_Check($input) {
//         void *array[2];
//         // these need to be unwrapped individually, can't just take a pyobject*
//         array[0] = (void*) PyTuple_GetItem($input, 0);
//         array[1] = (void*) PyTuple_GetItem($input, 1);
//         $1 = array;
//     }
}


// this content is copied directly into the .c file.
%{
#include "libopendp.h"
%}

// this %include is a macro to generate the wrappers around the functions in the .h file
%include "libopendp.h"