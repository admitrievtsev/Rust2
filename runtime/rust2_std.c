#include "rust2_std.h"

int64_t print_int(int64_t a) {
    printf("%ld\n", INT_NT(a));
    fflush(stdout);
    return NT_INT(0);
}

int8_t is_r2_ptr(int64_t arg) { return !(arg & 1); };

int64_t plus_r2(int64_t x, int64_t y) {
    return x + y - 1;
}
int64_t sub_r2(int64_t x, int64_t y) {
    return x - y + 1;
}
int64_t mul_r2(int64_t x, int64_t y) {
    return (x >> 1) * (y - 1) + 1;
}
int64_t div_r2(int64_t x, int64_t y) {
    return NT_INT(INT_NT(x) / INT_NT(y));
}

int8_t compare_r2(int64_t x, int64_t y);

int8_t cmp(int64_t x, int64_t y) {
    box_t* box_x = (box_t*)x;
    box_t* box_y = (box_t*)y;

    int64_t res = 0;
    for (int i = 1; i < box_x->header.size; i++) {
        int val_num = i - 1;
        res = compare_r2(box_x->values[val_num], box_y->values[val_num]);
        if (res != 0)
            break;
        }
    return res;
}

tag_t get_tag(int64_t obj) {
    if (is_r2_ptr(obj)) {
        return ((box_t*)obj)->header.tag;
    } else {
        return T_DRAINED;
    }
}

int8_t compare(int64_t x, int64_t y) {
    if (x < y) {
        return -1;
    } else if (x > y) {
        return 1;
    } else {
        return 0;
    }
}

int8_t compare_r2(int64_t x, int64_t y) {
    tag_t x_tag = get_tag(x);
    tag_t y_tag = get_tag(y);
    int8_t tag_comp = compare(x_tag, y_tag);

    if (tag_comp == 0) {
        if (x_tag == T_DRAINED) {
            x = INT_NT(x);
            y = INT_NT(y);
            
            return compare(x, y);
        } else {
            return cmp(x, y);
        }
    } else {
        return tag_comp;
    }
}

int64_t eq_r2(int64_t x, int64_t y) { return NT_INT(compare_r2(x, y) == 0); }
int64_t neq_r2(int64_t x, int64_t y) { return NT_INT(compare_r2(x, y) != 0); }
int64_t peq_r2(int64_t x, int64_t y) { return NT_INT((x == y)); }
int64_t pneq_r2(int64_t x, int64_t y) { return NT_INT((x != y)); }
int64_t g_r2(int64_t x, int64_t y) { return NT_INT(compare_r2(x, y) == 1); }
int64_t ge_r2(int64_t x, int64_t y) { return NT_INT(compare_r2(x, y) >= 0); }
int64_t l_r2(int64_t x, int64_t y) { return NT_INT(compare_r2(x, y) == -1); }
int64_t le_r2(int64_t x, int64_t y) { return NT_INT(compare_r2(x, y) <= 0); }
int64_t lor_r2(int64_t x, int64_t y) { return x | y; };
int64_t land_r2(int64_t x, int64_t y) { return x & y; };