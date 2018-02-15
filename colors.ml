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
	r = (abso_exp_decay (col1.r -. col2.r) half_life);
	v = (abso_exp_decay (col1.v -. col2.v) half_life);
	b = (abso_exp_decay (col1.b -. col2.b) half_life)})

(*Redirige la saturation d'une couleur vers les couleurs proches*)
let redirect_spectre col = {
	r = if col.v > 255. then col.r +. col.v -. 255. else col.r;
	v = if col.b > 255. && col.r > 255. then col.v +. col.r +. col.b -. 510.
	    else if col.r > 255. then col.v +. col.r -. 255.
	    else if col.b > 255. then col.v +. col.b -. 255.
	    else col.v;
	b = if col.v > 255. then col.b +. col.v -. 255. else col.b}

(*Même chose, mais redirige encore plus loin en cas de saturation extrème*)
let redirect_spectre_wide col = {
	r = if col.b > 510. then (
			if col.v > 255. then col.r +. col.v +. col.b -. 510. -. 255. else col.r +. col.b -. 510.
	    ) else (
			if col.v > 255. then col.r +. col.v -. 255. else col.r
	    );
	v = if col.b > 255. && col.r > 255. then col.v +. col.r +. col.b -. 510.
	    else if col.r > 255. then col.v +. col.r -. 255.
	    else if col.b > 255. then col.v +. col.b -. 255.
	    else col.v;
	b = if col.r > 510. then (
			if col.v > 255. then col.r +. col.v +. col.b -. 510. -. 255. else col.r +. col.b -. 510.
	    ) else (
			 if col.v > 255. then col.v +. col.b -. 255. else col.b
	    );}


(*Conversion de couleur_hdr vers couleur*)
let rgb_of_hdr hdr =
  let hdr_mod = redirect_spectre_wide (hdr_mul (hdr_add hdr (intensify !add_color !game_exposure)) !mul_color)in
	let normal_color fl = max 0 (min 255 (int_of_float fl)) in (*Fonction ramenant entre 0 et 255, qui sont les bornes du sRGB*)
	rgb (normal_color hdr_mod.r) (normal_color hdr_mod.v) (normal_color hdr_mod.b)

(*Fonction de saturation de la couleur*)
(*i un ratio entre 0 (N&B) et ce que l'on veut comme intensité des couleurs.*)
(*1 ne change rien*)
let saturate hdr_in i =
  let value = (hdr_in.r +. hdr_in.v +. hdr_in.b) /. 3. in
  {r = i *. hdr_in.r +. ((1. -. i) *. value); v = i *. hdr_in.v +. ((1. -. i) *. value); b= i *. hdr_in.b +. ((1. -. i) *. value)}

let space_color = ref {r = 0.; v = 0.; b = 0.}
let space_color_goal = ref {r = 0.; v = 0.; b = 0.}
let star_color = ref {r = 0.; v = 0.; b = 0.}
let star_color_goal = ref {r = 0.; v = 0.; b = 0.}
