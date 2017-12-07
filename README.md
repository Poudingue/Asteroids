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

v1.7 - MORE FUN update

Features :
- Everything is bigger and faster !
- Light flashes for events (Shooting, teleporting and explosions)
- Color correction, star colors and space color change from one stage to the next
- More forgiving physics damages (allows easy asteroid-pushing :D)
- The default gamemode is now infinitespace
- The default weapon is now a powerfull shotgun
- Overexposed colors become whiter. A very intense red will look orange, or yellow, and a very intense blue will look cyan
- Lots of tweaks, too much to enumerate

Fixed :
- Infinitespace now works properly, with objects that are too far coming back the other side
- Smart camera now follows the action as intended :
  - Properly focus on objects depending on mass an squared distance to the ship
  - The ship can't go outside of screen anymore
- Better performance thanks to a higher asteroid minimum radius and default infinitespace
- No more intricated asteroids and physics bugs.

Bugs :
- At the beginning of stages, the sudden spawn of countless asteroids create a massive performance hit.
  - Side effect : It creates an insane amount of chaos for more advanced stages
  - Suggested fix : Make the asteroids of the stage appear progressively
- Momentum of objects still not taken into account for physics

For all changelogs, see changelog.txt
