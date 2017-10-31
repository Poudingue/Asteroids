# Asteroids

Bonjour,

Pour la compilation du programme, exécutez la commande :

ocamlc -o asteroids unix.cma graphics.cma asteroids.ml

Par défaut, les contrôles se font à la souris pour des raisons pratiques :
Clic pour accélérer, et ça reste barre d'espace pour tirer.
Il est possible d'avoir le contrôle clavier via le menu de pause (p),
en désactivant mouse control.
Il y a également d'autres options pour l'expérience de jeu ou les performances,
incluant un mode oldschool, se rapprochant beaucoup du jeu original,
avec mort instantanée, et destruction instantanée des astéroïdes.

Le contrôle clavier est adapté à un layout azerty :

p pour la pause,

z pour accélérer,
q pour tourner à gauche,
d pour tourner à droite,
a et e pour des déplacements latéraux gauche et droite

barre d'espace pour tirer (Maintenir fonctionne plus ou moins bien avec l'interface graphique d'Ocaml.)
f pour la téléportation aléatoire (Quand la barre bleue est pleine)
r pour recommencer rapidement une partie
k pour quitter rapidement le jeu

Dans le dossier archives, vous trouverez les versions précédentes du programme,
ainsi qu'un fichier changelog détaillant les évolutions.
Ces fichiers sont conçus pour fonctionner sous windows,
il vous faudra supprimer les imports du début pour pouvoir compiler,
ainsi qu'un appel à Unix.select qui sert à la limitation de framerate.

En cas de problèmes de performances, vous pouvez désactiver les effets de fumée dans les options,
ou baisser le nombre d'étoiles stars_nb_default, stars_nb, et stars_nb_previous,
ou simplement la résolution avec width et height

La limitation de framerate n'est pas fonctionelle sous linux,
mon appel bizarre à Unix.select n'est pas bien accepté.

Ocaml râle aussi sur des cas [] non pris en compte,
mais les tableaux sont déjà définis comme contenant 4 tuples de float,
donc j'ai ignoré ces avertissements.

Vous pouvez également tester le jeu en modifiant les valeurs par défaut,
par exemple en multipliant la densité du vaisseau par 1000 pour tester la physiques,
diminuer le cooldown des projectiles, augmenter leur nombre, le nombre d'étoiles...
La plupart des variables sont assez explicites et souvent commentées.
