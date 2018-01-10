## Synopsis

This project is based on an assignement, which goal was to imitate as closely as possible the classic arcade game Asteroids.

I ended up adding many features, including collision physics with energy conservation, advanced visual effects (considering the ocaml Graphics API), and an «ok» gameplay loop.

Even though i used oCaml, a language i'm not really a big fan of, it worked out pretty good. I know the code is pretty clunky and repetitive sometimes, with some ugly workarounds, but it works !

I work on it on my spare time, and make the source code available to everyone, for fun and/or learning.

## Compilation and Launch

(You will need ocamlc)
You can compile this game using the command line :

ocamlc -o asteroids unix.cma graphics.cma parameters.ml functions.ml colors.ml objects.ml buttons.ml asteroids.ml
(for windows, its asteroids.exe instead of asteroids)

Run it with ./asteroids (.exe for windows)

(Works both on linux and windows via powershell. Not sure for mac tho)

## Changelog

v1.8 beta - MAYHEM update - WIP

Features :
- Support for a huge number of asteroids at the same time, thanks to smart optimisation in the calculation of collisions
- Very satisfying and devastating death
- Same for teleportation
- Better explosions, better fire, better screenshake, better time dilation, better flashes
- Only infinitespace with mouse control available now

Fixed :
- No more massive asteroid spawn, fixed interval between two asteroid spawning
- Stupid performance problem where i forgot to despawn projectiles that are too far

Bugs :
- Momentum of objects still not taken into account for physics

Not cool :
- Final death can't be viewed yet.
- Respawn and instant die because of asteroid
- Teleport and instant die because of asteroid

For all changelogs, see changelog.txt
