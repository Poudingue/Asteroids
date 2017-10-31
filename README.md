## Synopsis

This project is based on an assignement, which goal was to imitate as closely as possible the classic arcade game Asteroids.

I ended up adding many features, including collision physics with energy conservation, advanced visual effects (considering the ocaml Graphics API, i did a really good job), and an «ok» gameplay loop.

Even though i used oCaml, a language i'm not really a big fan of, it worked out pretty good. I know the code is pretty clunky and repetitive sometimes, with some ugly workarounds, but it works !

I'll maybe work on it on my spare time, but for now, i'll just make the source code available to everyone, for fun and/or learning.

## Compilation and Launch

You can compile this game using the command line :

ocamlc -o asteroids unix.cma graphics.cma asteroids.ml

Run it with ./asteroids
