#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "Starting peak stack size benchmark for tail call optimization..."

TEMP_DIR=$(mktemp -d)
echo "Using temporary directory: $TEMP_DIR"

cp compiler/src/tests/sum_tail.ml "$TEMP_DIR/"

cd "$(dirname "$0")"

declare -a tce_measurements
declare -a simple_measurements

measure_peak_stack_size() {
    local elf_file="$1"
    local test_name="$2"

    echo -e "${BLUE}Measuring peak stack size for $test_name...${NC}"

    START_TIME=$(date +%s%N)
    PEAK_RSS=0

    qemu-riscv64 -L /usr/riscv64-linux-gnu/ -E LD_LIBRARY_PATH=../Rust2/runtime/riscv "$elf_file" 2>/dev/null &
    QEMU_PID=$!
    while kill -0 $QEMU_PID 2>/dev/null; do
      sleep 0.000001
        if ps -o pid,rss,vsz,comm -C qemu-riscv64 --no-headers 2>/dev/null | grep -q "$QEMU_PID"; then
            RSS=$(ps -o rss --no-headers -p $QEMU_PID 2>/dev/null | tr -d ' ')
            if [[ -n "$RSS" && "$RSS" =~ ^[0-9]+$ && "$RSS" -gt "$PEAK_RSS" ]]; then
                PEAK_RSS=$RSS
            fi
        fi
    done

    wait $QEMU_PID

    END_TIME=$(date +%s%N)
    EXECUTION_TIME=$(( (END_TIME - START_TIME) / 1000000 ))

    PEAK_MB=$((PEAK_RSS / 1024))

    if [[ "$test_name" == *"TCE"* ]]; then
        tce_measurements+=($PEAK_MB)
    else
        simple_measurements+=($PEAK_MB)
    fi


    echo -e "${GREEN}$test_name peak memory usage: ${PEAK_MB} MB${NC}"
    echo -e "${GREEN}$test_name execution time: ${EXECUTION_TIME} ms${NC}"

    echo ""
}

echo -e "${YELLOW}Compiling with tail call optimization...${NC}"
cargo run --bin compiler -- -s "$TEMP_DIR/sum_tail.ml" -o "$TEMP_DIR/temp_with_tail.ll" -t

echo -e "${YELLOW}Compiling without tail call optimization...${NC}"
cargo run --bin compiler -- -s "$TEMP_DIR/sum_tail.ml" -o "$TEMP_DIR/temp_without_tail.ll"

echo -e "${YELLOW}Generating ELF files...${NC}"

clang++-15 \
   --target=riscv64-linux-gnu \
   -O0 \
   -fPIC \
   -L../Rust2/runtime/riscv \
   -lmlstd \
   -lmlrt \
   -lffi \
   -o "$TEMP_DIR/output_with_tail.elf" \
   "$TEMP_DIR/temp_with_tail.ll"

clang++-15 \
   --target=riscv64-linux-gnu \
   -O0 \
   -fPIC \
   -L../Rust2/runtime/riscv \
   -lmlstd \
   -lmlrt \
   -lffi \
   -o "$TEMP_DIR/output_without_tail.elf" \
   "$TEMP_DIR/temp_without_tail.ll"

measure_peak_stack_size "$TEMP_DIR/output_with_tail.elf" "TCE Run 1"
measure_peak_stack_size "$TEMP_DIR/output_with_tail.elf" "TCE Run 2"
measure_peak_stack_size "$TEMP_DIR/output_with_tail.elf" "TCE Run 3"
measure_peak_stack_size "$TEMP_DIR/output_with_tail.elf" "TCE Run 4"
measure_peak_stack_size "$TEMP_DIR/output_with_tail.elf" "TCE Run 5"

measure_peak_stack_size "$TEMP_DIR/output_without_tail.elf" "Simple Run 1"
measure_peak_stack_size "$TEMP_DIR/output_without_tail.elf" "Simple Run 2"
measure_peak_stack_size "$TEMP_DIR/output_without_tail.elf" "Simple Run 3"
measure_peak_stack_size "$TEMP_DIR/output_without_tail.elf" "Simple Run 4"
measure_peak_stack_size "$TEMP_DIR/output_without_tail.elf" "Simple Run 5"

if [ ${#tce_measurements[@]} -gt 0 ]; then
    tce_sum=0
    for val in "${tce_measurements[@]}"; do
        tce_sum=$((tce_sum + val))
    done
    tce_avg=$((tce_sum / ${#tce_measurements[@]}))
    echo -e "${GREEN}Average peak memory usage with TCE: ${tce_avg} MB${NC}"
fi

if [ ${#simple_measurements[@]} -gt 0 ]; then
    simple_sum=0
    for val in "${simple_measurements[@]}"; do
        simple_sum=$((simple_sum + val))
    done
    simple_avg=$((simple_sum / ${#simple_measurements[@]}))
    echo -e "${GREEN}Average peak memory usage without TCE: ${simple_avg} MB${NC}"
fi

rm -rf "$TEMP_DIR"

echo -e "${GREEN}Peak stack size benchmark completed!${NC}"