#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct FfiMeasurement;

struct FfiObject;

struct FfiTransformation;

struct FfiError {
  char *variant;
  char *message;
  char *backtrace;
};

template<typename T>
struct FfiResult {
  enum class Tag : uint32_t {
    Ok,
    Err,
  };

  struct Ok_Body {
    T _0;
  };

  struct Err_Body {
    FfiError *_0;
  };

  Tag tag;
  union {
    Ok_Body ok;
    Err_Body err;
  };
};

using c_bool = uint8_t;

struct FfiSlice {
  const void *ptr;
  uintptr_t len;
};

extern "C" {

FfiResult<FfiMeasurement*> *opendp_core__make_chain_mt(const FfiMeasurement *measurement1,
                                                       const FfiTransformation *transformation0);

FfiResult<FfiTransformation*> *opendp_core__make_chain_tt(const FfiTransformation *transformation1,
                                                          const FfiTransformation *transformation0);

FfiResult<FfiMeasurement*> *opendp_core__make_composition(const FfiMeasurement *measurement0,
                                                          const FfiMeasurement *measurement1);

bool opendp_core__error_free(FfiError *this_);

FfiResult<c_bool*> opendp_core__measurement_check(const FfiMeasurement *this_,
                                                  const FfiObject *distance_in,
                                                  const FfiObject *distance_out);

FfiResult<FfiObject*> opendp_core__measurement_invoke(const FfiMeasurement *this_,
                                                      const FfiObject *arg);

FfiResult<void*> opendp_core__measurement_free(FfiMeasurement *this_);

FfiResult<FfiObject*> opendp_core__transformation_invoke(const FfiTransformation *this_,
                                                         const FfiObject *arg);

FfiResult<void*> opendp_core__transformation_free(FfiTransformation *this_);

const char *opendp_core__bootstrap();

FfiResult<FfiObject*> opendp_data__slice_as_object(const char *type_args, const FfiSlice *raw);

FfiResult<char*> opendp_data__object_type(FfiObject *this_);

FfiResult<FfiSlice*> opendp_data__object_as_slice(const FfiObject *obj);

FfiResult<void*> opendp_data__object_free(FfiObject *this_);

/// Frees the slice, but not what the slice references!
FfiResult<void*> opendp_data__slice_free(FfiSlice *this_);

FfiResult<void*> opendp_data__str_free(char *this_);

FfiResult<void*> opendp_data__bool_free(c_bool *this_);

FfiResult<char*> opendp_data__to_string(const FfiObject *this_);

const char *opendp_data__bootstrap();

FfiResult<FfiMeasurement*> *make_base_gaussian(const char *type_args, const void *scale);

FfiResult<FfiMeasurement*> *make_base_gaussian_vec(const char *type_args, const void *scale);

FfiResult<FfiMeasurement*> *make_base_simple_geometric(const char *type_args,
                                                       const void *scale,
                                                       const void *min,
                                                       const void *max);

FfiResult<FfiMeasurement*> *make_base_laplace2(const char *type_args, const void *scale);

FfiResult<FfiMeasurement*> *make_base_laplace_vec(const char *type_args, const void *scale);

FfiResult<FfiMeasurement*> *make_base_stability(const char *type_args,
                                                uintptr_t n,
                                                const void *scale,
                                                const void *threshold);

FfiResult<FfiTransformation*> *make_split_lines(const char *type_args);

FfiResult<FfiTransformation*> *make_parse_series(const char *type_args, c_bool impute);

FfiResult<FfiTransformation*> *make_split_records(const char *type_args, const char *separator);

FfiResult<FfiTransformation*> *make_create_dataframe(const char *type_args,
                                                     const FfiObject *col_names);

FfiResult<FfiTransformation*> *make_split_dataframe(const char *type_args,
                                                    const char *separator,
                                                    const FfiObject *col_names);

FfiResult<FfiTransformation*> *make_parse_column(const char *type_args,
                                                 const void *key,
                                                 c_bool impute);

FfiResult<FfiTransformation*> *make_select_column(const char *type_args, const void *key);

FfiResult<FfiTransformation*> *make_identity(const char *type_args);

FfiResult<FfiTransformation*> *make_clamp_vec(const char *type_args,
                                              const void *lower,
                                              const void *upper);

FfiResult<FfiTransformation*> *make_clamp_scalar(const char *type_args,
                                                 const void *lower,
                                                 const void *upper);

FfiResult<FfiTransformation*> *make_cast_vec(const char *type_args);

FfiResult<FfiTransformation*> *make_bounded_sum(const char *type_args,
                                                const void *lower,
                                                const void *upper);

FfiResult<FfiTransformation*> *make_bounded_sum_n(const char *type_args,
                                                  const void *lower,
                                                  const void *upper,
                                                  unsigned int n);

FfiResult<FfiTransformation*> *make_count(const char *type_args);

FfiResult<FfiTransformation*> *make_count_by_categories(const char *type_args,
                                                        const FfiObject *categories);

FfiResult<FfiTransformation*> *make_count_by(const char *type_args, unsigned int n);

FfiResult<FfiTransformation*> *make_bounded_mean(const char *type_args,
                                                 const void *lower,
                                                 const void *upper,
                                                 unsigned int length);

FfiResult<FfiTransformation*> *make_bounded_variance(const char *type_args,
                                                     const FfiObject *lower,
                                                     const FfiObject *upper,
                                                     unsigned int length,
                                                     unsigned int ddof);

FfiResult<FfiTransformation*> *make_bounded_covariance(const char *type_args,
                                                       const FfiObject *lower,
                                                       const FfiObject *upper,
                                                       unsigned int length,
                                                       unsigned int ddof);

} // extern "C"
