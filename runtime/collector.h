#pragma once
#include "rust2_runtime.h"
#include <stdarg.h>
#include <stddef.h>

#ifdef GC

void gc_on_load();

typedef enum { COLOR_UNPROCESSED = 0, COLOR_PROCESSED = 1 } color_t;



void add_global_vars_to_gc(int64_t n, va_list globs);

void compact();

void print_gc_info();

void* gc_malloc(size_t);

#endif