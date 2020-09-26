target=swirl.0.0.3

prj:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

all: bin/$(target)

bin/$(target):
	mkdir -p bin
	cargo build --release
	mv target/release/swirl bin/$(target)

.PHONY: run
run: bin/$(target)
	bin/$(target)

.PHONY: clean
clean:
	cargo clean
	rm -rf bin

.PHONY: install
install:
	test $(base)
	ln -sf $(prj)/bin/swirl.0.0.3 $(base)/bin/swirl.0.0.3
	ln -sf $(prj)/bin/swirl.0.0.3 $(base)/bin/swirl
	ln -sf $(prj)/lib/swirl $(base)/lib/swirl

.PHONY: uninstall
uninstall:
	echo "unimplemented"