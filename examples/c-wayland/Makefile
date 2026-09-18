SHITTY_LIB_DIR ?= .

all:
	@echo "Building \`C + Wayland\` example..."
	clang main.c -o main -lwayland-client -L$(SHITTY_LIB_DIR) -lshitty

clean:
	rm main
