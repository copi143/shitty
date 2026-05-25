SHITTY_LIB_DIR ?= .

all:
	@echo "Building `C + X11` example..."
	clang main.c -o main -lX11 -lpthread -lm -L$(SHITTY_LIB_DIR) -lshitty

clean:
	rm main
