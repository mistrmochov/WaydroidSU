APP_NAME = wsu
BUILD_DIR = target/release
BIN = $(BUILD_DIR)/$(APP_NAME)
INSTALL_BIN = /usr/bin

SRC = $(shell find src -name '*.rs')

all: $(BIN)

$(BIN): $(SRC) Cargo.toml Cargo.lock
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
