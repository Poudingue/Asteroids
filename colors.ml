open Graphics
open Parameters
open Functions
(*Fonctions sur les couleurs*)
(******************************************************************************)

(*Système de couleur*)
(*Pas de limite arbitraire de luminosité. Les négatifs donnent du noir et sont acceptés.*)
type hdr = {r : float ; v : float ; b : float;}

let hdr_add col1 col2 = {r = col1.r +. col2.r; v = col1.v +. col2.v; b = col1.b +. col2.b;}
let hdr_sous col1 col2 = {r = col1.r -. col2.r; v = col1.v -. col2.v; b = col1.b -. col2.b;}
let hdr_mul col1 col2 = {r = col1.r *. col2.r; v = col1.v *. col2.v; b = col1.b *. col2.b;}

(*couleur additive pour éclaircir toute l'image*)
let add_color = ref {r=0.;v=0.;b=0.}
let mul_base = ref {r=1.;v=1.;b=1.}
let mul_color = ref {r=0.;v=0.;b=0.}

(*Fonction d'intensité lumineuse d'une couleur hdr*)
let intensify hdr_in i = {r = i*. hdr_in.r ; v = i *. hdr_in.v ; b = i *. hdr_in.b}

let half_color col1 col2 half_life = (hdr_add col2 {
	r = (exp_decay (col1.r -. col2.r) half_life);
	v = (exp_decay (col1.v -. col2.v) half_life);
	b = (exp_decay (col1.b -. col2.b) half_life)})

(*Conversion de couleur_hdr vers couleur*)
let rgb_of_hdr hdr =
  let hdr_mod = hdr_mul (hdr_add hdr (intensify !add_color !game_exposure)) !mul_color in
  let normal_color fl = max 0 (min 255 (int_of_float fl)) in (*Fonction ramenant entre 0 et 255, qui sont les bornes du sRGB*)
  rgb (normal_color hdr_mod.r) (normal_color hdr_mod.v) (normal_color hdr_mod.b)

(*Fonction de saturation de la couleur*)
(*i un ratio entre 0 (N&B) et ce que l'on veut comme intensité des couleurs.*)
(*1 ne change rien*)
let saturate hdr_in i =
  let value = (hdr_in.r +. hdr_in.v +. hdr_in.b) /. 3. in
  {r = i *. hdr_in.r +. ((1. -. i) *. value); v = i *. hdr_in.v +. ((1. -. i) *. value); b= i *. hdr_in.b +. ((1. -. i) *. value)}

let space_color = {r = space_r; v = space_g; b = space_b}
