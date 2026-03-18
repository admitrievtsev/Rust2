#include "collector.h"
#include <cstdint>
#include <set>

#ifdef GC
    #define START_POOL_SIZE 0x20000

#define START_COLOR 0

void __attribute__((naked)) save_reg() {
    __asm__ volatile("addi sp, sp, -96\n"
                     "sd s0, 0(sp)\n"
                     "sd s1, 8(sp)\n"
                     "sd s2, 16(sp)\n"
                     "sd s3, 24(sp)\n"
                     "sd s4, 32(sp)\n"
                     "sd s5, 40(sp)\n"
                     "sd s6, 48(sp)\n"
                     "sd s7, 56(sp)\n"
                     "sd s8, 64(sp)\n"
                     "sd s9, 72(sp)\n"
                     "sd s10, 80(sp)\n"
                     "sd s11, 88(sp)\n"
                     "ret");
}
void __attribute__((naked)) restore_reg() {
    __asm__ volatile("ld s0, 0(sp)\n"
                     "ld s1, 8(sp)\n"
                     "ld s2, 16(sp)\n"
                     "ld s3, 24(sp)\n"
                     "ld s4, 32(sp)\n"
                     "ld s5, 40(sp)\n"
                     "ld s6, 48(sp)\n"
                     "ld s7, 56(sp)\n"
                     "ld s8, 64(sp)\n"
                     "ld s9, 72(sp)\n"
                     "ld s10, 80(sp)\n"
                     "ld s11, 88(sp)\n"
                     "addi sp, sp, 96\n"
                     "ret");
}
int64_t __attribute__((naked)) get_stack_pointer() {
    __asm__ volatile("mv a0, sp\n"
                     "ret");
}

box_t** stack_bottom = NULL;
box_t** stack_top;

box_t*** global_vars = NULL;
box_t*** global_vars_end = NULL;

typedef struct _pool_st {
    uint8_t* pool_bottom;
    uint8_t* pool_pointer;
    uint8_t* pool_top;
    std::set<box_t*> used_addresses;
} pool_t;

pool_t* the_pool;

    #define POOL_MALLOC(pool, sz)                                                                                      \
        ({                                                                                                             \
            void* res = pool->pool_pointer;                                                                            \
            pool->pool_pointer += sz;                                                                                  \
            pool->used_addresses.insert((box_t*)res);                                                                  \
            res;                                                                                                       \
        })

pool_t* create_pool_t(size_t sz) {
    uint8_t* start = (uint8_t*)malloc(sz);
    pool_t* pool = (pool_t*)malloc(sizeof(*pool));

    pool->pool_pointer = start;
    pool->pool_top = start + sz;
    pool->pool_bottom = start;
    pool->used_addresses = std::set<box_t*>();

    return pool;
}

void free_pool_t(pool_t* pool) {
    free(pool->pool_bottom);
    free(pool);
}

uint8_t is_inside_pool(pool_t* pool, box_t* some_box) {
    if (is_ml_ptr((int64_t)some_box))
        if ((pool->pool_bottom <= (void*)some_box) && ((void*)some_box < pool->pool_top))
            return 1;

    return 0;
}

box_t* process_node(box_t* old_box, pool_t* old_pool) {
    if (!is_inside_pool(old_pool, old_box)) {
        return old_box;
    } else {

        uint64_t orig_address = (uint64_t)old_box;
        old_box = *(--(old_pool->used_addresses.upper_bound(old_box)));
        uint64_t offset = orig_address - (uint64_t)old_box;

        if (old_box->header.color == COLOR_PROCESSED) {
            return (box_t*)old_box->values[0] + offset;
        } else {

            box_t* res = (box_t*)POOL_MALLOC(the_pool, old_box->header.size * 8);
            res->header = old_box->header;

            int64_t fst_value_buf = old_box->values[0];

            for (int i = 0; i < old_box->header.size - 1; i++) {
                if (i == 0) {
                    old_box->header.color = COLOR_PROCESSED;
                    old_box->values[0] = (int64_t)res;
                    res->values[i] = (int64_t)process_node((box_t*)fst_value_buf, old_pool);
                } else {
                    res->values[i] = (int64_t)process_node((box_t*)old_box->values[i], old_pool);
                }
            }
            return (box_t*)((uint64_t)res + offset);
        }
    }
}

void realloc_the_pool(size_t sz) {
    save_reg();
    stack_top = (box_t**)get_stack_pointer();
    pool_t* old_pool = the_pool;
    the_pool = create_pool_t(sz);

    for (box_t** iter = stack_top; iter < stack_bottom; iter++) {
        *iter = process_node(*iter, old_pool);
    }
    for (box_t*** iter = global_vars; iter < global_vars_end; iter++) {
        **iter = process_node(**iter, old_pool);
    }
    free_pool_t(old_pool);
    restore_reg();
}

void* gc_malloc(size_t sz) {
    if ((the_pool->pool_pointer + sz) <= the_pool->pool_top) {
        return POOL_MALLOC(the_pool, sz);
    } else {
        size_t cur_sz = (size_t)(the_pool->pool_top - the_pool->pool_bottom);
        size_t needed_sz = sz + (size_t)(the_pool->pool_pointer - the_pool->pool_bottom);
        while (cur_sz < needed_sz) {
            cur_sz <<= 1;
        }
        realloc_the_pool(cur_sz);
        return POOL_MALLOC(the_pool, sz);
    }
}

void gc_on_load() {
    stack_bottom = (box_t**)get_stack_pointer();
    the_pool = create_pool_t(START_POOL_SIZE);
}

void add_global_vars_to_gc(int64_t n, va_list globs) {
    global_vars = (box_t***)calloc(n, sizeof(*global_vars));
    global_vars_end = global_vars + n;
    for (int i = 0; i < n; i++) {
        global_vars[i] = (box_t**)va_arg(globs, void*);
    }
}

void compact() {
    size_t sz = the_pool->pool_top - the_pool->pool_bottom;
    realloc_the_pool(sz);
}

void print_gc_info() {
    size_t used = the_pool->pool_pointer - the_pool->pool_bottom;
    size_t all = the_pool->pool_top - the_pool->pool_bottom;
    printf("Used: %#8lx/%#8lx\n", used, all);
    printf("Bottom: %#8lx | Top: %#8lx |  Pointer: %#8lx\n", (int64_t)the_pool->pool_bottom,
                      (int64_t)the_pool->pool_top, (int64_t)the_pool->pool_pointer);
    printf("Stack bot: %lx | cur: %lx\n", stack_bottom, get_stack_pointer());
}

#endif