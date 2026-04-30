# LVGL Window 10 Mobile Demo Migration

This example is a NuttX `sim:lvgl_fb` port of:

`/home/uan-gpd/codes/windows-10-mobile-lvgl`

The runnable NuttX application is kept at this directory root.  It uses the
migrated SquareLine generated UI under `ui/` and a NuttX-specific entry point in
`lvgl_window10.c`.

Only the files needed by the NuttX port are kept here.  The original project
can still be referenced from `/home/uan-gpd/codes/windows-10-mobile-lvgl` when
SquareLine project files, HAL code, screenshots, or other upstream assets are
needed.

Build from `nuttx/`:

```sh
./lvgl_build.sh
```

The `sim:lvgl_fb` board configuration starts `lvgl_window10_main` directly.
