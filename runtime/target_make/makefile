CC := riscv64-linux-gnu-gcc
CXX := riscv64-linux-gnu-g++
CFLAGS := -fPIC -Wall -Wextra -I../libffi/riscv64/include
CXXFLAGS := -fPIC -Wall -Wextra -std=c++11 -I../libffi/riscv64/include
LDFLAGS := -L../libffi/riscv64

.PHONY: all clean

all: libr2rt.so libr2std.so libffi.so libffi_include

libr2rt.so: ../rust2_runtime.c ../rust2_runtime.h ../collector.h ../collector.cpp
	$(CXX) $(CXXFLAGS) $(LDFLAGS) -shared -o $@ ../rust2_runtime.c ../collector.cpp -lffi

libr2std.so: ../rust2_std.c ../rust2_std.h ../rust2_runtime.h libr2rt.so
	$(CC) $(CFLAGS) $(LDFLAGS) -shared -L./ -lr2rt -o $@ ../rust2_std.c

libffi.so:
	cp ../libffi/riscv64/libffi.so .

libffi_include:
	cp -r ../libffi/riscv64/include ./libffi_include

clean:
	rm -f libr2rt.so libr2std.so libffi.so
	rm -rf libffi_include