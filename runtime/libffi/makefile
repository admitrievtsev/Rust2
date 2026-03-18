.PHONY: all clean

all: libffi-3.5.2.tar.gz libffi-release

libffi-3.5.2.tar.gz:
	curl -L https://github.com/libffi/libffi/releases/download/v3.5.2/libffi-3.5.2.tar.gz -o $@

libffi-release: libffi-3.5.2.tar.gz
	tar --strip-components=1 -xzf libffi-3.5.2.tar.gz --one-top-level=libffi-release

clean:
	rm -rf libffi-3.5.2.tar.gz libffi-release