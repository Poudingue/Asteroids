open Graphics
open Parameters
(******************************************************************************)
(*Définition des fonctions d'ordre général*)

(*Fonction de random float entre 2 valeurs*)
let randfloat min max = min +. Random.float (max -. min)

(*Fonction de carre, pour écrire plus jolimment les formules de pythagore*)
let carre v = v *. v

(*Fonction de décroissance exponentielle de n au bout de t secondes en float. Basée sur le temps ingame*)
let exp_decay n half_life = n *. 2. ** (!game_speed *. (!time_last_frame -. !time_current_frame) /. half_life)

(*Fonction de décroissance exponentielle de n au bout de t secondes en float. Basée sur le temps réel, pas sur le temps de jeu*)
let abso_exp_decay n half_life = n *. 2. ** ((!time_last_frame -. !time_current_frame) /. half_life)

(*Toujours pratique pour faire du joli pythagore*)
let hypothenuse (x, y) = sqrt (carre x +. carre y)

(*Permet l'addition de deux tuples*)
let addtuple (x1, y1) (x2, y2) = (x1 +. x2, y1 +. y2)

(*Permet la soustraction de deux tuples*)
let soustuple (x1, y1) (x2, y2) = (x1 -. x2, y1 -. y2)

(*Permet la multiplication d'un tuple par un float*)
let multuple (x, y) ratio = (x *. ratio, y *. ratio)

(*Moyenne de float*)
let moyfloat val1 val2 ratio = val1 *. ratio +. val2 *. (1. -. ratio)

(*Moyenne de deux tuples. le ratio est la pondération du premier.*)
let moytuple tuple1 tuple2 ratio = (addtuple (multuple tuple1 ratio) (multuple tuple2 (1.-.ratio)))

(*Permet la multiplication de deux termes séparés au sein d'un tuple*)
let multuple_parallel (x1,y1) (x2,y2) = (x1 *. x2, y1 *. y2)

(*Permet de vérifier qu'un tuple se trouve entre deux autres*)
let entretuple (x0,y0) (x1,y1) (x2,y2) = x0 > x1 && x0 < x2 && y0 > y1 && y0 < y2

(*Permet de convertir un tuple de float en tuple de int*)
let inttuple (x, y) = (int_of_float x, int_of_float y)

(*Permet de convertir un tuple de int en float*)
let floattuple (x, y) = (float_of_int x, float_of_int y)

(*Application du dithering global avant conversion en int*)
let dither fl = if dither_aa then int_of_float (fl +. Random.float dither_power) else int_of_float fl

(*Application du dithering global avant conversion en int*)
let dither_radius fl = if dither_aa then int_of_float (fl -. 0.5 +. Random.float dither_power_radius) else int_of_float fl

(*Permet un dithering suivant le dithering global sur un tuple. Permet une meilleure consistance visuelle entre éléments «ditherés»*)
let dither_tuple (x,y) = if dither_aa then inttuple (addtuple !current_jitter_double (x,y)) else inttuple (x,y)


(*Permet l'addition de deux tuples, en poundérant le second par le ratio*)
let proj tuple1 tuple2 ratio = addtuple tuple1 (multuple tuple2 ratio)

(*Transfert d'un vecteur en angle*valeur en x*y*)
let polar_to_affine angle valeur = (valeur *. cos angle, valeur *. sin angle)

(*Transfert d'un vecteur en angle*valeur en x*y*)
let polar_to_affine_tuple (angle, valeur) = polar_to_affine angle valeur

(*Transfert d'un vecteur en x*y en angle*valeur *)
let affine_to_polar (x, y) =
let r = hypothenuse (x, y) in
if r = 0. then (0., 0.) (*Dans le cas où le rayon est nul, on ne peut pas déterminer d'angle donné*)
else (2. *. atan (y /. (x +. r)),r)

(*La fonction distancecarre est plus simple niveau calcul qu'une fonction distance,*)
(*Car on évite la racine carrée, mais n'en reste pas moins utile pour les hitbox circulaires*)
let distancecarre (x1, y1) (x2, y2) = carre (x2 -. x1) +. carre (y2 -. y1)

let modulo_float value modulo = if value < 0. then value +. modulo else if value >= modulo then value -. modulo else value

(*Modulo pour le recentrage des étoiles*)
let modulo_reso (x, y) = (modulo_float x !phys_width, modulo_float y !phys_height)

(*Modulo pour le recentrage des objets hors de l'écran.
On considère une surface de 3x3 la surface de jeu.*)
(*À considérer : un espace carré pour avoir un gameplay indépendant du ratio*)
let modulo_3reso (x, y) =
  ((modulo_float (x +. !phys_width ) (!phys_width  *. 3.)) -. !phys_width,
   (modulo_float (y +. !phys_height) (!phys_height *. 3.)) -. !phys_height)


let diff l1 l2 = List.filter (fun x -> not (List.mem x l2)) l1
