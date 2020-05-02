target=swirl.0.0.2

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
	rm -rf .cache

.PHONY: install
install:
	cp -r bin/ $(INSTALL_DIR)
	cp -r lib/ $(INSTALL_DIR)