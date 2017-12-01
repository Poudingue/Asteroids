open Graphics
open Parameters
open Functions
open Colors
open Objects
(******************************************************************************)
(*Définition types pour état du jeu*)




(*Types pour les boutons du menu*)
type buttonboolean = {
  pos1 : (float*float); (*Coin 1 du bouton*)
  pos2 : (float*float); (*Coin 2*)
  text : string; (*Texte à afficher dans le bouton*)
  text_over : string;
  boolean : bool ref; (*Référence du booléen à modifier*)
  mutable lastmousestate : bool; (*Permet de vérifier qu'à l'image précédente la souris était cliquée ou pas, afin d'éviter qu'à chaque frame le booléen soit changé*)
}

(*Type pour des boutons sliders*)
type sliderfloat = {
  pos1 : (float*float); (*Coin 1 du bouton*)
  pos2 : (float*float); (*Coin 2*)
  text : string; (*Texte à afficher dans le bouton*)
  valeur : float ref;(*Référence du float à modifier*)
  minval : float; (*Permet d'avoir une valeur plancher*)
  defaultval : float; (*Afficher la valeur par défaut*)
  maxval : float; (*Permet d'avoir une valeur max*)
}

(*Fonction permettant l'affichage du bouton et son activation*)
let applique_button button =
  ignore (button.boolean); (*Obligé de faire ça pour que la fonction n'essaye pas de l'appliquer à sliderfloat*)
  let (x0,y0) = inttuple (multuple button.pos1 !ratio_rendu) and (l,h) = inttuple (multuple (soustuple button.pos2 button.pos1) !ratio_rendu)in
  if !retro then (
    if !(button.boolean) = true then set_color white else set_color black;
    fill_rect x0 y0 l h; (*Intérieur du bouton*)
    set_color white; set_line_width 0; draw_rect x0 y0 l h; (*Contour du bouton*)
    let (wtext,htext) = text_size button.text in
    if !(button.boolean) = true then set_color black else set_color white;
    moveto (x0 + (l - wtext)/2 ) (y0 + (h - htext)/2 ); draw_string button.text
  ) else (
    if !(button.boolean) = true then set_color truecolor else set_color falsecolor;
    fill_rect x0 y0 l h; (*Intérieur du bouton*)
    set_color buttonframe; set_line_width buttonframewidth; draw_rect x0 y0 l h; (*Contour du bouton*)
    let (wtext,htext) = text_size button.text in
    set_color black; moveto (x0 + (l - wtext)/2 -1) (y0 + (h - htext)/2 -1); draw_string button.text;
    set_color white; moveto (x0 + (l - wtext)/2   ) (y0 + (h - htext)/2   ); draw_string button.text
  );
  (*On affiche le détail de ce que fait le bouton à côté de la souris*)
  if (entretuple (multuple (floattuple (mouse_pos ())) (1. /. !ratio_rendu)) button.pos1 button.pos2) then (
    let (x,y) = mouse_pos () in
    moveto (x-1) (y-1); set_color black; draw_string button.text_over;
    moveto x y; set_color white; draw_string button.text_over;
  );
  (*Si la souris est cliquée, ne l'était pas à la frame précédente, et est dans la surface du bouton*)
  if button_down () && not button.lastmousestate && (entretuple (multuple (floattuple (mouse_pos ())) (1. /. !ratio_rendu)) button.pos1 button.pos2)
    then button.boolean := not !(button.boolean);
  button.lastmousestate <- button_down ()

(*Fonction permettant l'affichage du bouton et son activation*)
let applique_slider ref_slider =
  let slider = !ref_slider in
  set_color slidercolor;
  let (x0,y0) = inttuple slider.pos1 and (l,h) = inttuple (soustuple slider.pos2 slider.pos1) in
  fill_rect x0 y0 l h; (*Intérieur du slider*)
  set_color buttonframe; set_line_width buttonframewidth; fill_rect x0 y0 l h; (*Contour du slider*)
  (*Si la souris est cliquée, ne l'était pas à la frame précédente, et est dans la surface du bouton*)
  if (button_down () && (entretuple (multuple (floattuple (mouse_pos ())) (1. /. !ratio_rendu)) slider.pos1 slider.pos2))
    then (let (x,y) = mouse_pos () in slider.valeur := (moyfloat slider.maxval slider.minval (float_of_int (x-x0)))) else();
  ref_slider := slider;;

type etat = {
  mutable buttons : buttonboolean list;
  mutable score : int;
  mutable lifes : int;
  mutable stage : int;
  mutable cooldown : float; (*Le cooldown est le temps restant avant de pouvoir de nouveau tirer*)
  mutable cooldown_tp : float; (*Le cooldown est le temps restant avant de pouvoir de nouveau tirer*)
  mutable last_health : float;
  mutable ref_ship : objet_physique ref;
(*Les objets sont des listes de référence, pour la simplicité de la gestion*)
(*Il est plus simple de gérer la physique en séparant les objets par type
plutôt que de vérifier le type d'objet à la volée*)
  mutable ref_fragments : objet_physique ref list; (*On fait apparaître les fragments dans une liste séparée pour éviter qu'ils ne s'entre-collisionnent*)
  mutable ref_fragments_unspawned : objet_physique ref list; (*Pour les fragments d'objets unspawned, pour éviter de les recentrer*)
  mutable ref_objets : objet_physique ref list;
  mutable ref_objets_unspawned : objet_physique ref list;
  mutable ref_chunks : objet_physique ref list;
  mutable ref_projectiles : objet_physique ref list;
  mutable ref_explosions : objet_physique ref list;
  mutable ref_smoke : objet_physique ref list;
  mutable ref_sparks : objet_physique ref list;
  mutable ref_stars : star ref list;
}

(*Rend un astéroïde spawné random*)
let spawn_random_asteroid stage =
  spawn_asteroid
    (random_out_of_screen asteroid_max_spawn_radius)
    (polar_to_affine (Random.float 2. *. pi) (randfloat asteroid_min_velocity (asteroid_max_velocity +. asteroid_stage_velocity *. (float_of_int stage))))
    (randfloat asteroid_min_spawn_radius asteroid_max_spawn_radius)

let rec spawn_n_asteroid stage n =
  match n with
  | 0 -> []
  | _ -> ref (spawn_random_asteroid stage) :: spawn_n_asteroid stage (n-1)


(*Diminution de la taille d'un astéroide*)
(*Permet de spawner plusieurs sous-asteroides lors de la fragmentation*)
let frag_asteroid ref_asteroid =
  let asteroid = !ref_asteroid in
  let fragment = spawn_asteroid asteroid.position asteroid.velocity asteroid.hitbox.int_radius in
  let orientation = (Random.float 2. *. pi) in
  let new_radius = (randfloat fragment_min_size fragment_max_size) *. fragment.hitbox.int_radius in
  let new_shape = polygon_asteroid new_radius (max asteroid_polygon_min_sides (int_of_float (asteroid_polygon_size_ratio *. new_radius))) in
  fragment.position <- addtuple fragment.position (polar_to_affine orientation (fragment.hitbox.int_radius -. new_radius));

  fragment.visuals.radius <- new_radius;
  fragment.visuals.color <- asteroid.visuals.color;
  fragment.visuals.shapes <- [(asteroid.visuals.color,new_shape)];

  fragment.hitbox.int_radius <- new_radius;
  fragment.hitbox.ext_radius <- new_radius *. asteroid_polygon_max;
  fragment.hitbox.points <- new_shape;

  fragment.mass <- pi *. asteroid_density *. (carre fragment.hitbox.int_radius);
  fragment.health <- asteroid_mass_health *. fragment.mass +. asteroid_min_health;
  fragment.max_health <- fragment.health;
  fragment.velocity <- addtuple fragment.velocity (polar_to_affine orientation (fragment_min_velocity +. Random.float (fragment_max_velocity -. fragment_min_velocity)));
  fragment.hdr_exposure <- fragment.hdr_exposure *. (fragment_min_exposure +. Random.float (fragment_max_exposure -. fragment_min_exposure));
  ref fragment


let spawn_random_star () =
let randpos = (Random.float !phys_width, Random.float !phys_height) in {
  last_pos = randpos;
  pos = randpos;
  proximity = (randfloat star_min_prox star_max_prox) ** 4.;
  lum = randfloat star_min_lum star_max_lum;
}

let rec n_stars n =
  if n=0 then [] else (ref (spawn_random_star ()) :: n_stars (n-1));;

let button_new_game={
  pos1 = ((4./.16.) *. !phys_width,(20./.24.) *. !phys_height);
  pos2 = ((6./.16.) *. !phys_width,(22./.24.) *. !phys_height);
  text = "New Game";
  text_over = "Start a new game with current parameters";
  boolean = restart;
  lastmousestate = false;}

let button_resume={
  pos1 = ((7./.16.) *. !phys_width,(20./.24.) *. !phys_height);
  pos2 = ((9./.16.) *. !phys_width,(22./.24.) *. !phys_height);
  text = "resume";
  text_over = "Resume current game";
  boolean = pause;
  lastmousestate = false;}

let button_quit={
  pos1 = ((10./.16.) *. !phys_width,(20./.24.) *. !phys_height);
  pos2 = ((12./.16.) *. !phys_width,(22./.24.) *. !phys_height);
  text = "quit";
  text_over = "Quit the game and go outside";
  boolean = quit;
  lastmousestate = false;}

let button_oldschool={
  pos1 = ((4./.16.) *. !phys_width,(12./.24.) *. !phys_height);
  pos2 = ((6./.16.) *. !phys_width,(14./.24.) *. !phys_height);
  text = "oldschool mode";
  text_over = "Play like in the old days.";
  boolean = oldschool;
  lastmousestate = false;}

let button_retro={
  pos1 = ((7./.16.) *. !phys_width,(12./.24.) *. !phys_height);
  pos2 = ((9./.16.) *. !phys_width,(14./.24.) *. !phys_height);
  text = "retro visuals";
  text_over = "White vectors on black background design";
  boolean = retro;
  lastmousestate = false;}

let button_scanlines={
  pos1 = ((10./.16.) *. !phys_width,(12./.24.) *. !phys_height);
  pos2 = ((12./.16.) *. !phys_width,(14./.24.) *. !phys_height);
  text = "scanlines";
  text_over = "Imitates the look of old CRT monitors.\nLowers luminosity.";
  boolean = scanlines;
  lastmousestate = false;}



let button_infinitespace={
  pos1 = ((4./.16.) *. !phys_width,(9./.24.) *. !phys_height);
  pos2 = ((6./.16.) *. !phys_width,(11./.24.) *. !phys_height);
  text = "infinitespace";
  text_over = "To infinity and beyond !";
  boolean = infinitespace;
  lastmousestate = false;}

let button_dynamic_camera={
  pos1 = ((7./.16.) *. !phys_width,(9./.24.) *. !phys_height);
  pos2 = ((9./.16.) *. !phys_width,(11./.24.) *. !phys_height);
  text = "dynamic camera";
  text_over = "Intelligent camera";
  boolean = dynamic_camera;
  lastmousestate = false;}

let button_hitbox={
  pos1 = ((10./.16.) *. !phys_width,(9./.24.) *. !phys_height);
  pos2 = ((12./.16.) *. !phys_width,(11./.24.) *. !phys_height);
  text = "Advanced hitbox";
  text_over = "A more precise hitbox.";
  boolean = advanced_hitbox;
  lastmousestate = false;}

let button_flashes={
  pos1 = ((10./.16.) *. !phys_width,(6./.24.) *. !phys_height);
  pos2 = ((12./.16.) *. !phys_width,(8./.24.) *. !phys_height);
  text = "Light Flashes";
  text_over = "Activates light flashes for events";
  boolean = flashes;
  lastmousestate = false;}

let button_screenshake={
  pos1 = ((4./.16.) *. !phys_width,(6./.24.) *. !phys_height);
  pos2 = ((6./.16.) *. !phys_width,(8./.24.) *. !phys_height);
  text = "screenshake";
  text_over = "Feel the impacts and explosions.";
  boolean = screenshake;
  lastmousestate = false;}

let button_smoke={
  pos1 = ((7./.16.) *. !phys_width,(6./.24.) *. !phys_height);
  pos2 = ((9./.16.) *. !phys_width,(8./.24.) *. !phys_height);
  text = "smoke particles";
  text_over = "Allows smoke. Disable for better performance.";
  boolean = smoke;
  lastmousestate = false;}

let button_mousecontrol={
  pos1 = ((4./.16.) *. !phys_width,(3./.24.) *. !phys_height);
  pos2 = ((6./.16.) *. !phys_width,(5./.24.) *. !phys_height);
  text = "mouse control";
  text_over = "Mouse to aim. Click to accelerate. Hold spacebar to shoot.";
  boolean = mousecontrol;
  lastmousestate = false;}

let button_framerate={
  pos1 = ((7./.16.) *. !phys_width,(3./.24.) *. !phys_height);
  pos2 = ((9./.16.) *. !phys_width,(5./.24.) *. !phys_height);
  text = "locked framerate";
  text_over = "Avoids calculating extra images, at the cost of fluidity";
  boolean = locked_framerate;
  lastmousestate = false;}

let init_etat () =  game_screenshake:=0. ;{
  buttons =
    [ button_quit ; button_resume ; button_new_game;
     button_scanlines ; button_retro ; button_oldschool;
     button_hitbox ; button_dynamic_camera ; button_infinitespace ;
     button_smoke ; button_screenshake ;
    button_framerate ; button_mousecontrol ; button_flashes];
  lifes = 3;
  score = 0;
  stage = 0;
  cooldown = 0.;
  cooldown_tp = 0.;
  last_health = ship_max_health;
  ref_ship = ref (spawn_ship ());
  ref_objets = [];
  ref_objets_unspawned = [];
  ref_fragments = [];
  ref_fragments_unspawned = [];
  ref_chunks = [];
  ref_projectiles = [];
  ref_explosions = [];
  ref_smoke = [];
  ref_sparks = [];
  ref_stars = n_stars !stars_nb;
}


(*Tout plein de fonctions permettant de faire des opérations sur des polygones.
Pas très bien présenté ni trié, je n'ai pas pris le temps de rendre ça plus lisible*)
(*Système de rotation de polygone pour rendu.*)
let rec rotat_poly poly rotat =
  let rotat_point (theta,rayon) rotat = (theta +. rotat,rayon) in
  if poly = [] then [] else List.append [(rotat_point (List.hd poly) rotat)] (rotat_poly (List.tl poly) rotat)

let rec scale_poly poly scale =
  let scale_point (theta,rayon) scale = (theta, rayon *. scale) in
  if poly = [] then [] else List.append [(scale_point (List.hd poly) scale)] (scale_poly (List.tl poly) scale)

let poly_to_affine poly rotat scale = List.map polar_to_affine_tuple (scale_poly (rotat_poly poly rotat) scale)

let rec depl_affine_poly poly pos = if poly = [] then [] else (addtuple (List.hd poly) pos) :: (depl_affine_poly (List.tl poly) pos)
let render_poly poly pos rotat color =
  let poly_to_render = depl_affine_poly (poly_to_affine poly rotat !ratio_rendu) pos in
  if !retro
    then (set_color white; set_line_width 0;draw_poly (Array.of_list (List.map dither_tuple poly_to_render)))
    else (set_color color; set_line_width 0;fill_poly (Array.of_list (List.map dither_tuple poly_to_render)));;

let rec render_shapes shapes pos rotat expos=
  match shapes with
  | [] -> ()
  | (hdcol,hdpoly)::tl ->
    (render_poly hdpoly pos rotat (rgb_of_hdr (intensify hdcol expos));
    render_shapes tl pos rotat expos);;

(*On dessine le polygone de l'objet.*)
let render_visuals objet offset =
  let visuals = objet.visuals in
  let position = (multuple (addtuple (addtuple objet.position !game_screenshake_pos) offset) !ratio_rendu) in
  if visuals.radius > 0. && not !retro then (
    set_color (rgb_of_hdr (intensify visuals.color (!game_exposure *. objet.hdr_exposure)));
    let (x,y) = dither_tuple position in
    fill_circle x y (dither_radius (visuals.radius *. !ratio_rendu))
  );
  render_shapes visuals.shapes position objet.orientation (!game_exposure *. objet.hdr_exposure)

let render_visuals_modulo objet =
  render_visuals objet (~-. !phys_width,~-. !phys_height);
  render_visuals objet (    0.         ,~-. !phys_height);
  render_visuals objet (    !phys_width,~-. !phys_height);

  render_visuals objet (~-. !phys_width, 0.);
  render_visuals objet (    0.         , 0.);
  render_visuals objet (    !phys_width, 0.);

  render_visuals objet (~-. !phys_width, !phys_height);
  render_visuals objet (    0.         , !phys_height);
  render_visuals objet (    !phys_width, !phys_height)

let render_objet ref_objet = if !infinitespace then (render_visuals !ref_objet (0.,0.)) else (render_visuals_modulo !ref_objet);;
let render_unspawned ref_objet = render_visuals !ref_objet (0.,0.)

(*Permet de rendre un polygone ayant des points déterminés en pourcentage de largeur et hauteur
en points en int. (Avec dither le cas échéant)*)
let rec relative_poly points_list =
  if points_list = [] then [] else inttuple (multuple_parallel (List.hd points_list) (float_of_int width,float_of_int height)) :: (relative_poly (List.tl points_list))


(*permet le rendu de motion blur sur des objets sphériques*)
(*Part de l'endroit où un objet était à l'état précédent pour décider*)
let render_light_trail radius last_pos pos velocity hdr_color =
(*TODO corriger le fait que le shutter_speed ne semble pas avoir d'influence sur la longueur des trainées de lumière dues au screenshake*)
  set_line_width (dither_radius (2.*.radius)); (*line_width en est le diamètre, d'où la multiplication par 2 du rayon*)
  let pos1 = (multuple (addtuple pos !game_screenshake_pos) !ratio_rendu) in (*Position actuelle de l'objet*)
  let veloc = multuple velocity ~-. (!game_speed *. (max (1. /. framerate_render) (1. *.(!time_current_frame -. !time_last_frame)))) in (*On projette d'une distance dépendant du temps depuis la dernière frame.*)
  let last_position = (multuple (addtuple (addtuple last_pos !game_screenshake_previous_pos) veloc) !ratio_rendu) in (*On calcule la position où l'objet était à la dernière frame en tenant compte de la vélocité et du screenshake.*)
  let pos2 = moytuple last_position pos1 shutter_speed in (*Plus la shutter_speed s'approche de 1, plus on se rapproche effectivement du point de l'image précédente pour la trainée.*)
  set_color (rgb_of_hdr (intensify hdr_color (!game_exposure *. 0.5 *. (sqrt (radius /. (radius +. hypothenuse (soustuple pos1 pos2)))))));(*Plus la trainée de lumière est grande par rapport au rayon de l'objet, moins la lumière est intense*)
  let (x1,y1) = dither_tuple pos1 in
  let (x2,y2) = dither_tuple pos2 in
  moveto x1 y1 ; lineto x2 y2;; (*On dessine le trait correspondant à la trainée.*)

(*Trainée de lumière pour le rendu des étoiles.*)
let render_star_trail ref_star =
  let star = !ref_star in (*Correspond globalement à la même fonction que ci-dessus*)
  let pos1 = (multuple (addtuple star.pos !game_screenshake_pos) !ratio_rendu) in
  let last_position = (multuple (addtuple star.last_pos (!game_screenshake_previous_pos)) !ratio_rendu) in
  let pos2 = moytuple last_position pos1 shutter_speed in
  let (x1,y1) = dither_tuple pos1 in
  let (x2,y2) = dither_tuple pos2 in
  let lum = star.lum +. Random.float star_rand_lum in
  let prox = star.proximity in
  let star_color = { (*Choix arbitraire de colorimétrie des étoiles.*)
    r= (prox *. star_prox_lum +.lum) *. 25.;
    v= (prox *. star_prox_lum +.lum) *. 50.;
    b= (prox *. star_prox_lum +.lum)  *. 200.} in
  if (x1 = x2 && y1 = y2) then ( (*Dans le cas où l'étoile n'a pas bougé, on rend plusieurs points, plutôt qu'une ligne.*)
    set_color (rgb_of_hdr (intensify (hdr_add star_color space_color) !game_exposure ));
    plot x1 y1;
      set_color (rgb_of_hdr (intensify star_color (0.25 *. !game_exposure)));
      plot (x1+1) y1 ; plot (x1-1) y1 ; plot x1 (y1+1) ; plot x1 (y1-1); (*Pour rendre un peu plus large qu'un simple point*)
      set_color (rgb_of_hdr (intensify star_color (0.125 *. !game_exposure)));
      plot (x1+1) (y1+1) ; plot (x1+1) (y1-1) ; plot (x1-1)  (y1+1) ; plot (x1-1)  (y1-1);
  )else (
    set_color (rgb_of_hdr (intensify star_color (!game_exposure *. (sqrt (1. /. (1. +. hypothenuse (soustuple pos1 pos2)))))));(*Plus la trainée de lumière est grande par rapport au rayon de l'objet, moins la lumière est intense*)
    moveto x1 y1 ; lineto x2 y2);;


let render_motion_blur ref_objet = (*TODO : Fonction ajouter, pour fondre avec le background*)
  let objet = !ref_objet in
  render_light_trail (!ratio_rendu *. objet.visuals.radius) objet.position objet.position objet.velocity (intensify objet.visuals.color (0.75 *. !game_exposure *. objet.hdr_exposure))
  (*Pour garder le motion blur discret, on rend les trainées plus sombres que l'objet.
  De même, on ne tient pas compte du déplacement de la caméra, car l'œuil humain va suivre ce type de mouvements.
  Le motion blur ne doit être visible que pour les mouvements violents de type screenshake,
  ou pour les objets allant vite.*)



(*Rendu des chunks. Pas de duplicatas, pas d'affichage de la vie, et l'objet est plus sombre*)
let render_chunk ref_objet =
  let objet = !ref_objet in
  let (x,y) = dither_tuple (multuple (addtuple objet.position !game_screenshake_pos) !ratio_rendu) in
  if !retro then (
    set_color (rgb 128 128 128);
    fill_circle x y (dither_radius (0.25 *. !ratio_rendu *. objet.visuals.radius));
  ) else (
    set_color (rgb_of_hdr (intensify objet.visuals.color (0.25 *. !game_exposure *. objet.hdr_exposure)));
    fill_circle x y (dither_radius (!ratio_rendu *. objet.visuals.radius)))


(*Rendu des projectiles. Dessine des trainées de lumière.*)
let render_projectile ref_projectile =
  let objet = !ref_projectile in
  let visuals = objet.visuals in
  let rad = !ratio_rendu *. (randfloat 0.5 1.) *. visuals.radius in
  if !retro
    then (let (x,y) = dither_tuple (multuple objet.position !ratio_rendu) in
      set_color white; fill_circle x y (dither_radius rad))
    else (
      (*On récupère les valeurs qu'on va utiliser plusieurs fois *)
      let last = objet.last_position and pos = objet.position and vel = objet.velocity
      and col = intensify visuals.color (objet.hdr_exposure *. !game_exposure) in
      (*On rend plusieurs traits concentriques pour un effet de dégradé*)
      render_light_trail rad last pos vel (intensify col 0.25);
      render_light_trail (rad *. 0.75) last pos vel (intensify col 0.5);
      render_light_trail (rad *. 0.5) last pos vel col;
      render_light_trail (rad *. 0.25) last pos vel (intensify col 2.))

let render_spark ref_spark =
  let objet = !ref_spark in
  render_light_trail objet.visuals.radius objet.last_position objet.position objet.velocity (intensify objet.visuals.color (objet.hdr_exposure *. !game_exposure));;


(*Fonction déplaçant un objet instantanémment sans prendre en compte le temps de jeu*)
let deplac_objet_abso ref_objet velocity =
let objet = !ref_objet in
objet.last_position <- objet.position;
if !dynamic_camera then objet.position <- proj objet.position velocity 1.;(*TODO sans doute moins sale de modifier direct la position*)
ref_objet := objet;;

(*Même chose pour plusieurs objets*)
let rec deplac_objets_abso ref_objets velocity =
if ref_objets = [] then () else (
deplac_objet_abso (List.hd ref_objets) velocity;
deplac_objets_abso (List.tl ref_objets) velocity)

(*Déplacement des étoiles en tenant compte de leur proximité*)
let deplac_star ref_star velocity =
  let star = !ref_star in
  star.last_pos <- star.pos;
  if !dynamic_camera then (
  let (next_x, next_y) = addtuple star.pos (multuple velocity star.proximity) in
  star.pos <- modulo_reso (next_x, next_y);
  if (next_x > !phys_width || next_x < 0. || next_y > !phys_height || next_y < 0.) then star.last_pos <- star.pos); (*On évite le motion blur incorrect causé par une téléportation d'un bord à l'autre de l'écran.*)
  ref_star := star

(*Déplacement d'un ensemble d'étoiles*)
let rec deplac_stars ref_stars velocity =
  if ref_stars = [] then [] else (deplac_star (List.hd ref_stars) velocity) :: (deplac_stars (List.tl ref_stars) velocity)


(*Fonction déplaçant un objet selon une vélocitée donnée.
On tient compte du framerate et de la vitesse de jeu,
mais également du temps propre de l'objet et de l'observateur*)
let deplac_objet ref_objet (dx, dy) =
let objet = !ref_objet in
  (*Si l'objet est un projectile, il despawne une fois au bord de l'écran*)
  objet.position <- proj objet.position (dx, dy) ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);

ref_objet := objet

(*Fonction accélérant un objet selon une accélération donnée.
On tient compte du framerate et de la vitesse de jeu,
mais également du temps propre de l'objet et de l'observateur*)
let accel_objet ref_objet (ddx, ddy) =
  let objet = !ref_objet in
  objet.velocity <- proj objet.velocity (ddx, ddy) ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
ref_objet := objet

(*Fonction boostant un objet selon une accélération donnée.*)
(*Utile pour le contrôle clavier par petites impulsions.*)
let boost_objet ref_objet boost =
  let objet = !ref_objet in objet.velocity <- (proj objet.velocity boost 1.);
ref_objet := objet

(*Fonction de rotation d'objet, avec rotation en radian*s⁻¹*)
let rotat_objet ref_objet rotation =
  let objet = !ref_objet in objet.orientation <- objet.orientation +. rotation *. ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
ref_objet := objet

(*Fonction de rotation d'objet, avec rotation en radian*s⁻²*)
let couple_objet ref_objet momentum =
  let objet = !ref_objet in
  objet.moment <- objet.moment +. momentum *. ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
ref_objet := objet

(*Fonction de rotation d'objet instantannée, avec rotation en radians.*)
let tourn_objet ref_objet rotation =
  let objet = !ref_objet in
  objet.orientation <- objet.orientation +. rotation;
ref_objet := objet

(*Fonction de rotation d'objet, avec rotation en radian*s⁻²*)
let couple_objet_boost ref_objet momentum =
  let objet = !ref_objet in
  objet.moment <- objet.moment +. momentum ;
ref_objet := objet

(*Fonction de calcul de changement de position inertiel d'un objet physique.*)
let inertie_objet ref_objet = deplac_objet ref_objet (!ref_objet).velocity

(*On calcule le changement de position inertiel de tous les objets en jeu*)
let inertie_objets ref_objets =
List.iter inertie_objet ref_objets (*TODO laisser tomber cette fonction, l'écrire direct telle-quelle dans la boucle de jeu.*)

let friction_objet ref_objet =
  let objet = !ref_objet in
  objet.velocity <- multuple objet.velocity (exp_decay 1. objet.half_stop);
  ref_objet:=objet

let friction_moment_objet ref_objet =
  let objet = !ref_objet in
  objet.moment <- exp_decay objet.moment objet.half_stop_rotat;
  ref_objet:=objet

(*On calcule l'inertie en rotation des objets*)
let moment_objet ref_objet = rotat_objet ref_objet (!ref_objet).moment

(*D'un groupe d'objets*)
let moment_objets ref_objets = List.iter moment_objet ref_objets (*TODO supprimer cette fonction et appeler direct telle-quelle dans la boucle principale.*)

let decay_smoke ref_smoke =
  let smoke = !ref_smoke in
  smoke.visuals.radius <- (exp_decay smoke.visuals.radius smoke_half_radius) -. (!game_speed *. smoke_radius_decay *. (!time_current_frame -. !time_last_frame));
  (*Si l'exposition est déjà minimale, ne pas encombrer par un calcul de décroissance expo supplémentaire*)
  if smoke.hdr_exposure > 0.005 then  smoke.hdr_exposure <- (exp_decay smoke.hdr_exposure smoke_half_col);
  ref smoke

let decay_chunk ref_chunk =
  let chunk = !ref_chunk in
  chunk.visuals.radius <- chunk.visuals.radius -. (!game_speed *. chunk_radius_decay *. (!time_current_frame -. !time_last_frame));
  ref chunk

let damage ref_objet damage =
  let objet = !ref_objet in
  if (!oldschool)
    then (objet.health <- ~-.0.1)
    else (objet.health <- objet.health -. (max 0. (objet.dam_ratio *. damage -. objet.dam_res)));
  game_screenshake := !game_screenshake +. damage *. screenshake_dam_ratio;
  if variable_exposure then game_exposure := !game_exposure *. exposure_ratio_damage;
  if !flashes then add_color := hdr_add !add_color (intensify {r=1.;v=0.7;b=0.5} (damage *. flashes_damage));
  ref_objet := objet

let phys_damage ref_objet damage =
  let objet = !ref_objet in
  if (!oldschool)
    then (objet.health <- ~-.0.1)
    else (objet.health <- objet.health -. (max 0. (objet.phys_ratio *. damage -. objet.phys_res)));
  game_screenshake := !game_screenshake +. damage *. screenshake_phys_ratio *. objet.mass /. screenshake_phys_mass;
  ref_objet := objet

let is_alive ref_objet = !ref_objet.health >= 0.
let is_dead ref_objet = !ref_objet.health <0.

(*Vérifie si un objet a le droit de spawner. (Si il est dans l'écran)*)
let checkspawn_objet ref_objet_unspawned =
  let objet = !ref_objet_unspawned in
  let (x, y) = objet.position in
  let rad = objet.hitbox.ext_radius in
 (x +. rad < !phys_width) && (x -. rad > 0.)
  && (y +. rad < !phys_height) && (y -. rad > 0.)
let checknotspawn_objet ref_objet_unspawned = not (checkspawn_objet ref_objet_unspawned)

(*Fait spawner tous les objets en ayant le droit*)
let checkspawn_etat ref_etat =
  if !ref_etat.ref_objets_unspawned = [] then ()
  else begin
    let etat = !ref_etat in
    let objets = etat.ref_objets
    and objets_unspawned = etat.ref_objets_unspawned in
    etat.ref_objets <- (List.filter checkspawn_objet objets_unspawned) @ objets;
    etat.ref_objets_unspawned <- (List.filter checknotspawn_objet objets_unspawned);
  ref_etat := etat end

(*Booléen indiquant qu'un objet est suffisamment proche pour être encore pris en compte dans l'espace de jeu*)
let close_enough ref_objet =
  let (x, y) = !ref_objet.position in
  if !infinitespace then (
    hypothenuse (soustuple (x,y) (!phys_width /. 2., !phys_height /. 2.)) < max_dist
  ) else (
       (x < 2. *. !phys_width)
    && (x > 0. -. !phys_width)
    && (y < 2. *. !phys_height)
    && (y > 0. -. !phys_height))

let too_far ref_objet = not (close_enough ref_objet)

let close_enough_bullet ref_objet =
  let (x, y) = !ref_objet.position in
  if !infinitespace then (
    hypothenuse (x,y) < max_dist
  ) else (
    (x < 1.01 *. !phys_width)
    && (x > 0. -. (0.01  *. !phys_width))
    && (y < 1.01 *. !phys_height)
    && (y > 0. -. (0.01 *. !phys_height)))

let positive_radius ref_objet = !ref_objet.visuals.radius > 0.

let big_enough ref_objet = !ref_objet.hitbox.int_radius > asteroid_min_size
let too_small ref_objet = not (big_enough ref_objet)


(*Fonctions permettant de calculer une caméra dynamique suivant en priorité les objets massifs et proches*)
let rec sum_center ref_objets pos =
  match ref_objets with
  |[] -> (0.,0.)
  |hd::tl -> addtuple (multuple !hd.position (!hd.mass /. ((distancecarre !hd.position pos)))) (sum_center tl pos)

let rec sum_mass ref_objets pos =
  match ref_objets with
  |[] -> 0.
  |hd::tl -> (!hd.mass /. (1. +. (distancecarre !hd.position pos))) +. (sum_mass tl pos)

let center_of_attention ref_objets pos = if ref_objets = [] then (0.,0.) else (multuple (sum_center ref_objets pos)  (1. /. (1. +. sum_mass ref_objets pos)))

(*Fonction despawnant les objets trop lointains et morts, ou avec rayon négatif*)
let despawn ref_etat =
  let etat = !ref_etat in
  (*On met les objets dans la liste de chunks, qui ne sont que décoratif et pour lesquels on ne calculera pas les collisions*)
    etat.ref_chunks <- (List.append (List.map decay_chunk etat.ref_chunks) (List.append
      (List.append (List.filter too_small etat.ref_objets) (List.filter too_small etat.ref_objets_unspawned))
      (List.append (List.filter too_small etat.ref_fragments) (List.filter too_small etat.ref_fragments_unspawned))));

    (*Plus besoin de checker les objets unspawned à supprimer, ils ont maintenant une légère tendance à accélérer vers le centre de l'écran.*)

    etat.ref_objets <- (List.filter is_alive etat.ref_objets);
    etat.ref_objets <- (List.filter big_enough etat.ref_objets);

    etat.ref_objets_unspawned <- (List.filter is_alive etat.ref_objets_unspawned);
    etat.ref_objets_unspawned <- (List.filter big_enough etat.ref_objets_unspawned);

    etat.ref_fragments <- (List.filter is_alive etat.ref_fragments);
    etat.ref_fragments <- (List.filter big_enough etat.ref_fragments);

    etat.ref_fragments_unspawned <- (List.filter is_alive etat.ref_fragments_unspawned);
    etat.ref_fragments_unspawned <- (List.filter big_enough etat.ref_fragments_unspawned);


    etat.ref_projectiles <- (List.filter is_alive etat.ref_projectiles);
    (*TODO permettre un missile ne despawnant pas après mort, mais provoquant plusieurs explosions sur son passage*)

    if not !infinitespace then etat.ref_projectiles <- (List.filter close_enough_bullet etat.ref_projectiles) else etat.ref_projectiles <- (List.filter close_enough etat.ref_projectiles);

    etat.ref_smoke <- (List.filter positive_radius etat.ref_smoke);
    etat.ref_chunks <- (List.filter positive_radius etat.ref_chunks);
  ref_etat := etat


(*Recentrer les objets débordant de l'écran d'un côté de l'écran ou de l'autre*)
let recenter_objet ref_objet =
  let objet = !ref_objet in
  let (next_x, next_y) = modulo_reso objet.position in
  objet.position <- (next_x, next_y);
  if (next_x > !phys_width || next_x < 0. || next_y > !phys_height || next_y < 0.)
    then objet.last_position <- objet.position;(*On évite d'avoir du flou incorrect d'un côté à l'autre de l'écran*)
ref_objet := objet

(*Fonction permettant de rapprocher les objets lointains*)
let closer_objet ref_objet =
  let objet = !ref_objet in
  let (x,y) = objet.position in
  objet.position <- addtuple (!phys_width /. 2., !phys_height /. 2.) (exp_decay (x -. !phys_width /. 2.) half_close, exp_decay (y -. !phys_height /. 2.) half_close);
  objet.velocity <- addtuple objet.velocity (multuple (!phys_width /. 2. -. x, !phys_height /. 2. -. y) accel_close);
  ref_objet := objet;;


(*On recentre les objets qui sont hors de l'écran, mais selon un écran 3 fois plus large et haut*)
let recenter_objet_unspawned ref_objet =
  let objet = !ref_objet in
  objet.position <- modulo_3reso objet.position;
ref_objet := objet

(*La racine carrée est une opération assez lourde,
Donc plutôt que de comparer la distance entre deux objets avec la somme de leur radii,
On compare le carré de leur distance avec le carré de la somme de leurs radii..
On travaille par hitbox circulaire pour 1-La simplicité du calcul 2-La proximité avec les formes réelles*)

let collision_circles pos0 r0 pos1 r1 = distancecarre pos0 pos1 < carre (r0 +. r1)

let collision_point pos_point pos_circle radius = distancecarre pos_point pos_circle < carre radius

let rec collisions_points pos_points pos_circle radius =
match pos_points with
|[] -> false
|hd::tl -> collision_point hd pos_circle radius || collisions_points tl pos_circle radius

let collision_poly pos poly rotat circle_pos radius =
  let pos_points = (depl_affine_poly (poly_to_affine poly rotat 1.) pos) in
  collisions_points pos_points circle_pos radius

(*Fonction vérifiant la collision entre deux objets*)
let collision objet1 objet2 =
(*Si on essaye de collisionner un objet avec lui-même, ça ne fonctionne pas*)
if objet1 = objet2 then false
  else (
  let hitbox1 = objet1.hitbox and hitbox2 = objet2.hitbox
  and pos1 = objet1.position and pos2 = objet2.position in
  collision_circles pos1 hitbox1.int_radius pos2 hitbox2.int_radius
  || !advanced_hitbox &&(  collision_circles pos1 hitbox1.ext_radius pos2 hitbox2.ext_radius
  && (( collision_poly pos1 hitbox1.points objet1.orientation pos2 hitbox2.int_radius)
     ||(collision_poly pos2 hitbox2.points objet2.orientation pos1 hitbox1.int_radius))))

(*Vérifie la collision entre un objet et une liste d'objets*)
let rec collision_objet_liste ref_objet ref_objets =
  match ref_objets with
  | [] -> false
  | _ -> collision !ref_objet !(List.hd ref_objets) || collision_objet_liste ref_objet (List.tl ref_objets)

(*Retourne les objets de la liste 1 étant en collision avec des objets de la liste 2*)
let rec collision_objets_listes ref_objets1 ref_objets2 =
  if ref_objets1 = [] || ref_objets2 = [] then []
  else if collision_objet_liste (List.hd ref_objets1) ref_objets2
    then List.hd ref_objets1 :: collision_objets_listes (List.tl ref_objets1) ref_objets2
    else collision_objets_listes (List.tl ref_objets1) ref_objets2

(*Retourne les objets de la liste 1 n'étant PAS en collision avec des objets de la liste 2*)
let rec no_collision_objets_listes ref_objets1 ref_objets2 =
  if ref_objets1 = [] then [] else if ref_objets2 = [] then ref_objets1
  else if collision_objet_liste (List.hd ref_objets1) ref_objets2
    then no_collision_objets_listes (List.tl ref_objets1) ref_objets2
    else List.hd ref_objets1 :: no_collision_objets_listes (List.tl ref_objets1) ref_objets2

(*Retourne tous les objets d'une liste étant en collision avec au moins un autre*)
let rec collisions_sein_liste ref_objets = collision_objets_listes ref_objets ref_objets

(*Retourne tous les objets au sein d'une liste n'étant pas en collision avec les autres*)
let rec no_collisions_liste ref_objets = no_collision_objets_listes ref_objets ref_objets

(*Fonction appelée en cas de collision de deux objets.
La fonction pourrait être améliorée, avec une variable friction sur les objets,
et transfert entre moment et inertie.*)
let consequences_collision ref_objet1 ref_objet2 =
  match !ref_objet1.objet with
  | Explosion -> damage ref_objet2 explosion_damages (*On applique les dégats de l'explosion*)
  | Projectile -> damage ref_objet1 0.1 (*On endommage le projectile pour qu'il meure*)
  | _ -> (*Si ce n'est ni une explosion ni un projectile, on calcule les effets de la collision physique*)
    (let objet1 = !ref_objet1 in let objet2 = !ref_objet2 in
    let total_mass = objet1.mass +. objet2.mass in
    let moy_velocity = moytuple objet1.velocity objet2.velocity (objet1.mass /. total_mass) in
    let (angle_obj1, dist1) = affine_to_polar (soustuple objet1.position objet2.position) in
    let (angle_obj2, dist2) = affine_to_polar (soustuple objet2.position objet1.position) in
    (*Stockage des ancienne vélocités, pour calculer les dégats en fonction du nombre de G encaissées*)

    let old_vel1 = objet1.velocity in
    let old_vel2 = objet2.velocity in

    let veloc_obj1 = addtuple moy_velocity (polar_to_affine angle_obj1 (total_mass /. objet1.mass)) in
    objet2.velocity <- addtuple moy_velocity (polar_to_affine angle_obj2 (total_mass /. objet2.mass));
    objet1.velocity <- veloc_obj1;

    (*Pour éloigner les objets intriqués*)
    objet1.position <- addtuple objet1.position (polar_to_affine angle_obj1 min_repulsion);
    objet2.position <- addtuple objet2.position (polar_to_affine angle_obj2 min_repulsion);

    (*Changement de velocité subi par l'objet*)
    let g1 = hypothenuse (soustuple old_vel1 objet1.velocity) in
    let g2 = hypothenuse (soustuple old_vel2 objet2.velocity) in

    ref_objet1 := objet1;
    ref_objet2 := objet2;
    (*Les dégats physiques dépendent du changement de vitesse subie au carré.
    On applique un ratio pour réduire la valeur gigantesque générée*)
    phys_damage ref_objet1 (!ratio_phys_deg *. carre g1);
    phys_damage ref_objet2 (!ratio_phys_deg *. carre g2))

(*Fonction vérifiant la collision entre un objet et les autres objets
et appliquant les effets de collision*)
let rec calculate_collisions_objet ref_objet ref_objets =
if ref_objets = [] then () else (
  if collision !ref_objet !(List.hd ref_objets) then consequences_collision ref_objet (List.hd ref_objets);
  calculate_collisions_objet ref_objet (List.tl ref_objets))

let rec calculate_collisions_objets ref_objets =
if List.length ref_objets <= 1 then () else (
  calculate_collisions_objet (List.hd ref_objets) (List.tl ref_objets);
  calculate_collisions_objets (List.tl ref_objets))

let rec calculate_collisions_listes_objets ref_objets1 ref_objets2 =
if ref_objets1 = [] || ref_objets2 = [] then () else (
  calculate_collisions_objet (List.hd ref_objets1) ref_objets2;
  calculate_collisions_listes_objets (List.tl ref_objets1) ref_objets2)

(*Petite fonction de déplacement d'objet exprès pour les modulos*)
(*Car la fonction de déplacement standard dépend de Δt*)
let deplac_obj_modulo ref_objet (x,y) = (*x et y sont des entiers, en quantité d'écrans*)
  let objet = !ref_objet in
  objet.position <- addtuple objet.position (!phys_width *. float_of_int x, !phys_height *. float_of_int y);
  ref_objet := objet

(*Fonction permettant aux objets simultanément à plusieurs endroits de l'écran de réagir correctement au niveau physique*)
let rec calculate_collisions_modulo ref_objet ref_objets =
if List.length ref_objets > 0 then (
  (*En mode infinitespace, on ignore la partie modulo du calcul*)
  if not !infinitespace then (
    (*duplicata haut de l'objet*)
    deplac_obj_modulo ref_objet (0, 1);
    calculate_collisions_objet ref_objet ref_objets;
    (*Duplicata haut droite de l'objet*)
    deplac_obj_modulo ref_objet (1, 0);
    calculate_collisions_objet ref_objet ref_objets;
    (*Duplicata droit de l'objet*)
    deplac_obj_modulo ref_objet (0, ~-1);
    calculate_collisions_objet ref_objet ref_objets;
    (*Duplicata droit bas*)
    deplac_obj_modulo ref_objet (0, ~-1);
    calculate_collisions_objet ref_objet ref_objets;
    (*Duplicata bas*)
    deplac_obj_modulo ref_objet (~-1, 0);
    calculate_collisions_objet ref_objet ref_objets;
  (*Duplicata bas gauche*)
    deplac_obj_modulo ref_objet (~-1, 0);
    calculate_collisions_objet ref_objet ref_objets;
    (*Duplicata gauche*)
    deplac_obj_modulo ref_objet (0, 1);
    calculate_collisions_objet ref_objet ref_objets;
    (*Duplicata haut gauche*)
    deplac_obj_modulo ref_objet (0, 1);
    calculate_collisions_objet ref_objet ref_objets;
    (*On remet l'objet à son emplacement habituel et on calcule sa physique*)
    deplac_obj_modulo ref_objet (1, ~-1)));

  (*On calcule aussi la collision à son lieu original)*)
  calculate_collisions_objet ref_objet ref_objets

(*Même chose, pour une liste de ref objets*)
let rec calculate_collisions_modulos ref_objets =
if List.length ref_objets > 1 then (
calculate_collisions_modulo (List.hd ref_objets) (List.tl ref_objets);
calculate_collisions_modulos (List.tl ref_objets))
else ()

(*Même chose, mais collision entre deux listes*)
let rec calculate_collisions_modulo_listes ref_objets1 ref_objets2 =
if ref_objets1 = [] || ref_objets2 = [] then () else (
calculate_collisions_modulo (List.hd ref_objets1) ref_objets2;
calculate_collisions_modulo_listes (List.tl ref_objets1) ref_objets2)


(* --- initialisations etat --- *)

(* Affichage des états*)

(*Fonction d'affichage de barre de vie. Nécessite un quadrilatère comme polygone d'entrée.
Les deux premiers points correspondent à une valeur de zéro, et les deux derniers à la valeur max de la barre.
On peut mettre des quadrilatères totalement arbitraires*)
let affiche_barre ratio [point0;point1;point2;point3] color_bar =
  (*Cette fonction me prévient comme quoi je n'ai pas prévu le cas [].
  Cependant, je pense que c'est suffisamment clair qu'on impose un argument
  avec 4 tuplés. Du coup j'ignore purement et simplement cet avertissement.*)
  set_color color_bar;
  fill_poly (Array.of_list (relative_poly
  [point0;point1;
  (*Pour les deux points devant bouger selon le ratio,
  on fait simplement une moyenne pondérée.*)
  (moytuple point2 point1 ratio);
  (moytuple point3 point0 ratio)]));;


(*Fonction attribuant une forme à un caractère*)
let shape_char carac =
  match carac with
  |'0' -> [(0.25 ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.8);(0.75 , 1.);(0.25 ,1. );(0.  ,0.8 );(0.  ,0.2 );(0.25 ,0.2);(0.75 ,0.6);(0.75 ,0.8);(0.25,0.375);(0.25,0.8);(0.75,0.8);(0.75,0.2);(0.,0.2)]
  |'1' -> [(0.125,0.);(0.875,0.);(0.875,0.2);(0.625,0.2);(0.625,1. );(0.375,1. );(0.  ,0.75);(0.15,0.65);(0.375,0.8);(0.375,0.2);(0.125,0.2)]
  |'2' -> [(0.   ,0.);(1.   ,0.);(1.   ,0.2);(0.35 ,0.2);(1.   ,0.5);(1.   ,0.8);(0.75,1.  );(0.25,1.  );(0.   ,0.8);(0.   ,0.6);(0.25 ,0.6);(0.25,0.8  );(0.75,0.8);(0.75,0.6);(0.,0.2)]
  |'3' -> [(0.25 ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.4);(0.875,0.5);(1.   ,0.6);(1.  ,0.8 );(0.75,1.  );(0.25 ,1. );(0.   ,0.8);(0.   ,0.6);(0.25,0.6  );(0.25,0.8);(0.75,0.8);(0.75,0.6);(0.5,0.6);(0.5,0.4);(0.75,0.4);(0.75,0.2);(0.25,0.2);(0.25,0.4);(0.,0.4);(0.,0.2)]
  |'4' -> [(0.5  ,0.);(0.75 ,0.);(0.75 ,1. );(0.5  ,1. );(0.   ,0.4);(0.   ,0.2);(1.  ,0.2 );(1.  ,0.4 );(0.25 ,0.4);(0.5  ,0.8)]
  |'5' -> [(0.25 ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.5);(0.25 ,0.7);(0.25 ,0.8);(1.  ,0.8 );(1.  ,1.  );(0.   ,1. );(0.   ,0.6);(0.75 ,0.4);(0.75,0.2  );(0.25,0.2);(0.25,0.35);(0.,0.4);(0.,0.2);(0.25,0.)]
  |'6' -> [(0.25 ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.4);(0.75 ,0.6);(0.25 ,0.6);(0.25,0.8 );(1.  ,0.8 );(0.75 ,1. );(0.25 ,1. );(0.   ,0.8);(0.  ,0.4  );(0.75,0.4);(0.75,0.2);(0.25,0.2);(0.25,0.4);(0.,0.4);(0.,0.2)]
  |'7' -> [(0.25 ,0.);(0.5  ,0.);(1.   ,0.8);(1.   ,1. );(0.   ,1. );(0.   ,0.8);(0.75,0.8 )]
  |'8' -> [(0.25 ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.4);(0.875,0.5);(1.   ,0.6);(1.  ,0.8 );(0.75,1.  );(0.25 ,1. );(0.25 ,0.8);(0.75 ,0.8);(0.75,0.6  );(0.25,0.6);(0.25,0.4);(0.75,0.4);(0.75,0.2);(0.25,0.2);(0.25,1.);(0.,0.8);(0.,0.6);(0.125,0.5);(0.,0.4);(0.,0.2)]
  |'9' -> [(0.75 ,1.);(0.25 ,1.);(0.   ,0.8);(0.   ,0.6);(0.25 ,0.4);(0.75 ,0.4);(0.75,0.2 );(0.  ,0.2 );(0.25 ,0. );(0.75 ,0. );(1.   ,0.2);(1.  ,0.6  );(0.25,0.6);(0.25,0.8);(0.75,0.8);(0.75,0.6);(1.,0.6);(1.,0.8)]
  |' ' -> [(0.,0.);(0.,0.);(0.,0.)]
  |'A' -> [(0.   ,0.);(0.25 ,0.);(0.25 ,0.4);(0.75 ,0.4);(0.75 ,0.4);(0.75 ,0.6);(0.25,0.6 );(0.25,0.8 );(0.75 ,0.8);(0.75 ,0. );(1.   ,0. );(1.  ,0.8  );(0.75,1. );(0.25,1. );(0.,0.8)]
  |'B' -> [(0.   ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.4);(0.875,0.5);(1.   ,0.6);(1.  ,0.8 );(0.75,1.  );(0.25 ,1. );(0.25 ,0.8);(0.75 ,0.8);(0.75,0.6  );(0.25,0.6);(0.25,0.4);(0.75,0.4);(0.75,0.2);(0.25,0.2);(0.,1.)]
  |'C' -> [(0.25 ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.4);(0.75 ,0.4);(0.75 ,0.2);(0.25,0.2 );(0.25 ,0.8);(0.75 ,0.8);(0.75 ,0.6);(1.   ,0.6);(1.   ,0.8);(0.75,1.   );(0.25,1. );(0.  ,0.8);(0.  ,0.2)]
  |'D' -> [(0.   ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.8);(0.75 ,1. );(0.   ,1. );(0.   ,0.2);(0.25 ,0.2);(0.25 ,0.8);(0.75,0.8);(0.75,0.2);(0.,0.2)]
  |'E' -> [(0.   ,0.);(0.75 ,0.);(1.   ,0.2);(0.25 ,0.2);(0.25 ,0.4);(0.5  ,0.4);(0.5 ,0.6 );(0.25 ,0.6);(0.25 ,0.8);(1.   ,0.8);(0.75 ,1. );(0.   ,1. )]
  |'F' -> [(0.   ,0.);(0.25 ,0.);(0.25 ,0.4);(0.5  ,0.4);(0.75 ,0.6);(0.25 ,0.6);(0.25,0.8 );(1.   ,0.8);(1.   ,1. );(0.   ,1. );]
  |'G' -> [(0.25 ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.6);(0.5  ,0.6);(0.5  ,0.4);(0.75,0.4 );(0.75 ,0.2);(0.25 ,0.2);(0.25 ,0.8);(1.   ,0.8);(0.75,1.   );(0.25,1. );(0.  ,0.8);(0.  ,0.2)]
  |'I' -> [(0.125,0.);(0.875,0.);(0.875,0.2);(0.625,0.2);(0.625,0.8);(0.875,0.8);(0.875,1. );(0.125,1. );(0.125,0.8);(0.375,0.8);(0.375,0.2);(0.125,0.2)]
  |'O' -> [(0.25 ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.8);(0.75 ,1. );(0.25 ,1. );(0.  ,0.8 );(0.   ,0.2);(0.25 ,0.2);(0.25 ,0.8);(0.75,0.8);(0.75,0.2);(0.,0.2)]
  |'R' -> [(0.   ,0.);(0.25 ,0.);(0.25 ,0.8);(0.75 ,0.8);(0.75 ,0.6);(0.25 ,0.6);(0.25,0.4 );(0.75 ,0. );(1.   ,0. );(0.5  ,0.4);(0.75,0.4);(1.  ,0.6);(1.,0.8);(0.75,1.);(0.,1.)]
  |'S' -> [(0.25 ,0.);(0.75 ,0.);(1.   ,0.2);(1.   ,0.4);(0.75 ,0.6);(0.25 ,0.6);(0.25,0.8 );(1.   ,0.8);(0.75 ,1. );(0.25 ,1. );(0.   ,0.8);(0.  ,0.6  );(0.25,0.4);(0.75,0.4);(0.75,0.2);(0. ,0.2)]
  |'T' -> [(0.385,0.);(0.625,0.);(0.625,0.8);(1.   ,0.8);(1.   ,1. );(0.   ,1. );(0.  ,0.8 );(0.385,0.8)]
  |'W' -> [(0.   ,1.);(0.2  ,0.);(0.4  ,0. );(0.5  ,0.2);(0.6  ,0. );(0.8  ,0. );(1.  ,1.  );(0.6  ,0.4);(0.6  ,0.6);(0.4  ,0.6);(0.4  ,0.4);(0.2 ,1.   )]
  | _  -> [(0.   ,0.);(1.   ,0.);(1.   ,1. );(0.   ,1. )]

(*Fonction prenant 4 points d'encadrement, et un point relatif, et le rendant transformé*)
let displacement [point0;point1;point2;point3] (relx,rely) = multuple (moytuple (moytuple point2 point1 rely) (moytuple point3 point0 rely) relx) !ratio_rendu

(*Fonction prenant 4 points et un poly incrit dans ces 4 points, et rendant les coordonées du poly qui en découle.*)
let rec displace_shape encadrement shape =
match shape with
| [] -> []
| hd::tl -> (inttuple (displacement encadrement hd) :: displace_shape encadrement tl)

let render_char encadrement charac = fill_poly (Array.of_list (displace_shape encadrement (shape_char charac)))

let rec render_characs str (x0, y0) l_char h_char l_space shake =
  match str with
  | [] -> ()
  | hd::tl -> (
    render_char [(x0 +. (randfloat ~-.shake shake),           y0 +. (randfloat ~-.shake shake));
                 (x0 +. (randfloat ~-.shake shake) +. l_char, y0 +. (randfloat ~-.shake shake));
                 (x0 +. (randfloat ~-.shake shake) +. l_char, y0 +. (randfloat ~-.shake shake) +. h_char);
                 (x0 +. (randfloat ~-.shake shake),           y0 +. (randfloat ~-.shake shake) +. h_char)] hd;
    render_characs tl (x0 +. l_char +. l_space, y0) l_char h_char l_space shake
    )

(*Fonction trouvée sur stackoverflow pour pouvoir transformer une string en liste de charactères*)
let rec list_car charac = match charac with
    | "" -> []
    | ch -> (String.get ch 0 ) :: (list_car (String.sub ch 1 ( (String.length ch)-1) ) )  ;;

let render_string str pos l_char h_char l_space shake= (render_characs (list_car str) pos l_char h_char l_space shake)


(*L'effet de scanlines a pour but d'imiter les anciens écrans CRT,
qui projetaient l'image ligne par ligne.*)
let rec render_scanlines nb=
  set_color black;
  set_line_width 0;
  if nb < height then (
  moveto 0 nb;
  lineto width nb;
  render_scanlines (nb + scanlines_period))

(*Rendu de cœur*)
let draw_heart (x0,y0) (x1,y1) =
  let (x0,y0) = multuple (x0,y0) !ratio_rendu and (x1,y1) = multuple (x1,y1) !ratio_rendu in
  set_color red;
  let quartx = (x1 -. x0)/. 4. and tiery = (y1 -. y0) /. 3. in
  fill_ellipse (int_of_float (x0 +. quartx +. 0.5)) (int_of_float (y1 -. tiery)) (int_of_float (quartx +. 0.5)) (int_of_float (tiery +. 0.5));
  fill_ellipse (int_of_float (x1 -. quartx +. 0.5)) (int_of_float (y1 -. tiery)) (int_of_float (quartx +. 0.5)) (int_of_float (tiery +. 0.5));
  let decal = 1. -. (1. /. (sqrt 2.)) in
  fill_poly (Array.of_list
    [(inttuple (x0 +. 2. *. quartx, y0));
     (inttuple (x0 +. (decal *. quartx +. 0.5), y0 +. ((1. +. decal) *. tiery)));
     (inttuple (x0 +. 2. *. quartx, y1 -. tiery ));
     (inttuple (x1 -. (decal *. quartx +. 0.5), y0 +. ((1. +. decal) *. tiery)))])

let rec draw_n_hearts lastx n =
  if n > 0 then (
  set_line_width 2;
  draw_heart (lastx -. 0.03 *. !phys_width, 0.75 *. !phys_height) (lastx, 0.80 *. !phys_height);
  draw_n_hearts (lastx -. 0.05  *. !phys_width) (n-1));;

(*Affichage de l'interface utilisateur*)
let affiche_hud ref_etat =
  let etat = !ref_etat in
  let ship = !(etat.ref_ship) in
  if not !retro && not !pause then (
    (*Affichage des cœurs*)
    draw_n_hearts (0.95 *. !phys_width) etat.lifes;
    (*Affichage de la vie*)
    etat.last_health <- (max 0. ship.health) +. (exp_decay (etat.last_health -. (max 0. ship.health)) 0.5);
    set_line_width 0;
    affiche_barre 1. [(0.95,0.9);(0.95,0.85);(0.6,0.85);(0.55,0.9)] (rgb 32 0 0);
    affiche_barre (etat.last_health /. ship_max_health) [(0.95,0.9);(0.95,0.85);(0.6,0.85);(0.55,0.9)] (rgb 255 128 0);
    affiche_barre ((max 0. ship.health) /. ship_max_health) [(0.95,0.9);(0.95,0.85);(0.6,0.85);(0.55,0.9)] red;
    set_line_width buttonframewidth; set_color buttonframe;
    draw_poly (Array.of_list (relative_poly[(0.95,0.9);(0.95,0.85);(0.6,0.85);(0.55,0.9)]));
    (*Affichage du cooldown de téléportation*)
    set_line_width 0;
    affiche_barre 1. [(0.95,0.7);(0.95,0.65);(0.8,0.65);(0.75,0.7)] (rgb 0 0 32);
    affiche_barre ((cooldown_tp -. (max 0. etat.cooldown_tp)) /. cooldown_tp) [(0.95,0.7);(0.95,0.65);(0.8,0.65);(0.75,0.7)] (rgb 0 192 255);
    set_line_width buttonframewidth; set_color buttonframe;
    draw_poly (Array.of_list (relative_poly[(0.95,0.7);(0.95,0.65);(0.8,0.65);(0.75,0.7)]));
    if etat.cooldown_tp <= 0. then (
      set_line_width 0;
      set_color (rgb 0 192 255);
      render_char
      [(0.7  *. !phys_width,0.65 *. !phys_height);
       (0.72 *. !phys_width,0.65 *. !phys_height);
       (0.72 *. !phys_width,0.7  *. !phys_height);
       (0.7  *. !phys_width,0.7  *. !phys_height)]
        'F');
    (*Affichage du cooldown de l'arme*)
    set_line_width 0;
    affiche_barre 1. [(0.95,0.6);(0.95,0.55);(0.9,0.55);(0.85,0.6)] (rgb 32 16 0);
    affiche_barre (max 0.((!projectile_cooldown -. (max 0. etat.cooldown)) /. !projectile_cooldown))[(0.95,0.6);(0.95,0.55);(0.9,0.55);(0.85,0.6)] yellow;
    set_line_width buttonframewidth; set_color buttonframe;
    draw_poly (Array.of_list (relative_poly[(0.95,0.6);(0.95,0.55);(0.9,0.55);(0.85,0.6)]));
    (*Affichage du score*)
    set_color (rgb_of_hdr (intensify {r=10000.;v=1000.;b=300.} (1. /. (1. +. 0.9 *. !shake_score))));
    set_line_width 0;
    render_string ("SCORE " ^ string_of_int etat.score) (*(string_of_int etat.score)*)
      (0.02 *. !phys_width, 0.82 *. !phys_height *. (1. -. (0.05 *. !shake_score *.0.08)))
      ((1. +. 0.05 *. !shake_score) *.0.03 *. !phys_width)
      ((1. +. 0.05 *. !shake_score) *.0.08 *. !phys_height)
      ((1. +. 0.05 *. !shake_score) *.0.01 *. !phys_width) (!shake_score *. 7.);
    (*Affichage du niveau de difficulté*)
    set_color white ; set_line_width 0;
    render_string ("STAGE " ^ (string_of_int etat.stage))
      (0.02 *. !phys_width, 0.2 *. !phys_height)
      (0.02 *. !phys_width) (0.05 *. !phys_height) (0.01 *. !phys_width)
      0.;
  );


  if !scanlines then (
    if animated_scanlines then
      (render_scanlines (0 + !scanlines_offset);scanlines_offset := (!scanlines_offset + 1) mod  scanlines_period)
    else
      render_scanlines 0);

  if !pause then (
    List.iter applique_button !ref_etat.buttons;
    set_color white ; set_line_width 0;
    render_string ("ASTEROIDS")
      ((2./.16.) *. !phys_width, (15./.24.) *. !phys_height)
      ((1./.16.) *. !phys_width) (4. /. 24. *. !phys_height) ((1. /. 40.)*. !phys_width)
      0.;
  );

  (*Calcul du framerate toutes les secondes*)
  if (!time_current_count -. !time_last_count > 1.) then (
    last_count := !current_count;
    current_count := 0;
    time_last_count := !time_current_count;);
  time_current_count := Unix.gettimeofday ();
  current_count := !current_count + 1;
  (*Affichage du framerate en bas à gauche.*)
  moveto 0 0;
  set_color white;
  draw_string (string_of_int !last_count)


let affiche_etat ref_etat =
  let etat = !ref_etat in
  (*On actualise la caméra en fonction du vaisseau.
  Dans les faits on bouge les objets, mais tous de la même valeur donc pas de réel impact*)
    (*On calcule les déplacements de la caméra pour le rendu de caméra dynamique*)
    let ship = !(etat.ref_ship) in
    let (next_x, next_y) =
      addtuple (polar_to_affine ship.orientation (!phys_width *. camera_ratio_vision)) (
      addtuple
        (addtuple ship.position (multuple ship.velocity camera_prediction))
        (multuple (center_of_attention
          (List.append (List.append etat.ref_objets etat.ref_objets_unspawned) (List.append etat.ref_fragments etat.ref_fragments_unspawned))
          ship.position)
          camera_ratio_objects)) in
    (*move_camera décrit plutôt un déplacement de la totalité des objets en jeu.*)
    let move_camera =
      (((!phys_width /. 2.) -. next_x) -. (exp_decay ((!phys_width /. 2.) -. next_x) camera_half_depl),
       ((!phys_height/. 2.) -. next_y) -. (exp_decay ((!phys_height/. 2.) -. next_y) camera_half_depl)) in

    if not !pause then ignore (deplac_stars etat.ref_stars move_camera);
    deplac_objet_abso etat.ref_ship move_camera;
    deplac_objets_abso etat.ref_objets move_camera;
    deplac_objets_abso etat.ref_objets_unspawned move_camera;
    deplac_objets_abso etat.ref_fragments move_camera;
    deplac_objets_abso etat.ref_fragments_unspawned move_camera;
    deplac_objets_abso etat.ref_chunks move_camera;
    deplac_objets_abso etat.ref_projectiles move_camera;
    deplac_objets_abso etat.ref_explosions move_camera;
    deplac_objets_abso etat.ref_smoke move_camera;

  (*Fond d'espace*)
  if !retro then set_color black else set_color (rgb_of_hdr (intensify space_color !game_exposure));
  fill_rect 0 ~-1 width height;
  (*On calcule éventuellement le motion blur si il est activé*)
  if !motion_blur && not !retro then (
    List.iter render_motion_blur etat.ref_fragments_unspawned;
    List.iter render_motion_blur etat.ref_fragments;
    List.iter render_motion_blur etat.ref_objets_unspawned;
    List.iter render_motion_blur etat.ref_objets;
    set_line_width 2;
    List.iter render_star_trail etat.ref_stars;(*On rend les étoiles derrière la fumée, mais derrière les autres objets moins lumineux.*)
    (*List.iter render_motion_blur etat.ref_smoke;*)(*TODO régler le fait que le blur soit appliqué ou non de manière erratique.*)
  )else if not !retro then (set_line_width 2; List.iter render_star_trail etat.ref_stars);(*Avec ou sans motion blur, on rend les étoiles comme il faut*)
  set_line_width 0;

  if not !oldschool then (List.iter render_chunk etat.ref_chunks;
  List.iter render_objet etat.ref_smoke);
  List.iter render_projectile etat.ref_projectiles;
  render_objet etat.ref_ship;
  List.iter render_unspawned etat.ref_fragments_unspawned;
  List.iter render_objet etat.ref_fragments;
  List.iter render_unspawned etat.ref_objets_unspawned;
  List.iter render_objet etat.ref_objets;
  List.iter render_objet etat.ref_explosions;

  affiche_hud ref_etat;
  synchronize ()


(********************************************************************************************************************)
(*WHERE THE MAGIC HAPPENS*)
(********************************************************************************************************************)
(* calcul de l'etat suivant, apres un pas de temps *)
let etat_suivant ref_etat =
  if !quit then (print_endline "Bye bye!"; exit 0);
  if !restart then (
    ref_etat := init_etat ();
    game_exposure := 0.;
    restart := false;
    pause := false
  );
  let etat = !ref_etat in
  if !pause then (
    game_speed_target := game_speed_target_pause;
    game_speed := game_speed_target_pause
  );
  if !oldschool
    then (
      retro := true;
      projectile_number := 1;
      fragment_number := 2;
      stars_nb := 0;
      screenshake := false;
      dynamic_camera := false;
      infinitespace := false;
      smoke := false;
      motion_blur := false;
      mousecontrol := false)
    else (
      stars_nb := stars_nb_default;
      projectile_number := projectile_number_default;
      fragment_number := 5);
  if !infinitespace then dynamic_camera := true;

  if !stars_nb != !stars_nb_previous then (etat.ref_stars <- n_stars !stars_nb; stars_nb_previous := !stars_nb);

  (*On calcule le changement de vitesse naturel du jeu. Basé sur le temps réel et non le temps ingame pour éviter les casi-freeze*)
  game_speed := !game_speed_target +. abso_exp_decay (!game_speed -. !game_speed_target) half_speed_change;


  (*On calcule le jitter, pour l'appliquer de manière uniforme sur tous les objets et tous les rayons.*)
  current_jitter_double := (Random.float dither_power, Random.float dither_power);

  if not !pause then (
    (*On calcule la puissance du screenshake. Basé sur le temps en jeu. (Sauf si le jeu est en pause, auquel cas on actualise plus)*)
    game_screenshake := exp_decay !game_screenshake screenshake_half_life;
    (*On calcule l'emplacement caméra pour le screenshake,
    en mémorisant l'emplacement précédent du screenshake (Pour le rendu correct des trainées de lumière et du flou)*)
    game_screenshake_previous_pos := !game_screenshake_pos;
    if !screenshake then game_screenshake_pos := (!game_screenshake *. ((Random.float 2.) -. 1.), !game_screenshake *. ((Random.float 2.) -. 1.));
    (*Dans le cas du lissage de screenshake, on fait une moyenne entre le précédent et l'actuel, pour un lissage du mouvement*)
    if screenshake_smooth then game_screenshake_pos := moytuple !game_screenshake_previous_pos !game_screenshake_pos screenshake_smoothness;
    (*On calcule le changement d'exposition du jeu. Basé sur le temps en jeu *)
    game_exposure := !game_exposure_target +. exp_decay (!game_exposure -. !game_exposure_target) exposure_half_life;
    (*On calcule le changement de flashes du jeu. Basé sur le temps en jeu *)
    add_color := intensify !add_color (exp_decay 1. flashes_half_life);
    (*On calcule le changement de filtre du jeu. Basé sur le temps en jeu *)
    mul_color := half_color !mul_color !mul_base filter_half_life
  );

  (*On calcule tous les déplacements naturels dus à l'inertie des objets*)
  time_last_frame := !time_current_frame;
  time_current_frame := Unix.gettimeofday ();

  observer_proper_time := !(etat.ref_ship).proper_time;

  inertie_objet etat.ref_ship;
  inertie_objets etat.ref_objets;
  inertie_objets etat.ref_objets_unspawned;
  inertie_objets etat.ref_fragments;
  inertie_objets etat.ref_fragments_unspawned;
  inertie_objets etat.ref_chunks;
  inertie_objets etat.ref_projectiles;
  inertie_objets etat.ref_smoke;

  moment_objet etat.ref_ship;
  moment_objets etat.ref_objets;
  moment_objets etat.ref_objets_unspawned;
  moment_objets etat.ref_fragments;
  moment_objets etat.ref_fragments_unspawned;
  (*Inutile de calculer le moment des projectiles, explosions ou fumée, comme leur rotation n'a aucune importance*)

(* On ne rapproche plus les objets
  (*On rapproche les objets non spawnés*)
  List.iter closer_objet etat.ref_objets_unspawned;
  List.iter closer_objet etat.ref_fragments_unspawned;

    (*On rapproche les objets spawnés*)
  List.iter closer_objet etat.ref_objets;
  List.iter closer_objet etat.ref_fragments;
*)

  (*On calcule la friction et friction angulaire des objets*)
  friction_objet etat.ref_ship;
  friction_moment_objet etat.ref_ship;

  (*Collisions entre le vaisseau et les objets*)
  calculate_collisions_modulo etat.ref_ship etat.ref_objets;
  (*Collisions entre le vaisseau et les objets «non spwanés»*)
  calculate_collisions_objet etat.ref_ship etat.ref_objets_unspawned;
  (*Collisions entre le vaisseau et les fragments*)
  calculate_collisions_modulo etat.ref_ship etat.ref_fragments;
  (*Collisions entre le vaisseau et les fragments non spawnés*)
  calculate_collisions_objet etat.ref_ship etat.ref_fragments_unspawned;

  (*Collisions entre projectiles et objets*)
  calculate_collisions_modulo_listes etat.ref_projectiles etat.ref_objets;
  (*Collisions entre projectiles et objets «non spawnés» - non modulo*)
  calculate_collisions_listes_objets etat.ref_projectiles etat.ref_objets_unspawned;
  (*Collisions entre les projectiles et les fragments*)
  calculate_collisions_modulo_listes etat.ref_projectiles etat.ref_fragments;
  (*Collisions entre les projectiles et les fragments non spawnés (non modulo)*)
  calculate_collisions_listes_objets etat.ref_projectiles etat.ref_fragments_unspawned;

  (*Collisions entre explosions et objets*)
  calculate_collisions_modulo_listes etat.ref_explosions etat.ref_objets;
  (*Collisions entre explosions et objets «non spawnés» - non modulo*)
  calculate_collisions_listes_objets etat.ref_explosions etat.ref_objets_unspawned;
  (*Collisions entre explosions et les fragments*)
  calculate_collisions_modulo_listes etat.ref_explosions etat.ref_fragments;
  (*Collisions entre explosions et fragments non spawnés*)
  calculate_collisions_listes_objets etat.ref_explosions etat.ref_fragments_unspawned;

  (*Collisions entre objets*)
  calculate_collisions_modulos etat.ref_objets;
  (*Collisions entre objets unspawned*)
  calculate_collisions_modulos etat.ref_objets_unspawned;
  (*Collisions entre objets spawnés et «non spawnés» - modulo pour le coup*)
  calculate_collisions_modulo_listes etat.ref_objets etat.ref_objets_unspawned;
  (*Collisions entre objets et fragments*)
  calculate_collisions_modulo_listes etat.ref_objets etat.ref_fragments;
  (*Collisions entre objets et fragments non spawnés*)
  calculate_collisions_listes_objets etat.ref_objets etat.ref_fragments_unspawned;

  (*Les explosions sont ajoutées à la fumée, et la fumée précédente avec decay. Uniquement si smoke = true.*)
  if !smoke then etat.ref_smoke <- List.append (List.map decay_smoke etat.ref_smoke) etat.ref_explosions else etat.ref_smoke <- [];
  (*On fait apparaitre les explosions correspondant aux projectiles détruits*)
  etat.ref_explosions <- List.map spawn_explosion (List.filter is_dead etat.ref_projectiles);

  (*On fait apparaitre les explosions correspondant aux objets détruits,
  sauf en mode oldschool*)
  if not ! oldschool then (
  etat.ref_explosions <- List.append etat.ref_explosions (List.map spawn_explosion_object (List.filter is_dead etat.ref_objets));
  etat.ref_explosions <- List.append etat.ref_explosions (List.map spawn_explosion_object (List.filter is_dead etat.ref_objets_unspawned)));
  (*On ne fait pas exploser les fragments, car ils sont tous superposés, et en quelques frames ils meurent tous.
  On fait par contre entrer les explosions dans les effets de fumée, pour l'effet visuel.*)
  if !smoke then(
    etat.ref_smoke <- List.append etat.ref_smoke (List.map spawn_explosion_object (List.filter is_dead etat.ref_fragments));
    etat.ref_smoke <- List.append etat.ref_smoke (List.map spawn_explosion_object (List.filter is_dead etat.ref_fragments_unspawned)));(*Le vaisseau génère aussi une trainée d'explosions après sa mort*)
  if (is_dead etat.ref_ship) then etat.ref_explosions <- (spawn_explosion etat.ref_ship) :: etat.ref_explosions;

  (*On ralentit le temps selon le nombre d'explosions*)
  game_speed := !game_speed *. ratio_time_explosion ** (float_of_int (List.length etat.ref_explosions));

  (*Fonction permettant de spawner un nombre de fragments d'astéroïde*)
  let rec spawn_n_frags ref_source ref_dest n = (
    if n=0 then ref_dest else List.append (spawn_n_frags ref_source ref_dest (n-1)) (List.map frag_asteroid (List.filter is_dead ref_source))
) in

  (*On fait apparaitre les fragments des astéroïdes détruits*)
  etat.ref_fragments <- spawn_n_frags etat.ref_objets etat.ref_fragments !fragment_number;
  (*Pareil pour les astéroïdes «non spawnés»*)
  etat.ref_fragments_unspawned <- spawn_n_frags etat.ref_objets_unspawned etat.ref_fragments_unspawned !fragment_number;
  (*Pareil pour les fragments déjà cassés*)
  etat.ref_fragments <- spawn_n_frags etat.ref_fragments etat.ref_fragments !fragment_number;
  (*Pareil pour les fragments unspawned déjà cassés*)
  etat.ref_fragments_unspawned <- spawn_n_frags etat.ref_fragments_unspawned etat.ref_fragments_unspawned !fragment_number;

  (*On ralentit le temps selon le nombre d'astéroïde détruits*)
  (*TODO : Baser le ralentissement de temps non sur la quantité d'astéroïdes détruits et le nombre d'explosions,
  mais selon les dégats et dégats physiques, comme pour le screenshake.*)
  let nb_destroyed = List.length (List.filter is_dead etat.ref_objets) + List.length (List.filter is_dead etat.ref_objets_unspawned) + List.length (List.filter is_dead etat.ref_fragments)in
  game_speed := !game_speed *. ratio_time_destr_asteroid ** (float_of_int nb_destroyed);
  etat.score <- etat.score + nb_destroyed; (*TODO meilleure version du score avec multiplicateurs*)
  shake_score := (exp_decay !shake_score shake_score_half_life) +. shake_score_ratio *. (float_of_int nb_destroyed);

  (*On transfère les fragments qui ne sont pas en collision avec les autres dans les objets physiques.
  On les considère comme unspawned pour éviter qu'ils ne se téléportent dans l'écran de jeu alors qu'ils sont hors de l'écran*)
  etat.ref_objets <- List.append (no_collisions_liste etat.ref_fragments) etat.ref_objets ;
  etat.ref_objets_unspawned <- List.append (no_collisions_liste etat.ref_fragments_unspawned) etat.ref_objets_unspawned ;

  etat.ref_fragments <- collisions_sein_liste etat.ref_fragments;
  etat.ref_fragments_unspawned <- collisions_sein_liste etat.ref_fragments_unspawned;

  (*Gestion des vagues d'astéroïdes, ou «niveaux».
  On fait apparaitre la vague suivante quand tous les objets en jeu sont sur l'écran du joueur.
  On ne tient pas compte des fragments parce qu'il y a beaucoup plus de chances qu'ils se «perdent»*)
  if List.length etat.ref_objets_unspawned = 0
    then (
      etat.stage <- etat.stage + 1;
      etat.ref_objets_unspawned <- spawn_n_asteroid etat.stage (asteroid_min_nb + asteroid_stage_nb * etat.stage)
    );

  (*Recentrage des objets sortis de l'écran.
  Ne pas appeler en infinitespace*)
  if not !infinitespace then (recenter_objet etat.ref_ship;
  List.iter recenter_objet etat.ref_chunks;
  List.iter recenter_objet etat.ref_objets;
  List.iter recenter_objet etat.ref_fragments);
  (*Recentrage des objets unspawned*)
  List.iter recenter_objet_unspawned etat.ref_objets_unspawned;
  List.iter recenter_objet_unspawned etat.ref_fragments_unspawned;
  if !infinitespace then(
    List.iter recenter_objet_unspawned etat.ref_objets;
    List.iter recenter_objet_unspawned etat.ref_fragments
  );
  (*On ne recentre pas les projectiles car ils doivent despawner une fois sortis de l'espace de jeu*)

  let elapsed_time = !time_current_frame -. !time_last_frame in
  (*On diminue le cooldown en fonction du temps passé depuis la dernière frame.*)
  (*On laisse si le cooldown est négatif, cela veut dire qu'un projectile a été tiré trop tard,
  et ce sera compensé par un projectile tiré trop tôt, afin d'équilibrer.*)
  if etat.cooldown > 0. then etat.cooldown <- etat.cooldown -. !game_speed *. elapsed_time;
  if etat.cooldown_tp > 0. then etat.cooldown_tp <- etat.cooldown_tp -. !game_speed *. elapsed_time;
  ref_etat := etat;
  (*Suppression des objets qu'il faut*)
  despawn ref_etat;
  (*On spawne ce qui doit spawner*)
  checkspawn_etat ref_etat;
  affiche_etat ref_etat;
  (*Équivalent bidouillé de sleepf en millisecondes, pour que le programme fonctionne aussi avec les anciennes versions d'Ocaml*)
  (*TODO trouver pourquoi ça ne marche pas sur les systèmes linux. Penser à le régler avant de le rendre.*)
  if !locked_framerate then ignore (Unix.select [] [] [] (max 0. ((1. /. framerate_limit) -. elapsed_time)));;
  (*ne marche pas sur linux*)


(* acceleration du vaisseau *)
let acceleration ref_etat =
  let etat = !ref_etat in
  let orientation = !(etat.ref_ship).orientation in
if !ship_direct_pos then
  deplac_objet etat.ref_ship (polar_to_affine orientation ship_max_depl)
else
  (*Dans le cas d'un contrôle de la vélocité et non de la position.*)
  (*C'est à dire en respectant le TP, et c'est bien mieux en terme d'expérience de jeu :) *)
  accel_objet etat.ref_ship (polar_to_affine orientation ship_max_accel);
(*Feu à l'arrière du vaisseau. Spawne à chaque frame
plus de frames = plus de particules, pertes de perf = moins de particules.*)
if !(etat.ref_ship).health > 0. then (let list_fire = [spawn_fire etat.ref_ship;spawn_fire etat.ref_ship] in
if !smoke then etat.ref_smoke <- etat.ref_smoke @ list_fire;);
ref_etat:=etat;
etat_suivant ref_etat;;

(* boost du vaisseau, pour contrôle clavier *)
let boost ref_etat =
  let etat = !ref_etat in
  let orientation = !(etat.ref_ship).orientation in
  (*Dans le cas d'un contrôle de la vélocité et non de la position.*)
  (*C'est à dire en respectant le TP, et c'est bien mieux en terme d'expérience de jeu :) *)
  boost_objet etat.ref_ship (polar_to_affine orientation ship_max_boost);
(*Feu à l'arrière du vaisseau. Spawne spawn plusieurs particules à la fois pour le boost*)
let list_fire1 = [spawn_fire etat.ref_ship;spawn_fire etat.ref_ship;spawn_fire etat.ref_ship] in
let list_fire2 = [spawn_fire etat.ref_ship;spawn_fire etat.ref_ship;spawn_fire etat.ref_ship] in
let list_fire3 = [spawn_fire etat.ref_ship;spawn_fire etat.ref_ship;spawn_fire etat.ref_ship] in
etat.ref_smoke <- etat.ref_smoke @ list_fire1 @ list_fire2 @ list_fire3;
ref_etat:=etat;
etat_suivant ref_etat;;


(* rotation vers la gauche et vers la droite du ship *)
let rotation_gauche ref_etat =
if !ship_direct_rotat then
  rotat_objet !ref_etat.ref_ship ship_max_tourn
else(*Dans le cas d'un contrôle de la du couple et non de la rotation. Non recommandé de manière générale*)
  couple_objet !ref_etat.ref_ship ship_max_tourn;
etat_suivant ref_etat;;

let rotation_droite ref_etat =
if !ship_direct_rotat then
  rotat_objet !ref_etat.ref_ship (0. -. ship_max_tourn)
else(*Dans le cas d'un contrôle de la du couple et non de la rotation. Non recommandé de manière générale*)
  couple_objet !ref_etat.ref_ship (0. -. ship_max_tourn);
etat_suivant ref_etat;;


(* rotation vers la gauche et vers la droite du ship *)
let boost_gauche ref_etat =
if !ship_direct_rotat then
  tourn_objet !ref_etat.ref_ship (0. +. ship_max_rotat)
else(*Dans le cas d'un contrôle de la du couple et non de la rotation. Non recommandé de manière générale*)
  couple_objet_boost !ref_etat.ref_ship ship_max_tourn_boost;
etat_suivant ref_etat;;

let boost_droite ref_etat =
if !ship_direct_rotat then
  tourn_objet !ref_etat.ref_ship (0. -. ship_max_rotat)
else(*Dans le cas d'un contrôle de la du couple et non de la rotation. Non recommandé de manière générale*)
  couple_objet_boost !ref_etat.ref_ship (0. -. ship_max_tourn_boost);
etat_suivant ref_etat;;


(* Boost de côté, pour un meilleur contrôle clavier *)
let strafe_left ref_etat =
  let etat = !ref_etat in
  let orientation = !(etat.ref_ship).orientation +. (pi /. 2.) in
  boost_objet etat.ref_ship (polar_to_affine orientation ship_max_boost);
ref_etat:=etat;
etat_suivant ref_etat;;

(* Boost de côté, pour un meilleur contrôle clavier *)
let strafe_right ref_etat =
  let etat = !ref_etat in
  let orientation = !(etat.ref_ship).orientation -. (pi /. 2.) in
  boost_objet etat.ref_ship (polar_to_affine orientation ship_max_boost);
ref_etat:=etat;
etat_suivant ref_etat

(* tir d'un nouveau projectile *)
let tir ref_etat =
(*Tant que le cooldown est supérieur à 0, on ne tire pas.*)
(*Sauf si le temps que la prochaine frame arrive justifie qu'on puisse tirer entre temps*)
(*Plus le cooldown est faible, plus le tir devrait arriver tôt*)
(*Donc on laisse le hasard décider si le tir spawne maintenant ou à la frame suivante.*)
(*On considère que le temps de la prochaine frame sera celui de la dernière,
ce qui est une approximation généralement correcte*)
  let etat = !ref_etat in
  let ship = !(etat.ref_ship) in
  while etat.cooldown <= 0.
  do
    if !flashes then add_color := hdr_add !add_color (intensify {r=100.;v=50.;b=25.} flashes_tir);
    if variable_exposure then game_exposure := !game_exposure *. exposure_tir;
    game_screenshake := !game_screenshake +. screenshake_tir_ratio;
    (*On ajoute les projectiles *)
    etat.ref_projectiles <- List.append (spawn_n_projectiles ship !projectile_number) etat.ref_projectiles;
    (*Ajout du muzzleflash correspondant aux tirs*)
    if !smoke && not !oldschool then etat.ref_smoke <- List.append etat.ref_smoke (List.map spawn_muzzle (spawn_n_projectiles ship !projectile_number));
    etat.cooldown <- etat.cooldown +. !projectile_cooldown;
    ship.velocity <- addtuple ship.velocity (polar_to_affine (ship.orientation +. pi) !projectile_recoil)
  done;
  etat.ref_ship <- ref ship;
  ref_etat := etat;
  etat_suivant ref_etat

let random_teleport ref_etat =
  let etat = !ref_etat in
  if etat.cooldown_tp <= 0. then (
    if !flashes then add_color := hdr_add !add_color (intensify {r=6.25;v=50.;b=400.} flashes_teleport);
    game_exposure := game_exposure_target_tp;
    let ship = !(etat.ref_ship) in
    ship.position <- (Random.float !phys_width, Random.float !phys_height);
    ship.velocity <- (0.,0.);
    etat.ref_ship := ship;
    etat.ref_explosions <- (spawn_explosion_tp ship) :: etat.ref_explosions;
    etat.cooldown_tp <- etat.cooldown_tp +. cooldown_tp;
    ref_etat:=etat)


(*Fonction  de contrôle souris*)
let controle_souris ref_etat =
  let etat = !ref_etat in
  let ship = !(etat.ref_ship) in
  let status = wait_next_event[Poll] in
  let (xv,yv) = ship.position in
  let (theta, r) =
    affine_to_polar
      ((float_of_int status.mouse_x) /. !ratio_rendu -. xv,
      (float_of_int status.mouse_y) /. !ratio_rendu -. yv) in
  ship.orientation <- theta;
  etat.ref_ship :=  ship;
  ref_etat := etat;
  if status.button && not !pause then acceleration ref_etat else ();;


(*État une fois mort*)
let rec mort ref_etat =
  game_speed_target := game_speed_target_death;
  game_exposure_target := game_exposure_target_death;
  if not !oldschool then (
    acceleration ref_etat;
    !(!ref_etat.ref_ship).mass <- 100000.
  );
  etat_suivant ref_etat;
  if (Unix.gettimeofday () < !time_of_death +. time_stay_dead) then (
    if !mousecontrol then controle_souris ref_etat;
    if key_pressed  ()then (
      let status = wait_next_event[Key_pressed] in
        match status.key  with (* ...en fonction de la touche frappee *)
        | 'r' -> ref_etat := init_etat ()(*R permet de recommencer une partie de zéro rapidement.*)
        | 'p' -> pause := not !pause
        | 'k' -> print_endline "Bye bye!"; exit 0 (* on quitte le jeu *)
        | _ -> mort ref_etat)
    else mort ref_etat)
  else (
  if (!ref_etat).lifes = 0 then (ref_etat := init_etat ();pause := true) else (
  !ref_etat.ref_ship <- ref (spawn_ship ());
  game_speed_target := game_speed_target_boucle;
  game_exposure_target := game_exposure_target_boucle))

(* --- boucle d'interaction --- *)

let rec boucle_interaction ref_etat =
  game_speed_target := game_speed_target_boucle;
  game_exposure_target := game_exposure_target_boucle;

  if !(!ref_etat.ref_ship).health<0. then (
    time_of_death := Unix.gettimeofday ();
    (!ref_etat).lifes <- (!ref_etat).lifes - 1;
    mort ref_etat;
  );
  if !mousecontrol then controle_souris ref_etat;
  if key_pressed () && not !pause then
  let status = wait_next_event[Key_pressed] in
    match status.key  with (* ...en fonction de la touche frappee *)
    | 'r' -> ref_etat := init_etat (); pause:=false (*R permet de recommencer une partie de zéro rapidement.*)
    | 'a' -> strafe_left ref_etat; boucle_interaction ref_etat (*strafe vers la gauche *)
    | 'q' -> if !ship_impulse_pos then boost_gauche ref_etat else rotation_gauche ref_etat; boucle_interaction ref_etat (* rotation vers la gauche *)
    | 'z' -> if !ship_impulse_pos then boost ref_etat else acceleration ref_etat; boucle_interaction ref_etat (* acceleration vers l'avant *)
    | 'd' -> if !ship_impulse_pos then boost_droite ref_etat else rotation_droite ref_etat; boucle_interaction ref_etat (* rotation vers la gauche *)
    | 'e' -> strafe_right ref_etat; boucle_interaction ref_etat (*strafe vers la droite *)
    | 'f' -> random_teleport ref_etat; boucle_interaction ref_etat
    | ' ' -> tir ref_etat;boucle_interaction ref_etat (* tir d'un projectile *)
    | 'p' -> pause := not !pause
    | 'k' -> print_endline "Bye bye!"; exit 0 (* on quitte le jeu *)
    | _ -> etat_suivant ref_etat;boucle_interaction ref_etat
 else if key_pressed() then (
let status = wait_next_event[Key_pressed] in
 match status.key  with (* ...en fonction de la touche frappee *)
   | 'r' -> ref_etat := init_etat (); pause:=false (*R permet de recommencer une partie de zéro rapidement.*)
   | 'p' -> pause := not !pause
   | 'k' -> print_endline "Bye bye!"; exit 0 (* on quitte le jeu *)
   | _ -> etat_suivant ref_etat;boucle_interaction ref_etat
 )else
  etat_suivant ref_etat;
  boucle_interaction ref_etat;;

(* --- fonction principale --- *)

let main () =
  Random.self_init ();
  open_graph (" " ^ string_of_int width ^ "x" ^ string_of_int height);
  auto_synchronize false;
(*set_text_size ne semble être implémenté correctement sur aucun système, est-ce depuis de nombreuses années. On fera sans.*)
(*set_text_size (int_of_float (10. *. !ratio_rendu));*)

  (* initialisation de l'etat du jeu *)
  let ref_etat = ref (init_etat ()) in

(*On s'assure d'avoir un repère temporel correct*)
  time_last_frame := Unix.gettimeofday();
  time_current_frame := Unix.gettimeofday();
  etat_suivant ref_etat;
  affiche_etat ref_etat;
  boucle_interaction ref_etat;; (* lancer la boucle d'interaction avec le joueur *)

let _ = main ();; (* demarrer le jeu *)
