/* Minimal X11 example for shitty
 * - creates an X11 window
 * - allocates a framebuffer compatible with shitty_flush (32-bit Color)
 * - calls shitty_flush in a timer to update the window
 * - forwards simple keyboard input to shitty_input
 */

#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "../../src/ffi/shitty.h"

/* Provide allocation callbacks used by the Rust library */
void shitty_callback_panic(shitty_str info) {
  fprintf(stderr, "Panic: %s\n", info);
  exit(1);
}

void *shitty_callback_alloc(shitty_usize size) { return malloc(size); }
void shitty_callback_free(void *ptr) { free(ptr); }
void *shitty_callback_aligned_alloc(shitty_usize size, shitty_usize align) {
  void *ptr = NULL;
  if (posix_memalign(&ptr, align, size) != 0)
    return NULL;
  return ptr;
}
void *shitty_callback_realloc(void *ptr, shitty_usize size) {
  return realloc(ptr, size);
}
void *shitty_callback_aligned_realloc(void *ptr, shitty_usize size,
                                      shitty_usize align) {
  void *newp = shitty_callback_aligned_alloc(size, align);
  if (!newp)
    return NULL;
  if (ptr) {
    memcpy(newp, ptr, size);
    shitty_callback_free(ptr);
  }
  return newp;
}

int main(int argc, char **argv) {
  Display *dpy = XOpenDisplay(NULL);
  if (!dpy) {
    fprintf(stderr, "Unable to open X display\n");
    return 1;
  }

  int scr = DefaultScreen(dpy);
  Window root = RootWindow(dpy, scr);
  unsigned int width = 800, height = 600;

  Window win = XCreateSimpleWindow(dpy, root, 10, 10, width, height, 1,
                                   BlackPixel(dpy, scr), WhitePixel(dpy, scr));
  XStoreName(dpy, win, "shitty X11 example");
  XSelectInput(dpy, win,
               ExposureMask | KeyPressMask | KeyReleaseMask |
                   StructureNotifyMask);
  XMapWindow(dpy, win);

  /* Create graphics context and image buffer */
  GC gc = DefaultGC(dpy, scr);

  /* bmt uses 32-bit Color struct (r,g,b,a). We'll create an XImage with 32bpp
   */
  int depth = DefaultDepth(dpy, scr);
  if (depth < 24) {
    fprintf(stderr,
            "Display depth %d is less than 24, may not render correctly\n",
            depth);
  }

  /* Create terminal and font */
  shitty_fontrenderer font = shitty_unifont_new();
  if (!font) {
    fprintf(stderr, "Failed to create unifont\n");
    XCloseDisplay(dpy);
    return 1;
  }

  shitty_terminal term = shitty_new(width, height, SHITTY_BUFMODE_DOUBLE, font);
  if (!term) {
    fprintf(stderr, "Failed to create terminal\n");
    shitty_unifont_del(font);
    XCloseDisplay(dpy);
    return 1;
  }

  /* Allocate a framebuffer: we'll use 32-bit little-endian (0xAARRGGBB as u32)
   * The Rust Color::as_u32 returns different layouts depending on features;
   * typical build uses BGRA For this example we assume RGBA-like layout: pack
   * as 0x00RRGGBB which works on most X11 visuals.
   */
  size_t pitch = width; /* number of pixels per row */
  uint32_t *fb = calloc(height * pitch, sizeof(uint32_t));
  if (!fb) {
    fprintf(stderr, "Failed to allocate framebuffer\n");
    shitty_del(term);
    XCloseDisplay(dpy);
    return 1;
  }

  /* Create XImage that wraps our framebuffer memory. Use ZPixmap format and 32
   * bits per pixel */
  XImage *xim = XCreateImage(dpy, DefaultVisual(dpy, scr), depth, ZPixmap, 0,
                             (char *)fb, width, height, 32, 0);
  if (!xim) {
    fprintf(stderr, "Failed to create XImage\n");
    free(fb);
    shitty_del(term);
    XCloseDisplay(dpy);
    return 1;
  }

  /* Event loop with periodic redraw */
  struct timespec last = {0, 0};
  while (1) {
    while (XPending(dpy)) {
      XEvent ev;
      XNextEvent(dpy, &ev);
      if (ev.type == Expose) {
        XPutImage(dpy, win, gc, xim, 0, 0, 0, 0, width, height);
      } else if (ev.type == KeyPress) {
        KeySym ks = XLookupKeysym(&ev.xkey, 0);
        char buf[32] = {0};
        int len = XLookupString(&ev.xkey, buf, sizeof(buf), NULL, NULL);
        if (len > 0) {
          shitty_process(term, (const shitty_u8 *)buf, (shitty_usize)len);
        }
      } else if (ev.type == ConfigureNotify) {
        XConfigureEvent *ce = &ev.xconfigure;
        if ((unsigned)ce->width != width || (unsigned)ce->height != height) {
          width = ce->width;
          height = ce->height;
          /* resize terminal and recreate framebuffer/XImage */
          shitty_resize(term, width, height);
          XDestroyImage(xim);
          pitch = width;
          fb = calloc(height * pitch, sizeof(uint32_t));
          xim = XCreateImage(dpy, DefaultVisual(dpy, scr), depth, ZPixmap, 0,
                             (char *)fb, width, height, 32, 0);
        }
      }
    }

    char time_text[64];
    snprintf(time_text, sizeof(time_text), "Current time: %ld\r\n", time(NULL));
    shitty_process(term, (const shitty_u8 *)time_text, strlen(time_text));

    /* call shitty_flush to render into our framebuffer */
    uint64_t ms = (uint64_t)(time(NULL) * 1000);
    shitty_flush(term, width, height, pitch, fb, ms);

    /* Put image to window */
    XPutImage(dpy, win, gc, xim, 0, 0, 0, 0, width, height);
    XFlush(dpy);

    /* simple sleep to limit CPU usage */
    usleep(16000);
  }

  /* never reached in this minimal example */
  XDestroyImage(xim); /* frees fb when using XCreateImage in this form on many
                         Xlibs, but we also free explicitly */
  free(fb);
  shitty_del(term);
  XCloseDisplay(dpy);
  return 0;
}
