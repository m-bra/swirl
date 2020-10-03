target=swirl.0.2.0

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

.PHONY: global_link
global_link:
	test $(base)
	ln -sf $(prj)/bin/$(target) $(base)/bin/$(target)
	ln -sf $(prj)/bin/$(target) $(base)/bin/swirl
	ln -sf $(prj)/lib/swirl $(base)/lib/swirl

.PHONY: global_unlink
global_unlink:
	echo "unimplemented"