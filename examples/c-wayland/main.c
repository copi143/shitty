#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <wayland-client.h>

#include "../../src/ffi/shitty.h"

void shitty_callback_panic(shitty_str info) {
  fprintf(stderr, "Panic: %s\n", info);
  exit(1);
}

void *shitty_callback_alloc(shitty_usize size) { return malloc(size); }

void shitty_callback_free(void *ptr) { free(ptr); }

void *shitty_callback_aligned_alloc(shitty_usize size, shitty_usize align) {
  void *ptr = NULL;
  if (posix_memalign(&ptr, align, size) != 0) {
    return NULL;
  }
  return ptr;
}

void *shitty_callback_realloc(void *ptr, shitty_usize size) {
  return realloc(ptr, size);
}

int main(int argc, char **argv) {
  struct wl_display *display = wl_display_connect(NULL);
  if (!display) {
    fprintf(stderr, "无法连接到 Wayland 显示服务器\n");
    return 1;
  }

  shitty_fontrenderer font = shitty_unifont_new();
  if (!font) {
    fprintf(stderr, "无法创建字体实例\n");
    wl_display_disconnect(display);
    return 1;
  }

  shitty_terminal term = shitty_new(1280, 720, SHITTY_BUFMODE_DOUBLE, font);
  if (!term) {
    fprintf(stderr, "无法创建终端实例\n");
    wl_display_disconnect(display);
    return 1;
  }

  while (wl_display_dispatch(display) != -1) {
  }

  shitty_del(term);
  wl_display_disconnect(display);
  return 0;
}
