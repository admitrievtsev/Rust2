let rec fib_acc a b n =
  if n = 1 then b
  else fib_acc b (a + b) (n - 1)

let rec fib n =
  if n < 2 then n else fib (n - 1) + fib (n - 2)

let main =
  let _ = print_int (((fib_acc 0) 1) 6) in
  0