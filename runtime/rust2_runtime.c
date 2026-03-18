#include "rust2_runtime.h"
#include <ffi.h>
#include <stdarg.h>

int8_t is_r2_ptr(int64_t arg) { return !(arg & 1); };

#define START_COLOR 0

void* ml_malloc(size_t size) {
#ifdef GC
    return gc_malloc(size);
#else
    return malloc(size);
#endif
};

#define DEBUG_EX(fmt, ...)                                                                                             \
    do {                                                                                                               \
        fprintf(stderr, fmt, ##__VA_ARGS__);                                                                           \
        exit(-1);                                                                                                      \
    } while (1)

box_t* put_box(size_t size) {
    if (size % 8 != 0)
        size += 8 - (size % 8);
    box_t* result_box = (box_t*) ml_malloc (size);

    result_box -> header.size = size / 8;
    result_box -> header.color = START_COLOR;

    return result_box;
}

void rt_globals(int64_t n, ...) {
    va_list globs;
    va_start(globs, n);
#ifdef GC
    add_global_vars_to_gc(n, globs);
#endif
    va_end(globs);
}

int64_t rt_create_tuple(int64_t tuple_len, ...) {
    va_list elements;
    va_start(elements, tuple_len);

    box_t* tuple_box = put_box(sizeof(box_header_t) + tuple_len * 8);
    tuple_box -> header.tag = T_TUPLE;

    for (int i = 0; i < tuple_len; i++)
        tuple_box -> values[i] = va_arg(elements, int64_t);

    va_end(elements);

    return (int64_t)tuple_box;
}

typedef struct {
    int64_t fun;
    int64_t args_num;
    int64_t args_applied;
    int64_t applied_args[];
} closure_t;

int64_t rt_create_closure(int64_t fun, int64_t args_num) {
    box_t* closure_box = put_box(sizeof(box_header_t) + 0x18);
    closure_box->header.tag = T_CLOSURE;

    closure_t* clos = (closure_t*)&closure_box->values;

    clos->args_num = args_num;
    clos->args_applied = 0;
    clos->fun = fun;

    return (int64_t)closure_box;
}

int64_t closure_new(box_t* src_box, int64_t new_args_num, va_list* new_args) {
    box_t* closure_box = put_box((src_box->header.size + new_args_num) * 8);
    closure_box->header.tag = T_CLOSURE;
    closure_t* src_clos = (closure_t*)&src_box->values;
    closure_t* clos = (closure_t*)&closure_box->values;

    clos->fun = src_clos->fun;
    clos->args_applied = src_clos->args_applied + new_args_num;
    clos->args_num = src_clos->args_num;

    for (int i = 0; i < src_clos->args_applied + new_args_num; i++) {
        if (i < src_clos->args_applied)
            clos->applied_args[i] = src_clos->applied_args[i];
        else
            clos->applied_args[i] = va_arg(*new_args, int64_t);
    }

    return (int64_t)closure_box;
}

int64_t call_closure(box_t* closure_box, int64_t new_args_num, va_list* new_args) {
    closure_t* closure = (closure_t*)&closure_box->values;
    size_t args_count = closure->args_num;
    ffi_type* arg_types[args_count];
    int64_t* args[args_count];
    int64_t args_buf[new_args_num];

    for (int i = 0; i < args_count; ++i) {
        arg_types[i] = &ffi_type_sint64;
        if (i < closure->args_applied)
            args[i] = &(closure->applied_args[i]);
        else {
            int na_num = i - closure->args_applied;
            args_buf[na_num] = va_arg(*new_args, int64_t);
            args[i] = &(args_buf[na_num]);
        }
    }

    ffi_cif cif;
    int64_t result = 0;
    if (ffi_prep_cif(&cif, FFI_DEFAULT_ABI, args_count, &ffi_type_sint64, arg_types) == FFI_OK) {
        ffi_call(&cif, (void (*)())closure->fun, &result, (void**)args);
    } else {
        DEBUG_EX("Impossible to call closures\n");
    }

    return result;
}

int64_t args_application(box_t* closure_box, int64_t new_args_num, va_list* new_args) {
    closure_t* closure = (closure_t*)&closure_box->values;
    int64_t num_to_apply = closure->args_num - closure->args_applied;
    int64_t result;
    if (num_to_apply <= new_args_num) {
        int64_t call_result = call_closure(closure_box, num_to_apply, new_args);
        new_args_num -= num_to_apply;
        if (new_args_num == 0)
            result = call_result;
        else {
            result = args_application((box_t*)call_result, new_args_num, new_args);
        }
    } else {
        result = closure_new(closure_box, new_args_num, new_args);
    }

    return result;
}

int64_t rt_application(int64_t closure_box, int64_t new_args_num, ...) {
    va_list new_args;
    va_start(new_args, new_args_num);
    int64_t result = args_application((box_t*)closure_box, new_args_num, &new_args);
    va_end(new_args);

    return result;
}

int64_t rt_get_field(int64_t box, int64_t field_num) {
    field_num = CONVERT_INT_ML_TO_NATIVE(field_num);
    if (!is_r2_ptr(box)) {
        return CONVERT_INT_NATIVE_TO_ML(0);
    } else {
        int64_t result = ((box_t*)box)->values[field_num];
        return result;
    }
}
