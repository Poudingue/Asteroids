## Synopsis

This project is based on an assignement, which goal was to imitate as closely as possible the classic arcade game Asteroids.

I ended up adding many features, including collision physics with energy conservation, advanced visual effects (considering the ocaml Graphics API), and an «ok» gameplay loop.

Even though i used oCaml, a language i'm not really a big fan of, it worked out pretty good. I know the code is pretty clunky and repetitive sometimes, with some ugly workarounds, but it works !

I work on it on my spare time, and make the source code available to everyone, for fun and learning.

## Compilation and Launch

(You will need ocamlc)
You can compile this game using the command line :

ocamlc -o asteroids unix.cma graphics.cma parameters.ml functions.ml colors.ml objects.ml buttons.ml asteroids.ml
(for windows, it's asteroids.exe instead of asteroids)

Run it with ./asteroids (.exe for windows)

(Works both on linux and windows via powershell. Not sure for mac tho)

## Changelog

v1.9 - Optimisation update WIP

Features :
- Runs a lot smoother, even with a lot of objects, thanks to an other way to optimise collisions calculation
- Better camera behavior, still moves when paused, for style :)
- Gameplay tweaks everywhere
- Changed pause options

Fixed :
- Objects and fragments bouncing now time-based.
- Proper time correctly taken into account for every object, for physics and rendering
- Simplified code and objects

Problems :
- Now that the collision algorithm is optimised, what takes up most of the time is the rendering. Disable chunks and smoke effects for huge performance improvements.
- Angular momentum of objects still not taken into account for physics
- ???


For all changelogs, see changelog.txt
