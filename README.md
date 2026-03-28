## Historical OCaml Version

> **Note**: This branch (`ocaml`) preserves the original OCaml implementation of the game. It is no longer actively developed.
> Active development continues on the **`master`** branch, which is a full rewrite in **Rust + wgpu + SDL2** with many new features and visual effects.

## Synopsis

This project started as a university assignment to recreate the classic arcade game Asteroids as closely as possible.

It grew beyond that — collision physics with energy conservation, advanced visual effects (within the limits of the OCaml Graphics API), and a reasonably fun gameplay loop.

Even though OCaml isn't my favourite language, it worked out well. The code is sometimes clunky or repetitive with ugly workarounds, but it works!

## Compilation and Launch

You will need `ocamlc`.

Compile with:

```
ocamlc -o asteroids unix.cma graphics.cma parameters.ml functions.ml colors.ml objects.ml buttons.ml asteroids.ml
```

(On Windows, the output binary is `asteroids.exe`.)

Run with `./asteroids` (or `./asteroids.exe` on Windows).

Tested on Linux and Windows (PowerShell). macOS support is uncertain.

## Changelog

v1.9 - Optimisation update WIP

Features:
- Runs a lot smoother even with many objects, thanks to an improved collision optimisation algorithm
- Better camera behaviour — still moves when paused, for style
- Gameplay tweaks throughout
- Changed pause options

Fixed:
- Object and fragment bouncing is now time-based
- Proper time correctly taken into account for every object (physics and rendering)
- Simplified code and objects

Known issues:
- Rendering is now the main bottleneck. Disable chunks and smoke effects for large performance improvements.
- Angular momentum of objects not yet taken into account for physics

For full changelog history, see `changelog.txt`.
