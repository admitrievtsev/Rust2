; ModuleID = 'Rust2'
source_filename = "Rust2"
target triple = "riscv64-unknown-linux-gnu"

@pneq_r2_global_llvm = global i64 0
@"&&_global_llvm" = global i64 0
@mul_r2_global_llvm = global i64 0
@plus_r2_global_llvm = global i64 0
@sub_r2_global_llvm = global i64 0
@div_r2_global_llvm = global i64 0
@l_r2_global_llvm = global i64 0
@le_r2_global_llvm = global i64 0
@neq_r2_global_llvm = global i64 0
@eq_r2_global_llvm = global i64 0
@peq_r2_global_llvm = global i64 0
@g_r2_global_llvm = global i64 0
@ge_r2_global_llvm = global i64 0
@"||_global_llvm" = global i64 0
@rt_application_global_llvm = global i64 0
@rt_create_closure_global_llvm = global i64 0
@rt_create_tuple_global_llvm = global i64 0
@rt_globals_global_llvm = global i64 0
@rt_get_field_global_llvm = global i64 0
@print_int_global_llvm = global i64 0
@sum_tc_global_llvm = global i64 0
@print_f_global_llvm = global i64 0
@main_global_llvm = global i64 0

declare i64 @pneq_r2(i64, i64)

declare i64 @land_r2(i64, i64)

declare i64 @mul_r2(i64, i64)

declare i64 @plus_r2(i64, i64)

declare i64 @sub_r2(i64, i64)

declare i64 @div_r2(i64, i64)

declare i64 @l_r2(i64, i64)

declare i64 @le_r2(i64, i64)

declare i64 @neq_r2(i64, i64)

declare i64 @eq_r2(i64, i64)

declare i64 @peq_r2(i64, i64)

declare i64 @g_r2(i64, i64)

declare i64 @ge_r2(i64, i64)

declare i64 @lor_r2(i64, i64)

declare i64 @rt_application(i64, i64, ...)

declare i64 @rt_create_closure(i64)

declare i64 @rt_create_tuple(i64, ...)

declare i64 @rt_globals(i64, ...)

declare i64 @rt_get_field(i64, i64)

declare i64 @print_int(i64)

