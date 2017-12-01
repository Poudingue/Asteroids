## Synopsis

This project is based on an assignement, which goal was to imitate as closely as possible the classic arcade game Asteroids.

I ended up adding many features, including collision physics with energy conservation, advanced visual effects (considering the ocaml Graphics API), and an «ok» gameplay loop.

Even though i used oCaml, a language i'm not really a big fan of, it worked out pretty good. I know the code is pretty clunky and repetitive sometimes, with some ugly workarounds, but it works !

I work on it on my spare time, and make the source code available to everyone, for fun and/or learning.

## Compilation and Launch

You can compile this game using the command line :

ocamlc -o asteroids unix.cma graphics.cma parameters.ml functions.ml colors.ml objects.ml buttons.ml asteroids.ml
(Not sure it works in windows)

Run it with ./asteroids

## Changelog

v1.7 - MORE FUN update

Features :
- Everything is faster !
- Light flashes for events (Shooting, teleporting and explosions)
- Tweaked screenshake, game speed change and exposure for a better game feel.
- More forgiving physics damages (allows easy asteroid-pushing :D)
- The default gamemode is now infinitespace
- The default weapon is now a powerfull shotgun

Fixed :
- Infinitespace now works properly, with objects that are too far coming back the other side
- Better performances thanks to a higher asteroid minimum radius and default infinitespace

For all changelogs, see changelog.txt
