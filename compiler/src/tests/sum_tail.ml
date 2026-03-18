let rec sum_tc n acc =
        if (n == 0) then acc else sum_tc (n - 1) (acc + 1)

let print_f n = print_int (sum_tc n 0)

let main = print_f 174600