define i64 @init_llvm() {
entry:
  %0 = tail call i64 (i64, ...) @rt_globals(i64 20, i64 ptrtoint (ptr @pneq_r2_global_llvm to i64), i64 ptrtoint (ptr @"&&_global_llvm" to i64), i64 ptrtoint (ptr @mul_r2_global_llvm to i64), i64 ptrtoint (ptr @plus_r2_global_llvm to i64), i64 ptrtoint (ptr @sub_r2_global_llvm to i64), i64 ptrtoint (ptr @div_r2_global_llvm to i64), i64 ptrtoint (ptr @l_r2_global_llvm to i64), i64 ptrtoint (ptr @le_r2_global_llvm to i64), i64 ptrtoint (ptr @neq_r2_global_llvm to i64), i64 ptrtoint (ptr @eq_r2_global_llvm to i64), i64 ptrtoint (ptr @peq_r2_global_llvm to i64), i64 ptrtoint (ptr @g_r2_global_llvm to i64), i64 ptrtoint (ptr @ge_r2_global_llvm to i64), i64 ptrtoint (ptr @"||_global_llvm" to i64), i64 ptrtoint (ptr @rt_application_global_llvm to i64), i64 ptrtoint (ptr @rt_create_closure_global_llvm to i64), i64 ptrtoint (ptr @rt_create_tuple_global_llvm to i64), i64 ptrtoint (ptr @rt_globals_global_llvm to i64), i64 ptrtoint (ptr @rt_get_field_global_llvm to i64), i64 ptrtoint (ptr @print_int_global_llvm to i64))
  %1 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @pneq_r2 to i64), i64 2)
  store i64 %1, ptr @pneq_r2_global_llvm, align 4
  %2 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @land_r2 to i64), i64 2)
  store i64 %2, ptr @"&&_global_llvm", align 4
  %3 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @mul_r2 to i64), i64 2)
  store i64 %3, ptr @mul_r2_global_llvm, align 4
  %4 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @plus_r2 to i64), i64 2)
  store i64 %4, ptr @plus_r2_global_llvm, align 4
  %5 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @sub_r2 to i64), i64 2)
  store i64 %5, ptr @sub_r2_global_llvm, align 4
  %6 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @div_r2 to i64), i64 2)
  store i64 %6, ptr @div_r2_global_llvm, align 4
  %7 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @l_r2 to i64), i64 2)
  store i64 %7, ptr @l_r2_global_llvm, align 4
  %8 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @le_r2 to i64), i64 2)
  store i64 %8, ptr @le_r2_global_llvm, align 4
  %9 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @neq_r2 to i64), i64 2)
  store i64 %9, ptr @neq_r2_global_llvm, align 4
  %10 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @eq_r2 to i64), i64 2)
  store i64 %10, ptr @eq_r2_global_llvm, align 4
  %11 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @peq_r2 to i64), i64 2)
  store i64 %11, ptr @peq_r2_global_llvm, align 4
  %12 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @g_r2 to i64), i64 2)
  store i64 %12, ptr @g_r2_global_llvm, align 4
  %13 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @ge_r2 to i64), i64 2)
  store i64 %13, ptr @ge_r2_global_llvm, align 4
  %14 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @lor_r2 to i64), i64 2)
  store i64 %14, ptr @"||_global_llvm", align 4
  %15 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @rt_application to i64), i64 2)
  store i64 %15, ptr @rt_application_global_llvm, align 4
  %16 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @rt_create_closure to i64), i64 1)
  store i64 %16, ptr @rt_create_closure_global_llvm, align 4
  %17 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @rt_create_tuple to i64), i64 1)
  store i64 %17, ptr @rt_create_tuple_global_llvm, align 4
  %18 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @rt_globals to i64), i64 1)
  store i64 %18, ptr @rt_globals_global_llvm, align 4
  %19 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @rt_get_field to i64), i64 2)
  store i64 %19, ptr @rt_get_field_global_llvm, align 4
  %20 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @print_int to i64), i64 1)
  store i64 %20, ptr @print_int_global_llvm, align 4
  ret i64 0
}

define i64 @main() {
entry:
  %0 = tail call i64 @init_llvm()
  %1 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @sum_tc to i64), i64 2)
  store i64 %1, ptr @sum_tc_global_llvm, align 4
  %2 = tail call i64 @rt_create_closure(i64 ptrtoint (ptr @print_f to i64), i64 1)
  store i64 %2, ptr @print_f_global_llvm, align 4
  %main_global_llvm = tail call i64 @main.1()
  store i64 %main_global_llvm, ptr @main_global_llvm, align 4
  ret i64 0
}

define i64 @sum_tc(i64 %0, i64 %1) {
entry:
  br label %tailrecurse

tailrecurse:                                      ; preds = %6, %entry
  %.tr = phi i64 [ %0, %entry ], [ %7, %6 ]
  %.tr1 = phi i64 [ %1, %entry ], [ %8, %6 ]
  %2 = tail call i64 @peq_r2(i64 %.tr, i64 1)
  %3 = ashr i64 %2, 1
  %4 = trunc i64 %3 to i1
  br i1 %4, label %5, label %6

continue:                                         ; preds = %5
  ret i64 %.tr1

5:                                                ; preds = %tailrecurse
  br label %continue

6:                                                ; preds = %tailrecurse
  %7 = tail call i64 @sub_r2(i64 %.tr, i64 3)
  %8 = tail call i64 @plus_r2(i64 %.tr1, i64 3)
  br label %tailrecurse
}

define i64 @print_f(i64 %0) {
entry:
  %1 = tail call i64 @sum_tc(i64 %0, i64 1)
  %2 = tail call i64 @print_int(i64 %1)
  ret i64 %2
}

define i64 @main.1() {
entry:
  %0 = tail call i64 @print_f(i64 349201)
  ret i64 %0
}
