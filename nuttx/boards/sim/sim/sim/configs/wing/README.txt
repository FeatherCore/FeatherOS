Wing Desktop Environment
========================

This configuration enables the Wing desktop environment, a mobile/watch-style
shell built on FHRE's declarative ECS architecture.

Features:
- Card-based UI (similar to Android/Symbian smartwatch systems)
- App preview cards in stack layout
- Quick settings panel
- Notification stack
- Gesture zones for navigation

Console Options
--------------

The Wing shell runs as an NSH builtin application. To start Wing::

  nsh> wing_rust

The shell will initialize and display the UI on the X11 framebuffer.

Framebuffer
-----------

- Resolution: 480x640 portrait (configurable via CONFIG_SIM_FBWIDTH/FBHEIGHT)
- X11-based display via NuttX simulation

Input
-----

- X11 mouse/touch events via /dev/input0
- X11 keyboard events via /dev/kbd

Dependencies
------------

- FHRE_RUST must be enabled
- Framebuffer support (X11)
- Touchscreen and keyboard input support
