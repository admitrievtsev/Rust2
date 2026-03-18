.PHONY: all clean

all: libffi.so include

libffi.so include: ../libffi-3.5.2.tar.gz
	tar --strip-components=1 -xzf ../libffi-3.5.2.tar.gz --one-top-level=libffi-release
	cd libffi-release && \
	./configure --host=riscv64-linux-gnu --target=riscv64-linux-gnu --prefix=$$PWD/../ > /dev/null && \
	make install > /dev/null
	cp lib/libffi.so libffi.so

clean:
	rm -rf libffi-release libffi.so include
