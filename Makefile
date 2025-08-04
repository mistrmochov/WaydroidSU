APP_NAME = wsu
BUILD_DIR = target/release
BIN = $(BUILD_DIR)/$(APP_NAME)
INSTALL_BIN = /usr/bin

SRC = $(shell find src -name '*.rs')
RESOURCES = \
	src/res/bootanim_magisk_new.rc \
	src/res/bootanim_magisk.rc \
	src/res/bootanim.rc \
	src/res/loadpolicy.sh

all: $(BIN)

$(BIN): $(SRC) $(RESOURCES) Cargo.toml Cargo.lock
	cargo build --release

install: all
	@echo "Installing binary..."
	install -Dm755 $(BIN) $(INSTALL_BIN)/$(APP_NAME)

uninstall:
	@echo "Uninstalling..."
	rm -f $(INSTALL_BIN)/$(APP_NAME)

clean:
	cargo clean

.PHONY: all install uninstall clean
