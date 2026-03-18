#!/bin/sh

usage() {
    echo "Usage: $0 <source_file>"
    exit 1
}

if [ "$#" -ne 1 ]; then
    usage
fi

SOURCE_FILE=$1
OUTPUT_FILE="output.elf"


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
cp --update=none libmlrt.so libmlrt.so.8
cp --update=none libmlstd.so libmlstd.so.8
cp --update=none libffi.so libffi.so.8
cd ../..

if [ ! -f "$SOURCE_FILE" ]; then
    echo "Error: Source file '$SOURCE_FILE' not found!"
    exit 1
fi

clang++-16                                  \
   --target=riscv64-linux-gnu               \
   -fPIC                                    \
   -L../Rust2/runtime/target_make           \
   -lr2std                                  \
   -lr2rt                                   \
   -lffi                                    \
   -o "$OUTPUT_FILE"                        \
   "$SOURCE_FILE"

if [ $? -eq 0 ]; then
    true
else
    echo "Compilation failed!"
    exit 1
fi


qemu-riscv64 -L /usr/riscv64-linux-gnu/ -E LD_LIBRARY_PATH=../Rust2/runtime/target_make "$OUTPUT_FILE"

cd runtime/libffi
make clean >/dev/null 2>&1

cd ../..
cd runtime
make clean >/dev/null 2>&1
cd ..
cd runtime/target_make
make clean >/dev/null 2>&1
rm -f libr2rt.so.8
rm -f lr2std.so.8
rm -f libffi.so.8
cd ../..
