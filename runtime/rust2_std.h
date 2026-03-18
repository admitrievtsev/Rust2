#include <stdint.h>

#include "rust2_runtime.h"

#define INT_NT(x) ((x) >> 1)
#define NT_INT(x) (((x) << 1) + 1)

int64_t print_int(int64_t a);
int64_t eq_r2(int64_t x, int64_t y);
int64_t neq_r2(int64_t x, int64_t y);
int64_t peq_r2(int64_t x, int64_t y);
int64_t pneq_r2(int64_t x, int64_t y);
int64_t l_r2(int64_t x, int64_t y);
int64_t le_r2(int64_t x, int64_t y);
int64_t g_r2(int64_t x, int64_t y);
int64_t ge_r2(int64_t x, int64_t y);
int64_t lor_r2(int64_t x, int64_t y);
int64_t land_r2(int64_t x, int64_t y);
int64_t plus_r2(int64_t x, int64_t y);
int64_t sub_r2(int64_t x, int64_t y);
int64_t mul_r2(int64_t x, int64_t y);
int64_t div_r2(int64_t x, int64_t y);