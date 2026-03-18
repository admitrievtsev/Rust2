let rec fact_cps n cont = if (n = 0) then
  cont 1
else
  fact_cps (n - 1) (fun acc -> cont (n * acc))

let main = (fact_cps 5) print_int