## Synopsis

This project is based on an assignement, which goal was to imitate as closely as possible the classic arcade game Asteroids.

I ended up adding many features, including collision physics with energy conservation, advanced visual effects (considering the ocaml Graphics API), and an «ok» gameplay loop.

Even though i used oCaml, a language i'm not really a big fan of, it worked out pretty good. I know the code is pretty clunky and repetitive sometimes, with some ugly workarounds, but it works !

I work on it on my spare time, and make the source code available to everyone, for fun and/or learning.

## Compilation and Launch

You can compile this game using the command line :
(You will need ocamlc)

ocamlc -o asteroids unix.cma graphics.cma parameters.ml functions.ml colors.ml objects.ml buttons.ml asteroids.ml
(On windows write asteroids.exe instead of asteroids)

Run it with ./asteroids

## Changelog

v1.8 beta - MAYHEM update - WIP

Features :
- Support for a huge number of asteroids at the same time, thanks to smart optimisation in the calculatioin of collisions
- Better explosions, better fire

Fixed :
- No more massive asteroid spawn, fixed interval between two asteroid spawning
- No more ad vitam eternam respawn without lives

Bugs :
- Momentum of objects still not taken into account for physics

For all changelogs, see changelog.txt
