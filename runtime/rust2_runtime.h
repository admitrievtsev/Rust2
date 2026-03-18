#pragma once

#include "collector.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif



typedef enum {
    T_TUPLE = 0,
    T_CLOSURE = 52,
    T_DRAINED = 67,
} tag_t;

#pragma pack(push, 1)
typedef struct {
    int64_t size : 54;
    uint8_t color : 2;
    tag_t tag : 8;
} box_header_t;

typedef struct {
    box_header_t header;
    int64_t values[];
} box_t;
#pragma pack(pop)

#define CONVERT_INT_ML_TO_NATIVE(x) ((x) >> 1)
#define CONVERT_INT_NATIVE_TO_ML(x) (((x) << 1) + 1)

int8_t is_ml_ptr(int64_t arg);
tag_t get_tag(int64_t obj);

int64_t rt_create_tuple(int64_t tuple_size, ...);
int64_t rt_create_closure(int64_t fun, int64_t args_num);
int64_t rt_application(int64_t closure_box, int64_t new_args_num, ...);
int64_t rt_get_field(int64_t box, int64_t field_num);

void rt_globals(int64_t n, ...);

#ifdef __cplusplus
}
#endif