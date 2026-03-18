#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'
TEMP_DIR=$(mktemp -d)
echo -e "${BLUE}Using temporary directory: $TEMP_DIR${NC}"

cd runtime/libffi
make clean && make >/dev/null 2>&1
cd riscv64
make clean && make >/dev/null 2>&1
cd ../../..
cd runtime
make clean && make >/dev/null 2>&1
cd ..
cd runtime/target_make
make clean && make >/dev/null 2>&1
cp --update=none libr2rt.so libr2rt.so.8
cp --update=none libr2std.so libr2std.so.8
cp --update=none libffi.so libffi.so.8
cd ../..

TEST_CASES=(
    "sum_tail.ml:174600"
    "fac_cps.ml:120"
    "fib_cps.ml:8"
    "infix.ml:6"
    "fix_fac.ml:720"
)

ALL_TESTS_PASSED=true

for TEST_CASE in "${TEST_CASES[@]}"; do
    TEST_FILE="${TEST_CASE%:*}"
    EXPECTED_OUTPUT="${TEST_CASE#*:}"

    echo -e "${BLUE}Testing $TEST_FILE with expected output: $EXPECTED_OUTPUT${NC}"
    cp "compiler/src/tests/$TEST_FILE" "$TEMP_DIR/"

    cargo run --bin compiler -- -s "$TEMP_DIR/$TEST_FILE" -o "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.ll" -t >/dev/null 2>&1
    cargo run --bin compiler -- -s "$TEMP_DIR/$TEST_FILE" -o "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.ll" >/dev/null 2>&1


    # Compile with TCE
    clang++-15 \
       --target=riscv64-linux-gnu \
       -O0 \
       -fPIC \
       -L../Rust2/runtime/target_make \
       -lr2std \
       -lr2rt \
       -lffi \
       -o "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.elf" \
       "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.ll" >/dev/null 2>&1

    # Compile without TCE
    clang++-15 \
       --target=riscv64-linux-gnu \
       -O0 \
       -fPIC \
       -L../Rust2/runtime/target_make \
       -lr2std \
       -lr2rt \
       -lffi \
       -o "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.elf" \
       "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.ll" >/dev/null 2>&1

    if [[ -f "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.elf" ]]; then
        OUTPUT_TCE=$(qemu-riscv64 -L /usr/riscv64-linux-gnu/ -E LD_LIBRARY_PATH=../Rust2/runtime/target_make "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.elf" 2>/dev/null)
        echo -e "${GREEN}TCE Output: $OUTPUT_TCE${NC}"

        if [[ "$OUTPUT_TCE" == "$EXPECTED_OUTPUT" ]]; then
            echo -e "${GREEN}TCE execution output matches expected: $EXPECTED_OUTPUT${NC}"
        else
            echo -e "${RED}TCE execution output mismatch. Expected: $EXPECTED_OUTPUT, Got: $OUTPUT_TCE${NC}"
            ALL_TESTS_PASSED=false
        fi
    else
        echo -e "${YELLOW}TCE ELF not available for execution${NC}"
    fi

    # Test non-TCE version
    if [[ -f "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.elf" ]]; then
        OUTPUT_NON_TCE=$(qemu-riscv64 -L /usr/riscv64-linux-gnu/ -E LD_LIBRARY_PATH=../Rust2/runtime/target_make "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.elf" 2>/dev/null)
        echo -e "${GREEN}Non-TCE Output: $OUTPUT_NON_TCE${NC}"

        if [[ "$OUTPUT_NON_TCE" == "$EXPECTED_OUTPUT" ]]; then
            echo -e "${GREEN}Non-TCE execution output matches expected: $EXPECTED_OUTPUT${NC}"
        else
            echo -e "${RED}Non-TCE execution output mismatch. Expected: $EXPECTED_OUTPUT, Got: $OUTPUT_NON_TCE${NC}"
            ALL_TESTS_PASSED=false
        fi
    else
        echo -e "${YELLOW}Non-TCE ELF not available for execution${NC}"
    fi

    TCE_IR=$(cat "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.ll")
    NON_TCE_IR=$(cat "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.ll")

    # Verify basic LLVM IR structure
    if [[ ! "$TCE_IR" =~ "define" ]]; then
        echo -e "${RED}FAILED: TCE IR doesn't contain function definitions for $TEST_FILE${NC}"
        ALL_TESTS_PASSED=false
        continue
    fi

    if [[ ! "$NON_TCE_IR" =~ "define" ]]; then
        echo -e "${RED}FAILED: Non-TCE IR doesn't contain function definitions for $TEST_FILE${NC}"
        ALL_TESTS_PASSED=false
        continue
    fi
    case "$TEST_FILE" in
        "sum_tail.ml")
            if [[ ! "$TCE_IR" =~ "tail call" ]]; then
                echo -e "${RED}FAILED: TCE IR doesn't contain tail call for $TEST_FILE${NC}"
                ALL_TESTS_PASSED=false
                continue#!/bin/bash
                        set -e
                        RED='\033[0;31m'
                        GREEN='\033[0;32m'
                        YELLOW='\033[1;33m'
                        BLUE='\033[0;34m'
                        NC='\033[0m'

                        TEMP_DIR=$(mktemp -d)
                        TEST_CASES=(
                            "sum_tail.ml:174600"
                            "fac_cps.ml:120"
                            "fib_cps.ml:8"
                            "infix.ml:6"
                            "fix_fac.ml:720"
                        )

                        ALL_TESTS_PASSED=true

                        for TEST_CASE in "${TEST_CASES[@]}"; do
                            TEST_FILE="${TEST_CASE%:*}"
                            EXPECTED_OUTPUT="${TEST_CASE#*:}"

                            echo -e "${BLUE}Testing $TEST_FILE with expected output: $EXPECTED_OUTPUT${NC}"

                            cp "compiler/src/tests/$TEST_FILE" "$TEMP_DIR/"

                            cargo run --bin compiler -- -s "$TEMP_DIR/$TEST_FILE" -o "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.ll" -t >/dev/null 2>&1

                            cargo run --bin compiler -- -s "$TEMP_DIR/$TEST_FILE" -o "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.ll" >/dev/null 2>&1

                            clang++-15 \
                               --target=riscv64-linux-gnu \
                               -O0 \
                               -fPIC \
                               -L../Rust2/runtime/target_make \
                               -lr2std \
                               -lr2rt \
                               -lffi \
                               -o "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.elf" \
                               "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.ll" >/dev/null 2>&1

                            clang++-15 \
                               --target=riscv64-linux-gnu \
                               -O0 \
                               -fPIC \
                               -L../Rust2/runtime/target_make \
                               -lr2std \
                               -lr2rt \
                               -lffi \
                               -o "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.elf" \
                               "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.ll" >/dev/null 2>&1

                            if [[ -f "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.elf" ]]; then
                                OUTPUT_TCE=$(qemu-riscv64 -L /usr/riscv64-linux-gnu/ -E LD_LIBRARY_PATH=../Rust2/runtime/target_make "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.elf" 2>/dev/null)
                                echo -e "${GREEN}TCE Output: $OUTPUT_TCE${NC}"

                                if [[ "$OUTPUT_TCE" == "$EXPECTED_OUTPUT" ]]; then
                                    echo -e "${GREEN}TCE execution output matches expected: $EXPECTED_OUTPUT${NC}"
                                else
                                    echo -e "${RED}TCE execution output mismatch. Expected: $EXPECTED_OUTPUT, Got: $OUTPUT_TCE${NC}"
                                    ALL_TESTS_PASSED=false
                                fi
                            else
                                echo -e "${YELLOW}TCE ELF not available for execution${NC}"
                            fi

                            if [[ -f "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.elf" ]]; then
                                echo -e "${BLUE}Executing non-TCE version with QEMU...${NC}"
                                OUTPUT_NON_TCE=$(qemu-riscv64 -L /usr/riscv64-linux-gnu/ -E LD_LIBRARY_PATH=../Rust2/runtime/target_make "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.elf" 2>/dev/null)
                                echo -e "${GREEN}Non-TCE Output: $OUTPUT_NON_TCE${NC}"

                                if [[ "$OUTPUT_NON_TCE" == "$EXPECTED_OUTPUT" ]]; then
                                    echo -e "${GREEN}Non-TCE execution output matches expected: $EXPECTED_OUTPUT${NC}"
                                else
                                    echo -e "${RED}Non-TCE execution output mismatch. Expected: $EXPECTED_OUTPUT, Got: $OUTPUT_NON_TCE${NC}"
                                    ALL_TESTS_PASSED=false
                                fi
                            else
                                echo -e "${YELLOW}Non-TCE ELF not available for execution${NC}"
                            fi

                            TCE_IR=$(cat "$TEMP_DIR/${TEST_FILE%.ml}_with_tce.ll")
                            NON_TCE_IR=$(cat "$TEMP_DIR/${TEST_FILE%.ml}_without_tce.ll")

                            if [[ ! "$TCE_IR" =~ "define" ]]; then
                                echo -e "${RED}FAILED: TCE IR doesn't contain function definitions for $TEST_FILE${NC}"
                                ALL_TESTS_PASSED=false
                                continue
                            fi

                            if [[ ! "$NON_TCE_IR" =~ "define" ]]; then
                                echo -e "${RED}FAILED: Non-TCE IR doesn't contain function definitions for $TEST_FILE${NC}"
                                ALL_TESTS_PASSED=false
                                continue
                            fi
                            case "$TEST_FILE" in
                                "sum_tail.ml")
                                    if [[ ! "$TCE_IR" =~ "tail call" ]]; then
                                        echo -e "${RED}FAILED: TCE IR doesn't contain tail call for $TEST_FILE${NC}"
                                        ALL_TESTS_PASSED=false
                                        continue
                                    fi
                                    echo -e "${GREEN}TCE IR contains expected tail call optimization${NC}"
                                    ;;
                                "fac_cps.ml"|"fib_cps.ml")
                                    if [[ ! "$TCE_IR" =~ "call" ]]; then
                                        echo -e "${RED}FAILED: TCE IR doesn't contain function calls for $TEST_FILE${NC}"
                                        ALL_TESTS_PASSED=false
                                        continue
                                    fi
                                    echo -e "${GREEN}TCE IR contains expected function calls${NC}"
                                    ;;
                                *)
                                    echo -e "${GREEN}Basic structure verified for $TEST_FILE${NC}"
                                    ;;
                            esac

                            echo -e "${GREEN}PASSED: $TEST_FILE compilation and execution verification successful${NC}"
                        done

                        rm -rf "$TEMP_DIR"

                        if [ "$ALL_TESTS_PASSED" = true ]; then
                            echo -e "${GREEN}All integration tests with QEMU execution and output verification passed!${NC}"
                            exit 0
                        else
                            echo -e "${RED}Some integration tests failed!${NC}"
                            exit 1
                        fi
            fi
            echo -e "${GREEN}TCE IR contains expected tail call optimization${NC}"
            ;;
        "fac_cps.ml"|"fib_cps.ml")

    esac

    echo -e "${GREEN}PASSED: $TEST_FILE compilation and execution verification successful${NC}"
done

rm -rf "$TEMP_DIR"

cd runtime
make clean >/dev/null 2>&1
cd ..
cd runtime/target_make
make clean >/dev/null 2>&1
rm -f libr2rt.so.8
rm -f libr2std.so.8
rm -f libffi.so.8
cd ../..

if [ "$ALL_TESTS_PASSED" = true ]; then
    echo -e "${GREEN}All integration tests with QEMU execution and output verification passed!${NC}"
    exit 0
else
    echo -e "${RED}Some integration tests failed!${NC}"
    exit 1
fi