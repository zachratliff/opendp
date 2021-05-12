import pyarrow as pa
import pandas as pd

from pyarrow.cffi import ffi

# from https://gist.github.com/wesm/d48908018c4b7a0d9789a31d10caf525?short_path=663194b#file-cinterfaceexample-ipynb

#
# 1. Make data
df = pd.DataFrame({'a': [1, 2, 3, 4, 5],
                   'b': ['a', 'b', 'c', 'd', 'e']})
rb = pa.record_batch(df)
int_array = pa.Int64Array.from_pandas(pd.Series([1, 2, 3, 4, 5]))

#
# 2. Export pyarrow.RecordBatch to C Interface

c_schema = ffi.new("struct ArrowSchema*")
c_schema_ptr = int(ffi.cast("uintptr_t", c_schema))

# NB: RecordBatch is packed as a StructArray
c_batch = ffi.new("struct ArrowArray*")
c_batch_ptr = int(ffi.cast("uintptr_t", c_batch))

# rb.schema._export_to_c(c_schema_ptr)
# rb._export_to_c(c_batch_ptr)

int_array.schema._export_to_c(c_schema_ptr)
int_array._export_to_c(c_batch_ptr)

#
# 3. Call into Rust and transform
lib_path = '/Users/michael/openDP/openDP/rust/target/debug/libopendp_ffi.dylib'
import ctypes
lib = ctypes.cdll.LoadLibrary(lib_path)
c_batch_ptr = lib.double(c_schema_ptr, c_batch_ptr)

#
# 4. Import pyarrow.RecordBatch given addresses of ArrowSchema, ArrowArray

# Deserialize schema
schema2 = pa.Schema._import_from_c(c_schema_ptr)

# Deserialize batch
rb2 = pa.RecordBatch._import_from_c(c_batch_ptr, schema2)

#
# 5. Test data
(rb * 2).equals(rb2)
rb2.to_pandas()
