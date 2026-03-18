CC := riscv64-linux-gnu-gcc
CXX := riscv64-linux-gnu-g++
CFLAGS := -fPIC -Wall -Wextra -I./libffi/riscv64/include
LDFLAGS := -L./libffi/riscv64

.PHONY: all clean

all: libr2rt.so libr2std.so

libr2rt.so: rust2_runtime.c rust2_runtime.h collector.h collector.cpp
	$(CC) $(CFLAGS) $(LDFLAGS) -shared -o $@ $< -lffi

libr2std.so: rust2_std.c rust2_std.h rust2_runtime.h libr2rt.so
	$(CC) $(CFLAGS) $(LDFLAGS) -shared -o $@ $< -L. -lr2rt

clean:
	rm -f libr2rt.so libr2std.so

libr2rt.o: rust2_runtime.c rust2_runtime.h collector.h
	$(CC) $(CFLAGS) -c -o $@ $<

libr2std.o: rust2_std.c rust2_std.h rust2_runtime.h
	$(CC) $(CFLAGS) -c -o $@ $<