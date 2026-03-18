# Rust2

## Blazingly fast Rust2 language compiler written in Rust

### This compiler can:
* compile Rust2 language construnctions
* compile CPS-based functions
* make ANF converfion
* make Lambda lifting
* make Closure Conversion
* support user's infix operators
* apply tail-call optimizations

### Run
To run this compiler you should provide following arguments:
* `--source %path_to_file` path of source file to be compiled
* `--out` %filename name of emited ll file
* (optional) `-t` enables tail call optimization
After generating .ll file you can execue it with ./llvm.sh %filename

### Optimization Benches
Tail call optimization benches results are stored in `tail_results.txt`
You can manually benchmark tail call optimizations with `./benchmark.sh`
Tail call optimizations reduce average execution time up to 25% and memory usage up to 50%
