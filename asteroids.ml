(*spécifique à windows*)
(*
#load "unix.cma";;

#load "graphics.cma";;
*)


open Graphics;;


(*Certaines valeurs par défaut ne suivent pas les instructions du tp pour une meilleure expérience de jeu.*)
(*Ces changements sont documentés dans les commentaires et peuvent être remis aux valeurs du pdf si nécessaire.*)



(******************************************************************************)
(*Paramètres affichage*)


(*Paramètres temporels*)

(*Le game_speed_target est la vitesse à laquelle on veut que le jeu tourne en temps normal*)
let game_speed_target = ref 1. ;;
(*Le game_speed est la vitesse réelle à laquelle le jeu tourne à l'heure actuelle.*)
(*Cela permet notamment de faire des effets de ralenti ou d'accéléré*)
let game_speed = ref 1.;;
(*Le temps propre de l'observateur. *)
(*En l'occurrence, on récupère celui du vaisseau.*)
(*Cela permet d'avoir une relativité Einsteinienne.*)
let observer_proper_time = ref 1.0;;(*En ratio du temps «absolu» de l'univers*)

(*Le half_speed_change détermine à quelle «vitesse» le game speed se rapproche de game_speed_target (En demi-vie) *)
let half_speed_change = 0.2;;
(*Ratios de changement de vitesse en fonction des évènements*)
let ratio_time_explosion = 0.95;;
let ratio_time_destr_asteroid = 0.6;;

(*Le framerate demandé dans l'énoncé est de 20*)
(*Un framerate plus élevé offre une meilleure expérience de jeu :*)
(*Des contrôles plus réactifs, un meilleur confort visuel, et une physique plus précise.*)
(*Bien sûr, il est possible de le changer ci-dessous :)*)
let framerate_limit = 600.;;
(*Il est également possible de désactiver purement et simplement la limitation de framerate*)
let locked_framerate = false;;

(*Le framerate de rendu permet de déterminer la longueur du motion blur*)
(*Et TODO rendre le jeu seulement au framerate de l'écran*)
let framerate_render = 60.;;

(*On stocke le moment auquel la dernière frame a été calculée
pour synchroniser correctement le moment de calcul de la frame suivante*)
let time_last_frame = ref 0.;;
let time_current_frame = ref 0.;;


(*Dimensions fenêtre graphique.*)

let width = 1200;;
let height = 600;;

let game_surface = 1.;;
(*Dimensions de l'espace physique dans lequel les objets évoluent.*)
(*On s'assure que la surface de jeu soit la même quelle que soit la résolution.*)
(*On conserve au passage le ratio de la résolution pour les dimensions de jeu*)
(*On a une surface de jeu de 1 000 000*)
let ratio_rendu = sqrt ((float_of_int width) *. (float_of_int height) /. (game_surface *. 1000000.));;
let phys_width = float_of_int width /. ratio_rendu;;
let phys_height = float_of_int height /. ratio_rendu;;



(******************************************************************************)
(*Paramètres graphiques avancés*)

(*L'antialiasing de jitter fait «trembler» l'espace de rendu.
C'est une forme de dithering spatial
afin de compenser la perte de précision due à la rastérisation
lors du placement des objets et du tracé des contours.*)
let dither_aa = true;;
(*La puissance du jitter détermine à quel point le rendu peut se décaler.*)
(*Déterminer à 1 ou moins pour éviter un effet de flou et de fatigue visuelle*)
let dither_power = 0.5;;(*En ratio de la taille d'un pixel*)

let space_r = 0.;;
let space_g = 0.;;
let space_b = 50.;;

(*Paramètres de flou de mouvement*)
(*TODO l'implémenter*)
let motion_blur = true;;
let shutter_speed = 1.0;;

(******************************************************************************)
(*Paramètres de jeu*)



(*Pi*)
let pi = 4. *. atan 1. ;;

(*Permet le contrôle du vaisseau à la souris. Viser avec la souris, clic droit pour accélérer, clic pour tirer.*)
let mousecontrol = true;;

(*Le mode infinitespace permet de se passer de limites physiques au jeu.*)
let infinitespace = false;;
(*Distance max du ship à laquelle un objet peut exister.*)
(*N'a d'impact sur le jeu qu'en mode infinitespace,*)
(*Dans lequel les objets peuvent continuer d'exister en dehors de l'espace de jeu*)
let objmaxdist = 10.;;

(*Ratio pour conversion des dégats physiques depuis le changement de vélocité au carré*)
let ratio_phys_deg = ref 0.002;;

(*Paramètres des astéroïdes*)

let asteroid_max_spawn_radius = 200.;;(*Taille max d'astéroïde au spawn.*)
let asteroid_min_spawn_radius = 40.;;(*Taille min de spawn*)
let asteroid_min_size = 15.;;(*En dessous de la taille minimale, un asteroide ne se divise pas à sa mort*)
let asteroid_max_moment = 1.;;(*Rotation max d'un astéroïde au spawn (dans un sens aléatoire)*)
let asteroid_max_velocity = 150.;; (*Velocité max au spawn*)
let asteroid_min_velocity = 50.;; (*Velocité min au spawn*)
let asteroid_density = 1.;;(*Sert à déterminer la masse d'un astéroïde en se basant sur sa surface*)
let asteroid_min_health = 20.;; (*Évite les astéroïdes trop fragiles à cause d'une masse trop faible. S'additionne au calcul.*)
let asteroid_mass_health = 0.01;;(*Sert à déterminer la vie d'un astéroïde basé sur sa masse*)

(*Dam : dommmages. phys : dommages physiques. Ratio : Multiplicateur du dégat. res : résistance aux dégats (soustraction)*)
let asteroid_dam_ratio = 1.;;
let asteroid_dam_res = 0.;;
let asteroid_phys_ratio = 1.;;
let asteroid_phys_res = 20.;;
(*Caractéristiques des fragments. Principalement hérité des parents.*)
let fragment_max_velocity = 200.;; (*Velocité max au spawn*)
let fragment_min_velocity = 50.;;  (*Velocité min au spawn*)
let fragment_max_size = 0.7;;(*En ratio de la taille de l'astéroïde parent*)
let fragment_min_size = 0.4;; (*En ratio de la taille de l'astéroïde parent*)
let fragment_min_exposure = 0.5;;
let fragment_max_exposure = 2.0;;


(*Paramètres ship*)

(*Les contrôles directs ne contrôlent pas la vitesse et le moment mais directement la position et la rotation*)
(*Les valeurs par défaut sont celles demandées dans le tp*)
let ship_direct_pos = false;;
let ship_direct_rotat = false;;

(*valeurs du ship*)
let ship_max_health = 100.;; (*health au spawn. Permet de l'appliquer au modèle physique.*)
let ship_max_healths = 3;; (*Nombre de fois que le vaisseau peut réapparaître*)

let ship_max_depl = 50.;; (*En px.s⁻¹. Utile si contrôle direct du déplacement.*)
let ship_max_accel = 1000.;; (*En px.s⁻² Utile si contrôle de l'accélération*)
let ship_max_boost = 100.;; (*En px.s⁻¹. Utile si contrôle par boost.*)
let ship_half_stop =1.;; (*En temps nécessaire pour perdre la moitié de l'inertie*)

let ship_max_tourn = 4.;; (*En radian.s⁻¹*)
let ship_max_moment = 0.5;; (*En radian.s⁻²*)
let ship_max_tourn_boost = 3.;;
let ship_max_rotat = pi /. 6.;;
let ship_half_stop_rotat = 0.2;;(*En temps nécessaire pour perdre la moitié du moment angulaire*)
let ship_density = 15.;; (*Pour calcul de la masse du vaisseau, qui a un impact sur la physique*)
let ship_radius = 15.;; (*Pour la hitbox et le rendu*)
(*Réduction des dégats et dégats physiques*)
let ship_dam_ratio = 0.8;;
let ship_dam_res = 10.;;
let ship_phys_ratio = 1.;;
let ship_phys_res = 5.;;
let ship_death_max_momentum = 2.;;

(*Valeurs du projectile*)
let projectile_recoil = 10.;;
let projectile_cooldown = 0.07;;
let projectile_max_speed = 1400.;;(*Vitesse relative au lanceur lors du lancement*)
let projectile_min_speed = 900.;;
let projectile_deviation = 0.2;;(*Déviation possible de la trajectoire des projectiles*)
let projectile_radius = 15.;;
let projectile_health = 0.;;(*On considère la mort quand la santé descend sous zéro. On a ici la certitude que le projectile se détruira*)

(*Valeurs des explosions*)
let explosion_max_radius = 40.;;
let explosion_min_radius = 20.;;
let explosion_min_exposure = 1.;;(*Détermine la luminosité max et min des explosions au spawn*)
let explosion_max_exposure = 2.;;
let explosion_damages = 6.;;
(*Pour les explosions héritant d'un objet*)
let explosion_ratio_radius = 1.5;;
let explosion_saturate = 6.;;
let explosion_min_exposure_heritate = 30.;;(*Détermine la luminosité max et min des explosions héritant d'objets au spawn*)
let explosion_max_exposure_heritate = 40.;;

(*Valeurs des muzzleflashes*)
let muzzle_ratio_radius = 0.5;;
let muzzle_ratio_speed = 0.01;;

(*Valeurs du feu à l'arrière du vaisseau*)
let fire_max_random = 20.;;
let fire_min_speed = 250.;;
let fire_max_speed = 500.;;
let fire_ratio_radius = 0.5;;

(*Valeurs de la fumée*)
let smoke_half_life = 0.1;; (*Vitesse de la décroissance de la couleur*)
let smoke_radius_decay = 20.;; (*Diminution du rayon des particules de fumée*)
let smoke_max_speed = 40.;;(*Vitesse random dans une direction random de la fumée*)

(*La camera predictive oriente la camera vers l'endroit où le vaisseau va,
pour le garder tant que possible au centre de l'écran*)
let dynamic_camera = true;;
let camera_prediction = 1.5;; (*En secondes de déplacement du vaisseau dans le futur.*)
let camera_half_depl = 2.;; (*Temps pour se déplacer de moitié vers l'objectif de la caméra*)


let screenshake = true;;
let screenshake_dam_ratio = 0.1;;
let screenshake_phys_ratio = 0.05;;
let screenshake_phys_mass =4000.;;(*Masse de screenshake «normal». Des objets plus légers en provoqueront moins, les objets plus lourds plus*)
let screenshake_half_life = 0.1;;
let game_screenshake = ref 0.;;
let game_screenshake_pos = ref (0.,0.);;
let game_screenshake_previous_pos = ref (0.,0.);; (*Permet d'avoir un rendu correct des trainées de lumières lors du screenshake*)


let variable_exposure = true;;
let exposure_ratio_damage = 0.99;;
let exposure_half_life = 0.5;;
let game_exposure_target = ref 1.1;;
let game_exposure = ref ~-.1.;;


(******************************************************************************)
(*Définition des fonctions d'ordre général*)


(*Fonction de carre, pour écrire plus jolimment les formules de pythagore*)
let carre v = v *. v;;

(*Fonction de décroissance exponentielle de n au bout de t secondes en float. Basée sur le temps ingame*)
let exp_decay n half_life =
  n *. 2. ** (!game_speed *. (!time_last_frame -. !time_current_frame) /. half_life);;


(*Fonction de décroissance exponentielle de n au bout de t secondes en float. Basée sur le temps réel, pas sur le temps de jeu*)
let abso_exp_decay n half_life =
  n *. 2. ** ((!time_last_frame -. !time_current_frame) /. half_life);;

(*Toujours pratique pour faire du joli pythagore*)
let hypothenuse (x, y) = sqrt (carre x +. carre y);;

(*Permet l'addition de deux tuples*)
let addtuple (x1, y1) (x2, y2) = (x1 +. x2, y1 +. y2);;

(*Permet la soustraction de deux tuples*)
let soustuple (x1, y1) (x2, y2) = (x1 -. x2, y1 -. y2);;

(*Permet la multiplication d'un tuple par un float*)
let multuple (x, y) ratio = (x *. ratio, y *. ratio);;

(*Permet de convertir un tuple de float en tuple de int*)
let inttuple (x, y) = (int_of_float x, int_of_float y);;

(*Permet de convertir un tuple de int en float*)
let floattuple (x, y) = (float_of_int x, float_of_int y);;

let dither fl = if dither_aa then int_of_float (fl +. Random.float dither_power) else int_of_float fl;;

let dither_tuple tuple = let (x,y) = tuple in ( dither x, dither y);;


(*Permet l'addition de deux tuples, en poundérant le second par le ratio*)
let proj (x1, y1) (x2, y2) ratio = addtuple (x1, y1) (multuple (x2, y2) ratio);;

(*Transfert d'un vecteur en angle*valeur en x*y*)
let polar_to_affine angle valeur = (valeur *. cos angle, valeur *. sin angle);;

(*Transfert d'un vecteur en angle*valeur en x*y*)
let polar_to_affine_tuple (angle, valeur) = (valeur *. cos angle, valeur *. sin angle);;

(*Transfert d'un vecteur en x*y en angle*valeur *)
let affine_to_polar (x, y) =
let r = hypothenuse (x, y) in
if r = 0. then (0., 0.) (*Dans le cas où le rayon est nul, on ne peut pas déterminer d'angle donné*)
else (2. *. atan (y /. (x +. r)),r);;

(*La fonction distancecarre est plus simple niveau calcul qu'une fonction distance,*)
(*Car on évite la racine carrée, mais n'en reste pas moins utile pour les hitbox circulaires*)
let distancecarre (x1, y1) (x2, y2) = carre (x2 -. x1) +. carre (y2 -. y1);;

let modulo_float value modulo = if value < 0. then value +. modulo else if value >= modulo then value -. modulo else value;;

let modulo_reso (x, y) = (modulo_float x phys_width, modulo_float y phys_height);;

let modulo_3reso (x, y) = (modulo_float x (phys_width *. 3.), modulo_float y (phys_height *. 3.));;





(******************************************************************************)
(*Définition types pour état du jeu*)

(*Système de couleur hdr*)
(*Les couleurs vont de 0 à 000 non inclus, en float*)
type hdr = {
  r : float;
  v : float;
  b : float;
}
(*Fonctions sur les couleurs*)

(*Normalisation pour un espace normal de couleurs*)
let normal_color fl = max 0 (min 255 (int_of_float fl));;

(*Conversion de couleur_hdr vers couleur*)
let rgb_of_hdr hdr =
  rgb (normal_color hdr.r) (normal_color hdr.v) (normal_color hdr.b);;

(*Fonction d'intensité lumineuse d'une couleur hdr*)
let intensify hdr_in i = {r = i*. hdr_in.r ; v = i *. hdr_in.v ; b = i *. hdr_in.b};;

(*Fonction de saturation de la couleur*)
(*i un ratio entre 0 (N&B) et ce que l'on veut comme intensité des couleurs.*)
(*1 ne change rien*)
let saturate hdr_in i =
  let value = (hdr_in.r +. hdr_in.v +. hdr_in.b) /. 3. in
  {r = i *. hdr_in.r +. ((1. -. i) *. value); v = i *. hdr_in.v +. ((1. -. i) *. value); b= i *. hdr_in.b +. ((1. -. i) *. value)};;



(*Système de caméra*)

type camera = {
  position : (float*float);
  velocity : (float*float); (*La vitesse actuelle déplace la position de la caméra*)
  position_target : (float*float);
  zoom : float;
  zoom_target : float;
};;


(*On pourrait ajouter des types différents, par exemple des missiles à tête chercheuse, des vaisseaux ennemis…*)
(*À faire plus tard si le temps*)
type type_object = Asteroid | Projectile | Ship | Explosion | Smoke ;;

type objet_physique = {
  objet : type_object;
  (*TODO : Si le temps, définir une hitbox autre que circulaire*)
  mutable radius : float;
  mutable mass : float;
  mutable health : float;
  mutable max_health : float;

  dam_ratio : float; (*ratio des degats bruts réellements infligés*)
  dam_res : float; (*Réduction des dégats bruts.*)
  phys_ratio : float; (*ratio des dégats physiques réellement infligés*)
  phys_res : float; (*Réduction des dégats physiques, pour que les collisions à faible vitesse ne fassent pas de dégats*)

  (*Les objets ne subissent pas d'accélération, on change directement leur inertie pour des raisons de simplicité*)
  mutable position : (float*float);(*En pixels non entiers*)
  (*On stocke l'inertie en tuples, les calculs sont plus simples que direction + vitesse, aussi bien pour l'humain que pour la machine.*)
  mutable velocity : (float*float);(*En pixels.s⁻¹*)
  half_stop : float;(*Friction en temps de demi arrêt*)

  (*orientation en radians, moment en radians.s⁻¹*)
  mutable orientation : float;
  mutable moment : float;
  half_stop_rotat : float;(*Friction angulaire, en temps de demi-arrêt*)

  proper_time : float;

  polygon : (float*float) list;
  mutable hdr_color : hdr;
  mutable hdr_exposure : float;
};;


type etat = {
  mutable score : int;
  mutable cooldown : float;(*Le cooldown est le temps restant avant de pouvoir de nouveau tirer*)
  mutable ref_ship : objet_physique ref;
(*Les objets sont des listes de référence, pour la simplicité de la gestion*)
(*Il est plus simple de gérer la physique en séparant les objets spawnés et objets non spawnés*)
  mutable ref_fragments : objet_physique ref list; (*On fait apparaître les fragments dans une liste séparée pour éviter qu'ils ne s'entre-collisionnent*)
  mutable ref_objets : objet_physique ref list;
  mutable ref_objets_unspawned : objet_physique ref list;
  mutable ref_projectiles : objet_physique ref list;
  mutable ref_explosions : objet_physique ref list;
  mutable ref_smoke : objet_physique ref list;
};;



(*TODO Rendu personnalisé des textes*)
(*
let render_chiffre xmin xmax ymin ymax integer =
  match integer with :
  | 0
  | 1
*)


(*Système de rotation de polygone pour rendu*)
let rotat_point (theta,rayon) rotat = (theta +. rotat,rayon);;
let rec rotat_poly poly rotat =
  if poly = [] then [] else List.append [(rotat_point (List.hd poly) rotat)] (rotat_poly (List.tl poly) rotat) ;;
let scale_point (theta,rayon) scale = (theta, rayon *. scale);;
let rec scale_poly poly scale =
  if poly = [] then [] else List.append [(scale_point (List.hd poly) scale)] (scale_poly (List.tl poly) scale);;
let poly_to_affine poly rotat scale = List.map polar_to_affine_tuple (scale_poly (rotat_poly poly rotat) scale);;
let rec depl_affine_poly poly pos = if poly = [] then [] else (addtuple (List.hd poly) pos) :: (depl_affine_poly (List.tl poly) pos);;
let render_poly poly pos rotat color =
  let poly_to_render = depl_affine_poly (poly_to_affine poly rotat ratio_rendu) pos in
  set_color color;
  fill_poly (Array.of_list (List.map dither_tuple poly_to_render));;


(*permet le rendu de motion blur sur des objets sphériques*)
(*Part de l'endroit où un objet était à l'état précédent pour décider*)
let rec render_light_trail radius pos velocity hdr_color =  set_line_width (dither (ratio_rendu *.radius));
  let (x1,y1) = (multuple (addtuple pos !game_screenshake_pos) ratio_rendu) in
  let veloc = multuple velocity ~-. (!game_speed *. (max (shutter_speed /. framerate_render) (shutter_speed *.(!time_current_frame -. !time_last_frame)))) in
  let (x2,y2) = (multuple (addtuple (addtuple pos (multuple !game_screenshake_previous_pos shutter_speed)) veloc) ratio_rendu) in

  set_color (rgb_of_hdr (intensify hdr_color (sqrt ((radius *. ratio_rendu) /. (1. +. hypothenuse (soustuple (x1,y1) (x2,y2)))))));(*Plus la trainée de lumière est grande par rapport au rayon de l'objet, moins la lumière est intense*)

  moveto (dither x1) (dither y1);
  lineto (dither x2) (dither y2);
  ();;


let render_modulo x y radius color =
  set_color color;
  (*Dessiner l'objet*)
  fill_circle (dither x) (dither y)  (dither radius);
  (*Dessiner les modulos de l'objet des cotés*)
  fill_circle ((dither x) + width) (dither y)  (dither radius);
  fill_circle ((dither x) - width) (dither y)  (dither radius);
  fill_circle (dither x) ((dither y) + height) (dither radius);
  fill_circle (dither x) ((dither y) - height) (dither radius);
  (*Dessiner les modulos dans les angles*)
  fill_circle ((dither x) + width) ((dither y) + height)  (dither radius);
  fill_circle ((dither x) + width) ((dither y) - height)  (dither radius);
  fill_circle ((dither x) - width) ((dither y) + height)  (dither radius);
  fill_circle ((dither x) - width) ((dither y) - height)  (dither radius);;



let render_objet ref_objet =
  let objet = !ref_objet in
  let (x,y) = multuple (addtuple objet.position !game_screenshake_pos) ratio_rendu in
  (*Rendu de la vie de l'objet -> Deux cercles l'un dans l'autres, plus ou moins sombres*)
  render_modulo x y (ratio_rendu *. objet.radius) (rgb_of_hdr (intensify objet.hdr_color (!game_exposure *. 0.75 *. objet.hdr_exposure)));
  (*On dessine le polygone de l'objet*)
  if objet.polygon != [] then
    render_poly objet.polygon (x, y) objet.orientation (rgb_of_hdr (intensify objet.hdr_color (!game_exposure *. 0.5 *. objet.hdr_exposure)));
(*Partie intérieure de la vie*)
  render_modulo x y (max 0.  ((objet.health /. objet.max_health) *. ratio_rendu *. objet.radius)) (rgb_of_hdr (intensify objet.hdr_color (!game_exposure *. objet.hdr_exposure)));
  if objet.objet = Ship then (
    set_color (rgb_of_hdr (intensify objet.hdr_color (10. *. !game_exposure *. objet.hdr_exposure)));
    set_line_width 0;
    let (x2, y2) = multuple (polar_to_affine objet.orientation objet.radius) ratio_rendu in
    Graphics.draw_segments (Array.of_list [dither x, dither y, dither (x +. x2),dither (y +. y2)]);
    fill_circle (dither x) (dither y) (dither (ratio_rendu *. objet.radius *. 0.3)));;


(*Rendu des objets non spawnés - ne rend pas de duplicatas modulo l'écran*)
let render_unspawned ref_objet =
  let objet = !ref_objet in
  let (x,y) = multuple (addtuple objet.position !game_screenshake_pos) ratio_rendu in
  (*Rendu de la vie de l'objet*)
  set_color (rgb_of_hdr (intensify objet.hdr_color (0.5 *. !game_exposure *. objet.hdr_exposure)));
  fill_circle (dither x) (dither y) (dither (ratio_rendu *. objet.radius));
  set_color (rgb_of_hdr (intensify objet.hdr_color (!game_exposure *. objet.hdr_exposure)));
  fill_circle (dither x) (dither y) (max 0 (dither ((objet.health /. objet.max_health) *. ratio_rendu *. objet.radius)));;



let render_projectile ref_projectile =
  let objet = !ref_projectile in
  let full_size_bullet = 0.5 *. objet.radius +. 0.5 *. (Random.float objet.radius) in
  (*On rend plusieurs traits concentriques pour un effet de dégradé*)
  render_light_trail full_size_bullet objet.position objet.velocity  (intensify objet.hdr_color ( 0.25 *. objet.hdr_exposure *. !game_exposure));
  render_light_trail (full_size_bullet *. 0.75) objet.position objet.velocity (intensify objet.hdr_color (0.5 *. objet.hdr_exposure *. !game_exposure));
  render_light_trail (full_size_bullet *. 0.5) objet.position objet.velocity (intensify objet.hdr_color (objet.hdr_exposure *. !game_exposure));
  render_light_trail (full_size_bullet *. 0.25) objet.position objet.velocity (intensify objet.hdr_color (2. *. objet.hdr_exposure *. !game_exposure)
);;



(*TODO*)
(*Rendu de motion blur pour un objet*)
let render_motion = ();;



(*Fonction déplaçant un objet instantanémment sans prendre en compte le temps de jeu*)
let deplac_objet_abso ref_objet velocity =
let objet = !ref_objet in
  (*Si l'objet est un projectile, il despawne une fois au bord de l'écran*)
  objet.position <- proj objet.position velocity 1.;
ref_objet := objet;;

(*Même chose pour plusieurs objets*)
let rec deplac_objets_abso ref_objets velocity =
if List.length ref_objets > 0 then (
deplac_objet_abso (List.hd ref_objets) velocity;
deplac_objets_abso (List.tl ref_objets) velocity)
else ();;

(*Fonction déplaçant un objet selon une vélocitée donnée.*)
(*On tient compte du framerate et de la vitesse de jeu,*)
(*mais également du temps propre de l'objet et de l'observateur*)
let deplac_objet ref_objet (dx, dy) =
let objet = !ref_objet in
  (*Si l'objet est un projectile, il despawne une fois au bord de l'écran*)
  objet.position <- proj objet.position (dx, dy) ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
ref_objet := objet;;

(*Fonction accélérant un objet selon une accélération donnée.*)
(*On tient compte du framerate et de la vitesse de jeu,*)
(*mais également du temps propre de l'objet et de l'observateur*)
let accel_objet ref_objet (ddx, ddy) =
  let objet = !ref_objet in
  objet.velocity <- proj objet.velocity (ddx, ddy) ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
ref_objet := objet;;

(*Fonction boostant un objet selon une accélération donnée.*)
(*Utile pour le contrôle clavier par petites impulsions.*)
let boost_objet ref_objet boost =
  let objet = !ref_objet in
  objet.velocity <- (proj objet.velocity boost 1.);
ref_objet := objet;;

(*Fonction de rotation d'objet, avec rotation en radian*s⁻¹*)
let rotat_objet ref_objet rotation =
  let objet = !ref_objet in
  objet.orientation <- objet.orientation +. rotation *. ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
ref_objet := objet;;


(*Fonction de rotation d'objet, avec rotation en radian*s⁻²*)
let couple_objet ref_objet momentum =
  let objet = !ref_objet in
  objet.moment <- objet.moment +. momentum *. ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
ref_objet := objet;;


(*Fonction de rotation d'objet instantannée, avec rotation en radians.*)
let tourn_objet ref_objet rotation =
  let objet = !ref_objet in
  objet.orientation <- objet.orientation +. rotation;
ref_objet := objet;;

(*Fonction de rotation d'objet, avec rotation en radian*s⁻²*)
let couple_objet_boost ref_objet momentum =
  let objet = !ref_objet in
  objet.moment <- objet.moment +. momentum ;
ref_objet := objet;;


(*Fonction de calcul de changement de position inertiel d'un objet physique.*)
let inertie_objet ref_objet = deplac_objet ref_objet (!ref_objet).velocity;;

(*On calcule le changement de position inertiel de tous les objets en jeu*)
let inertie_objets ref_objets =
List.iter inertie_objet ref_objets;;

let friction_objet ref_objet =
  let objet = !ref_objet in
  objet.velocity <- multuple objet.velocity (exp_decay 1. objet.half_stop);
  ref_objet:=objet;;

let friction_moment_objet ref_objet =
  let objet = !ref_objet in
  objet.moment <- exp_decay objet.moment objet.half_stop_rotat;
  ref_objet:=objet;;

(*On calcule l'inertie en rotation des objets*)
let moment_objet ref_objet = rotat_objet ref_objet (!ref_objet).moment;;

(*D'un groupe d'objets*)
let moment_objets ref_objets = List.iter moment_objet ref_objets;;



let decay_smoke ref_smoke =
  let smoke = !ref_smoke in
  smoke.radius <- smoke.radius -. (!game_speed *. smoke_radius_decay *. (!time_current_frame -. !time_last_frame));
  (*Si l'exposition est déjà minimale, ne pas encombrer par un calcul de décroissance expo*)
  if smoke.hdr_exposure > 0.02 then  smoke.hdr_exposure <- (exp_decay smoke.hdr_exposure smoke_half_life);
  ref smoke;;



let damage ref_objet damage =
  let objet = !ref_objet in
  objet.health <- objet.health -. (max 0. (objet.dam_ratio *. damage -. objet.dam_res));
  game_screenshake := !game_screenshake +. damage *. screenshake_dam_ratio;
  game_exposure := !game_exposure *. exposure_ratio_damage;
  ref_objet := objet;;

let phys_damage ref_objet damage =
  let objet = !ref_objet in
  objet.health <- objet.health -. (max 0. (objet.phys_ratio *. damage -. objet.phys_res));
  game_screenshake := !game_screenshake +. damage *. screenshake_phys_ratio *. objet.mass /. screenshake_phys_mass;
  ref_objet := objet;;


let is_alive ref_objet = !ref_objet.health >= 0.;;

let is_dead ref_objet = !ref_objet.health <0.;;

(*Vérifie si un objet a le droit de spawner. (Si il est dans l'écran)*)
let checkspawn_objet ref_objet_unspawned =
  let objet = !ref_objet_unspawned in
  let (x, y) = objet.position in
 (x +. objet.radius < phys_width) && (x -. objet.radius > 0.)
  && (y +. objet.radius < phys_height) && (y -. objet.radius > 0.);;

let checknotspawn_objet ref_objet_unspawned = not (checkspawn_objet ref_objet_unspawned);;

(*Fait spawner tous les objets en ayant le droit*)
let checkspawn_etat ref_etat =
  if !ref_etat.ref_objets_unspawned = [] then ()
  else begin
    let etat = !ref_etat in
    let objets = etat.ref_objets in
    let objets_unspawned = etat.ref_objets_unspawned in
    etat.ref_objets <- (List.filter checkspawn_objet objets_unspawned) @ objets;
    etat.ref_objets_unspawned <- (List.filter checknotspawn_objet objets_unspawned);
  ref_etat := etat end;;


(*Booléen indiquant qu'un objet est suffisamment proche pour être encore pris en compte dans l'espace de jeu*)
let close_enough ref_objet =
  let (x, y) = !ref_objet.position in
  (x < 2. *. phys_width) && (x > 0. -. phys_width) && (y < 2. *. phys_height) && (y > 0. -.phys_height);;

let close_enough_bullet ref_objet =
  let (x, y) = !ref_objet.position in
  (x < 1.1 *. phys_width) && (x > 0. -. (0.1 *.phys_width)) && (y < 1.1 *. phys_height) && (y > 0. -. (0.1 *. phys_height));;

let positive_radius ref_objet = !ref_objet.radius > 0.;;

let big_enough ref_objet = !ref_objet.radius > asteroid_min_size;;

(*Fonction despawnant les objets trop lointains et morts, ou avec rayon négatif*)
let despawn ref_etat =
  let etat = !ref_etat in
    etat.ref_objets <- (List.filter is_alive etat.ref_objets);
    etat.ref_objets <- (List.filter big_enough etat.ref_objets);

    etat.ref_fragments <- (List.filter is_alive etat.ref_fragments);
    etat.ref_fragments <- (List.filter big_enough etat.ref_fragments);

    etat.ref_fragments <- (List.filter close_enough etat.ref_fragments);

    etat.ref_objets_unspawned <- (List.filter is_alive etat.ref_objets_unspawned);

    etat.ref_objets_unspawned <- (List.filter close_enough etat.ref_objets_unspawned);

    etat.ref_projectiles <- (List.filter is_alive etat.ref_projectiles);
    etat.ref_projectiles <- (List.filter close_enough_bullet etat.ref_projectiles);

    etat.ref_smoke <- (List.filter positive_radius etat.ref_smoke);
  ref_etat := etat;;


(*Recentrer les objets débordant de l'écran d'un côté de l'écran ou de l'autre*)
let recenter_objet ref_objet =
  let objet = !ref_objet in
  objet.position <- modulo_reso objet.position;
ref_objet := objet;;


(*Un objet non spawné doit être recentré par 3 fois *)
(*Ne semble par marcher pour l'instant
let recenter_objet_unspawned ref_objet =
  let objet = !ref_objet in
  objet.position <- (soustuple (modulo_3reso (addtuple objet.position (phys_width, phys_height))) (phys_width, phys_height));
ref_objet := objet;;*)

(*La racine carrée est une opération assez lourde,
Donc plutôt que de comparer la distance entre deux objets avec la somme de leur radii,
On compare le carré de leur distance avec le carré de la somme de leurs radiuses.
On travaille par hitbox circulaire pour 1-La simplicité du calcul 2-La proximité avec les formes réelles*)

(*Fonction vérifiant la collision entre deux objets*)
let collision objet1 objet2 =
(*Si on essaye de collisionner un objet avec lui-même, ça ne fonctionne pas*)
if objet1 = objet2 then false else distancecarre objet1.position objet2.position < carre (objet1.radius +. objet2.radius);;

(*Vérifie la collision entre un objet et une liste d'objets*)
let rec collision_objet_liste ref_objet ref_objets =
  if List.length ref_objets > 0 then (
  collision !ref_objet !(List.hd ref_objets) || collision_objet_liste ref_objet (List.tl ref_objets))
  else false;;

(*Retourne les objets de la liste 1 étant en collision avec des objets de la liste 2*)
let rec collision_objets_listes ref_objets1 ref_objets2 =
  if List.length ref_objets1 > 0  && List.length ref_objets1 > 0 then (
    if collision_objet_liste (List.hd ref_objets1) ref_objets2
      then List.hd ref_objets1 :: collision_objets_listes (List.tl ref_objets1) ref_objets2
      else collision_objets_listes (List.tl ref_objets1) ref_objets2;
  )else [];;

(*Retourne les objets de la liste 1 n'étant pas en collision avec des objets de la liste 2*)
let rec no_collision_objets_listes ref_objets1 ref_objets2 =
  if List.length ref_objets1 > 0  && List.length ref_objets1 > 0 then (
    if collision_objet_liste (List.hd ref_objets1) ref_objets2
      then no_collision_objets_listes (List.tl ref_objets1) ref_objets2
      else List.hd ref_objets1 :: no_collision_objets_listes (List.tl ref_objets1) ref_objets2;
  )else [];;

(*Retourne tous les objets d'une liste étant en collision avec au moins un autre*)
let rec collisions_sein_liste ref_objets = collision_objets_listes ref_objets ref_objets;;

(*Retourne tous les objets au sein d'une liste n'étant pas en collision avec les autres*)
let rec no_collisions_liste ref_objets = no_collision_objets_listes ref_objets ref_objets;;

(*Fonction appelée en cas de collision de deux objets.*)
(*Conséquences à compléter et améliorer*)
(*TODO*)
let consequences_collision ref_objet1 ref_objet2 =
  if !ref_objet1.objet = Explosion
    (*On applique les dégats de l'explosion*)
    then damage ref_objet2 explosion_damages else
  if !ref_objet1.objet = Projectile
    (*On endommage le projectile pour qu'il meure*)
    then damage ref_objet1 0.1
  else (
    let objet1 = !ref_objet1 in
    let objet2 = !ref_objet2 in
    let total_mass = objet1.mass +. objet2.mass in
    let moy_velocity = addtuple (multuple objet1.velocity (objet1.mass /. total_mass)) (multuple objet2.velocity (objet2.mass /. total_mass)) in
    let (angle_obj1, dist1) = affine_to_polar (soustuple objet1.position objet2.position) in
    let (angle_obj2, dist2) = affine_to_polar (soustuple objet2.position objet1.position) in
    (*Stockage des ancienne vélocités, pour calculer les dégats en fonction du nombre de G encaissées*)

    let old_vel1 = objet1.velocity in
    let old_vel2 = objet2.velocity in

    let veloc_obj1 = addtuple moy_velocity (polar_to_affine angle_obj1 (total_mass /. objet1.mass)) in
    objet2.velocity <- addtuple moy_velocity (polar_to_affine angle_obj2 (total_mass /. objet2.mass));
    objet1.velocity <- veloc_obj1;

    (*Changement de velocité subi par l'objet*)
    let g1 = hypothenuse (soustuple old_vel1 objet1.velocity) in
    let g2 = hypothenuse (soustuple old_vel2 objet2.velocity) in

    ref_objet1 := objet1;
    ref_objet2 := objet2;

    inertie_objet ref_objet1;
    inertie_objet ref_objet2;
    (*Les dégats physiques dépendent du changement de vitesse subie au carré.
    On applique un ratio pour réduire la valeur gigantesque générée*)
    phys_damage ref_objet1 (!ratio_phys_deg *. carre g1);
    phys_damage ref_objet2 (!ratio_phys_deg *. carre g2));;

(*Fonction vérifiant la collision entre un objet et les autres objets*)
(*Dès la première collision détectée, déclencher les conséquences, on considère qu'un objet ne peut avoir qu'une collision à la fois*)
let rec calculate_collisions_objet ref_objet ref_objets =
if List.length ref_objets > 0 then (
  if collision !ref_objet !(List.hd ref_objets)
  then consequences_collision ref_objet (List.hd ref_objets);
  calculate_collisions_objet ref_objet (List.tl ref_objets))
else ();;

let rec calculate_collisions_objets ref_objets =
if List.length ref_objets > 1 then (
calculate_collisions_objet (List.hd ref_objets) (List.tl ref_objets);
calculate_collisions_objets (List.tl ref_objets))
else ();;

let rec calculate_collisions_listes_objets ref_objets1 ref_objets2 =
if List.length ref_objets1 > 0 && List.length ref_objets2 > 0 then (
calculate_collisions_objet (List.hd ref_objets1) ref_objets2;
calculate_collisions_listes_objets (List.tl ref_objets1) ref_objets2)
else ();;


(*Petite fonction de déplacement d'objet exprès pour les modulos*)
(*Car la fonction de déplacement standard dépend de Δt*)
let deplac_obj_modulo ref_objet (x,y) = (*x et y sont des entiers, en quantité d'écrans*)
  let objet = !ref_objet in
  objet.position <- addtuple objet.position (phys_width *. float_of_int x, phys_height *. float_of_int y);
  ref_objet := objet;;

(*Fonction permettant aux objets simultanément à plusieurs endroits de l'écran de réagir correctement*)
let rec calculate_collisions_modulo ref_objet ref_objets =
if List.length ref_objets > 0 then (
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
  deplac_obj_modulo ref_objet (1, ~-1);
  calculate_collisions_objet ref_objet ref_objets)
else ();;

(*Même chose, pour une liste de ref objets*)
let rec calculate_collisions_modulos ref_objets =
if List.length ref_objets > 1 then (
calculate_collisions_modulo (List.hd ref_objets) (List.tl ref_objets);
calculate_collisions_modulos (List.tl ref_objets))
else ();;

(*Même chose, mais collision entre deux listes*)
let rec calculate_collisions_modulo_listes ref_objets1 ref_objets2 =
if List.length ref_objets1 > 0 && List.length ref_objets2 > 0 then (
calculate_collisions_modulo (List.hd ref_objets1) ref_objets2;
calculate_collisions_modulo_listes (List.tl ref_objets1) ref_objets2)
else ();;



(* --- initialisations etat --- *)

(* A DEFINIR : generation positions, deplacements initiaux ... *)

let spawn_ship = {
    objet = Ship;
    radius = ship_radius;
    mass =  pi *. (carre ship_radius) *. ship_density;
    health = ship_max_health;
    max_health = ship_max_health;

    dam_ratio = ship_dam_ratio;
    dam_res = ship_dam_res;
    phys_ratio = ship_phys_ratio;
    phys_res = ship_phys_res;

    position = (phys_width /. 2., phys_height /. 2.);
    velocity = (0.,0.);
    half_stop = ship_half_stop;

    orientation = pi /. 2.;
    moment = 0.;
    half_stop_rotat = ship_half_stop_rotat;

    polygon = [(0.,3.*.ship_radius);(3. *. pi /. 4.,2.*.ship_radius);(pi,ship_radius);(~-.3. *. pi /. 4.,2.*.ship_radius)];
    proper_time = 1.;
    hdr_color = {r=256.;v=16.;b=4.};
    hdr_exposure = 1.;
};;


let spawn_projectile position velocity = {
    objet = Projectile;
    radius = projectile_radius;
    mass = 0.;
    health = projectile_health;
    max_health = projectile_health;
    (*Les projectiles sont conçus pour être détruits au contact*)
    dam_res = 0.;
    dam_ratio = 1.;
    phys_res = 0.;
    phys_ratio = 1.;

    position = position;
    velocity = velocity;
    half_stop = ~-.1.;(*On le définit négatif pour l'ignorer lors du calcul*)

    orientation = 0.;
    moment = 0.;
    half_stop_rotat = ~-.1.;(*On le définit négatif pour l'ignorer lors du calcul*)

    proper_time = 1.;

    polygon = [];
    hdr_color = {r=2000.;v=400.;b=200.};
    hdr_exposure = 2.0;
};;



(*Spawne une explosion d'impact de projectile*)
let spawn_explosion ref_projectile = ref {
  objet = Explosion;
  radius = explosion_min_radius +. (Random.float (explosion_max_radius -. explosion_min_radius));
  mass = 0.;
  health = 0.;
  max_health = 0.;

  dam_res = 0.;
  dam_ratio = 0.;
  phys_res = 0.;
  phys_ratio = 0.;

  position = !ref_projectile.position;
  (*On donne à l'explosion une vitesse random, afin que la fumée qui en découle en hérite*)
  velocity = polar_to_affine (Random.float 2. *. pi) (Random.float smoke_max_speed);
  half_stop = 0.;
  orientation = 0.;
  moment = 0.;
  half_stop_rotat = 0.;

  polygon = [];
  proper_time = 1.;
  hdr_color = {r = 1500. ; v = 500. ; b = 250. };
  hdr_exposure = explosion_min_exposure +. (Random.float (explosion_max_exposure -. explosion_min_exposure));
}


(*Spawne une explosion héritant d'un objet d'une taille au choix.*)
let spawn_explosion_object ref_objet = ref {
  objet = Explosion;
  radius = explosion_ratio_radius *. !ref_objet.radius; (*On récupère le rayon de l'objet*)
  mass = 0.;
  health = 0.;
  max_health = 0.;

  dam_res = 0.;
  dam_ratio = 0.;
  phys_res = 0.;
  phys_ratio = 0.;

  position = !ref_objet.position;
  (*On donne à l'explosion une vitesse random, afin que la fumée qui en découle en hérite*)
  velocity = polar_to_affine (Random.float 2. *. pi) (Random.float smoke_max_speed);
  half_stop = 0.;
  orientation = 0.;
  moment = 0.;
  half_stop_rotat = 0.;

  polygon = [];
  proper_time = 1.;
(*La nouvelle exposition est partagée entre couleur et exposition, pour que la fumée ne finisse pas trop sombre*)
  hdr_color = intensify (saturate !ref_objet.hdr_color explosion_saturate) (0.2 *. (explosion_min_exposure_heritate +. (Random.float (explosion_max_exposure_heritate -. explosion_min_exposure_heritate))));
  hdr_exposure = 0.8 *. explosion_min_exposure_heritate +. (Random.float (explosion_max_exposure_heritate -. explosion_min_exposure_heritate));
}


(*TODO pour l'instant les muzzle et le feu héritent bien trop des explosions et sont bien trop hardcodés.
Définir leurs propres valeurs*)

(*Spawne un muzzleflash à la position donnée*)
let spawn_muzzle ref_projectile = ref {
  objet = Smoke;
  radius = muzzle_ratio_radius *. !ref_projectile.radius;
  mass = 0.;
  health = 0.;
  max_health = 0.;

  dam_res = 0.;
  dam_ratio = 0.;
  phys_res = 0.;
  phys_ratio = 0.;

  position = !ref_projectile.position;
  velocity = multuple !ref_projectile.velocity muzzle_ratio_speed;
  half_stop = 0.;
  orientation = 0.;
  moment = 0.;
  half_stop_rotat = 0.;

  proper_time = 1.;
  polygon = [];
  hdr_color = {r = 1500. ; v = 500. ; b = 250. };
  hdr_exposure = explosion_min_exposure +. (Random.float (explosion_max_exposure -. explosion_min_exposure));
}


(*Spawne du feu à l'arrière d'un vaisseau accélérant*)
let spawn_fire ref_ship = ref {
  objet = Smoke;
  radius = fire_ratio_radius *. !ref_ship.radius;
  mass = 0.;
  health = 0.;
  max_health = 0.;

  dam_res = 0.;
  dam_ratio = 0.;
  phys_res = 0.;
  phys_ratio = 0.;

  position = !ref_ship.position;
  velocity = addtuple !ref_ship.velocity (addtuple (polar_to_affine (!ref_ship.orientation +. pi) (fire_min_speed +. (Random.float (fire_max_speed -. fire_min_speed)))) (polar_to_affine (Random.float 2. *. pi) (Random.float fire_max_random)));
  half_stop = 0.;
  polygon = [];
  orientation = 0.;
  moment = 0.;
  half_stop_rotat = 0.;

  proper_time = 1.;
  hdr_color = {r = 1500. ; v = 400. ; b = 200. };
  hdr_exposure = explosion_min_exposure +. (Random.float (explosion_max_exposure -. explosion_min_exposure));
}



let spawn_asteroid (x, y) (dx, dy) radius = {
  objet = Asteroid;
  radius = radius;
  mass = pi *. (carre radius) *. asteroid_density;
  health = asteroid_mass_health *. pi *. (carre radius) *. asteroid_density +. asteroid_min_health;
  max_health = asteroid_mass_health *. pi *. (carre radius) *. asteroid_density +. asteroid_min_health;

  dam_res = asteroid_dam_res;
  dam_ratio = asteroid_dam_ratio;
  phys_res = asteroid_phys_res;
  phys_ratio = asteroid_phys_ratio;

  position = (x, y);
  velocity = (dx, dy);
  half_stop = ~-. 1.;(*On le définit en négatif pour l'ignorer lors du calcul*)
  orientation = Random.float (2. *. pi);
  moment = Random.float (2. *. asteroid_max_moment) -. asteroid_max_moment ;
  half_stop_rotat = ~-.1.;(*On le définit négatif pour l'ignorer lors du calcul*)

  proper_time = 1.;
polygon = [];(*
  polygon = [(0.,radius +. Random.float (radius *. 0.5));(pi /.2.,radius +. Random.float (radius *. 0.5));(pi,radius +. Random.float (radius *. 0.5));((~-.pi /. 2.),radius +. Random.float (radius *. 0.5))];*)
  hdr_color = {r = 96. +. Random.float 32. ; v = 96. +. Random.float 32. ; b = 96. +. Random.float 32. };
  hdr_exposure = 1.;
};;



let spawn_random_asteroid ref_etat =
  let etat = !ref_etat in
  let asteroid = spawn_asteroid (Random.float phys_width, Random.float phys_height) (polar_to_affine (Random.float 2. *. pi) (Random.float asteroid_max_velocity)) ( asteroid_min_spawn_radius +. (Random.float (asteroid_max_spawn_radius -. asteroid_min_spawn_radius))) in
  etat.ref_objets_unspawned <- (ref asteroid) :: etat.ref_objets_unspawned;
  ref_etat := etat ;;



(*Diminution de la taille d'un astéroide*)
(*Permet de spawner plusieurs sous-asteroides lors de la fragmentation*)
let frag_asteroid ref_asteroid =
  let asteroid = !ref_asteroid in
  let fragment = spawn_asteroid asteroid.position asteroid.velocity asteroid.radius in
  let orientation = (Random.float 2. *. pi) in
  let new_radius = (fragment_min_size +. Random.float (fragment_max_size -. fragment_min_size)) *. fragment.radius in
  fragment.position <- addtuple fragment.position (polar_to_affine orientation (fragment.radius -. new_radius));
  fragment.radius <- new_radius;
  fragment.mass <- pi *. asteroid_density *. (carre fragment.radius);
  fragment.health <- asteroid_mass_health *. fragment.mass +. asteroid_min_health;
  fragment.max_health <- fragment.health;
  fragment.velocity <- addtuple fragment.velocity (polar_to_affine orientation (fragment_min_velocity +. Random.float (fragment_max_velocity -. fragment_min_velocity)));
  fragment.hdr_color <- asteroid.hdr_color;
  fragment.hdr_exposure <- fragment.hdr_exposure *. (fragment_min_exposure +. Random.float (fragment_max_exposure -. fragment_min_exposure));
  ref fragment;;



let init_etat = {
  score = 0;
  cooldown = 0.;
  ref_ship = ref spawn_ship;
  ref_objets_unspawned = [];
  ref_objets = [];
  ref_fragments = [];
  ref_projectiles = [];
  ref_explosions = [];
  ref_smoke = [];
};;



(* Affichage des états*)


let affiche_etat ref_etat =
  (*Fond d'espace*)
  set_color (rgb_of_hdr (intensify {r=space_r; v=space_g; b=space_b} !game_exposure));
  fill_rect 0 ~-1 width height;
  let etat = !ref_etat in

  List.iter render_objet etat.ref_smoke;
  List.iter render_projectile etat.ref_projectiles;
  render_objet etat.ref_ship;
  List.iter render_unspawned etat.ref_fragments;
  List.iter render_unspawned etat.ref_objets_unspawned;
  List.iter render_objet etat.ref_objets;
  List.iter render_objet etat.ref_explosions;

  (*Affichage du framerate*)
  (*draw_string (string_of_int (int_of_float (1. /. !time_one_frame)));*)

  set_color white;
  draw_string (string_of_int (int_of_float (1000. *. (!time_current_frame -. !time_last_frame))));
  moveto 0 0;
  let (width_score, height_score) = text_size (string_of_int etat.score) in
  moveto 0 (height - height_score -200);
  draw_string (string_of_int etat.score);

  synchronize ();;



(********************************************************************************************************************)
(*WHERE THE MAGIC HAPPENS*)
(* calcul de l'etat suivant, apres un pas de temps *)
let etat_suivant ref_etat =
  let etat = !ref_etat in

  (*TODO vraie version du score*)
  etat.score <- etat.score + 1;

  (*On calcule le changement de vitesse naturel du jeu. Basé sur le temps réel et non le temps ingame pour éviter les casi-freeze*)
  game_speed := !game_speed_target +. abso_exp_decay (!game_speed -. !game_speed_target) half_speed_change;
  (*On calcule la puissance du screenshake. Basé sur le temps en jeu.*)
  game_screenshake := exp_decay !game_screenshake screenshake_half_life;
  (*On calcule l'emplacement caméra pour le screenshake,
  en mémorisant l'emplacement précédent du screenshake (Pour le rendu correct des trainées de lumière et du flou)*)
  game_screenshake_previous_pos := !game_screenshake_pos;
  game_screenshake_pos := (!game_screenshake *. ((Random.float 2.) -. 1.), !game_screenshake *. ((Random.float 2.) -. 1.));
  (*On calcule le changement de vitesse naturel du jeu. Basé sur le temps réel et non le temps ingame pour éviter les casi-freeze*)
  game_exposure := !game_exposure_target +. exp_decay (!game_exposure -. !game_exposure_target) exposure_half_life;



  (*On calcule tous les déplacements naturels dus à l'inertie des objets*)
  time_last_frame := !time_current_frame;
  time_current_frame := Unix.gettimeofday ();

  inertie_objet etat.ref_ship;
  inertie_objets etat.ref_objets;
  inertie_objets etat.ref_objets_unspawned;
  inertie_objets etat.ref_fragments;
  inertie_objets etat.ref_projectiles;
  inertie_objets etat.ref_smoke;

  moment_objet etat.ref_ship;
  moment_objets etat.ref_objets;
  moment_objets etat.ref_objets_unspawned;
  moment_objets etat.ref_fragments;
  (*Inutile de calculer le moment des projectiles, explosions ou fumée, comme leur rotation n'a aucune importance*)

  (*On calcule la friction et friction angulaire des objets*)
  friction_objet etat.ref_ship;
  friction_moment_objet etat.ref_ship;

  (*Collisions entre le vaisseau et les objets*)
  calculate_collisions_modulo etat.ref_ship etat.ref_objets;
  (*Collisions entre le vaisseau et les objets «non spwanés»*)
  calculate_collisions_objet etat.ref_ship etat.ref_objets_unspawned;
  (*Collisions entre le vaisseau et les fragments*)
  calculate_collisions_objet etat.ref_ship etat.ref_fragments;

  (*Collisions entre projectiles et objets*)
  calculate_collisions_modulo_listes etat.ref_projectiles etat.ref_objets;
  (*Collisions entre projectiles et objets «non spawnés» - non modulo*)
  calculate_collisions_listes_objets etat.ref_projectiles etat.ref_objets_unspawned;
  (*Collisions entre les projectiles et les fragments*)
  calculate_collisions_listes_objets etat.ref_projectiles etat.ref_fragments;

  (*Collisions entre explosions et objets*)
  calculate_collisions_modulo_listes etat.ref_explosions etat.ref_objets;
  (*Collisions entre explosions et objets «non spawnés» - non modulo*)
  calculate_collisions_listes_objets etat.ref_explosions etat.ref_objets_unspawned;
  (*Collisions entre explosions et les fragments*)
  calculate_collisions_listes_objets etat.ref_explosions etat.ref_fragments;

  (*Collisions entre objets*)
  calculate_collisions_modulos etat.ref_objets;
  (*Collisions entre objets spawnés et «non spawnés» - modulo pour le coup*)
  calculate_collisions_listes_objets etat.ref_objets etat.ref_objets_unspawned;
  (*Collisions entre objets et fragments*)
  calculate_collisions_listes_objets etat.ref_objets etat.ref_fragments;


  (*Les explosions sont ajoutées à la fumée, et la fumée précédente avec decay*)
  etat.ref_smoke <- List.append (List.map decay_smoke etat.ref_smoke) etat.ref_explosions;
  (*On fait apparaitre les explosions correspondant aux projectiles détruits*)
  etat.ref_explosions <- List.map spawn_explosion (List.filter is_dead etat.ref_projectiles);


  (*On fait apparaitre les explosions correspondant aux objets détruits*)
  etat.ref_explosions <- List.append etat.ref_explosions (List.map spawn_explosion_object (List.filter is_dead etat.ref_objets));
  etat.ref_explosions <- List.append etat.ref_explosions (List.map spawn_explosion_object (List.filter is_dead etat.ref_objets_unspawned));
  (*On ne fait pas exploser les fragments, car ils sont tous superposés, ça ne leur permet pas d'entrer dans le jeu, et fait des chutes de performances terribles*)
  (*On ralentit le temps selon le nombre d'explosions*)
  game_speed := !game_speed *. ratio_time_explosion ** (float_of_int (List.length etat.ref_explosions));

  (*On fait apparaitre 4 fragments des astéroïdes détruits*)
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets)) etat.ref_fragments ;
  (*Pareil pour les astéroïdes «non spawnés»*)
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets_unspawned)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets_unspawned)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets_unspawned)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets_unspawned)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets_unspawned)) etat.ref_fragments ;
  (*Pareil pour les fragments déjà cassés*)
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments)) etat.ref_fragments ;
  (*On ralentit le temps selon le nombre d'astéroïde détruits*)
  game_speed := !game_speed *. ratio_time_destr_asteroid ** (float_of_int (List.length (List.filter is_dead etat.ref_objets) + List.length (List.filter is_dead etat.ref_objets_unspawned) + List.length (List.filter is_dead etat.ref_fragments)));

  (*On transfère les fragments qui ne sont pas en collision avec les autres dans les objets physiques.
  On les considère comme unspawned pour éviter qu'ils ne se téléportent dans l'écran de jeu alors qu'ils sont hors de l'écran*)
  etat.ref_objets_unspawned <- List.append (no_collisions_liste etat.ref_fragments) etat.ref_objets_unspawned ;
  etat.ref_fragments <- collisions_sein_liste etat.ref_fragments;


  if dynamic_camera then (
    (*On calcule les déplacements de la caméra pour le rendu de caméra dynamique*)
    let (next_x, next_y) = addtuple !(etat.ref_ship).position  (multuple !(etat.ref_ship).velocity camera_prediction) in
    (*move_camera décrit plutôt un déplacement de la totalité des objets en jeu.*)
    let move_camera = (((phys_width /. 2.) -. next_x) -. (exp_decay ((phys_width /. 2.) -. next_x) camera_half_depl), ((phys_height/. 2.) -. next_y) -. (exp_decay ((phys_height/. 2.) -. next_y) camera_half_depl)) in

    deplac_objet_abso etat.ref_ship move_camera;
    deplac_objets_abso  etat.ref_objets move_camera;
    deplac_objets_abso  etat.ref_objets_unspawned move_camera;
    deplac_objets_abso  etat.ref_fragments move_camera;
    deplac_objets_abso  etat.ref_projectiles move_camera;
    deplac_objets_abso  etat.ref_explosions move_camera;
    deplac_objets_abso  etat.ref_smoke move_camera;
  );

  (*Recentrage des objets sortis de l'écran*)
  recenter_objet etat.ref_ship;
  List.iter recenter_objet etat.ref_objets;
  (*On reboucle les objets arrivant aux extrémités
  pour éviter que le joueur se débarrasse d'unspawned objects en s'écartant simplement*)
(* TODO revoir le recentrage des objets unspawned. En attendant on supprime.
  List.iter recenter_objet_unspawned etat.ref_objets_unspawned;
  List.iter recenter_objet_unspawned etat.ref_fragments;
*)
  (*On ne recentre pas les projectiles car ils doivent despawner une fois trop loin de l'espace de jeu*)

(*TODO faire un système de spawn d'astéroides propre. Pas encore bon pour l'instant.*)
  if Random.float framerate_limit < 1. then spawn_random_asteroid ref_etat;

  let elapsed_time = !time_current_frame -. !time_last_frame in
  (*On diminue le cooldown en fonction du temps passé depuis la dernière frame.*)
  (*On laisse si le cooldown est négatif, cela veut dire qu'un projectile a été tiré trop tard,
  et ce sera compensé par un projectile tiré trop tôt, afin d'équilibrer.*)
  if etat.cooldown > 0. then etat.cooldown <- etat.cooldown -. !game_speed *. elapsed_time;
  ref_etat := etat;
  (*Suppression des objets qu'il faut*)
  despawn ref_etat;
  (*On spawne ce qui doit spawner*)
  checkspawn_etat ref_etat;
  affiche_etat ref_etat;
  (*Équivalent bidouillé de sleepf en millisecondes, pour que le programme fonctionne aussi avec les anciennes versions d'Ocaml*)

  if locked_framerate then ignore (Unix.select [] [] [] (max 0. ((1. /. framerate_limit) -. elapsed_time)));;
  (*ne marche pas sur linux*)
();;


(* acceleration du vaisseau *)
let acceleration ref_etat =
  let etat = !ref_etat in
  let orientation = !(etat.ref_ship).orientation in
if ship_direct_pos then
  deplac_objet etat.ref_ship (polar_to_affine orientation ship_max_depl)
else
  (*Dans le cas d'un contrôle de la vélocité et non de la position.*)
  (*C'est à dire en respectant le TP, et c'est bien mieux en terme d'expérience de jeu :) *)
  accel_objet etat.ref_ship (polar_to_affine orientation ship_max_accel);
(*Feu à l'arrière du vaisseau. Spawne à chaque frame
plus de frames = plus de particules, pertes de perf = moins de particules.*)
let list_fire = [spawn_fire etat.ref_ship;spawn_fire etat.ref_ship] in
etat.ref_smoke <- etat.ref_smoke @ list_fire;
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
if ship_direct_rotat then
  rotat_objet !ref_etat.ref_ship ship_max_tourn
else(*Dans le cas d'un contrôle de la du couple et non de la rotation. Non recommandé de manière générale*)
  couple_objet !ref_etat.ref_ship ship_max_tourn;
etat_suivant ref_etat;;

let rotation_droite ref_etat =
if ship_direct_rotat then
  rotat_objet !ref_etat.ref_ship (0. -. ship_max_tourn)
else(*Dans le cas d'un contrôle de la du couple et non de la rotation. Non recommandé de manière générale*)
  couple_objet !ref_etat.ref_ship (0. -. ship_max_tourn);
etat_suivant ref_etat;;

(* rotation vers la gauche et vers la droite du ship *)
let boost_gauche ref_etat =
if ship_direct_rotat then
  tourn_objet !ref_etat.ref_ship (0. +. ship_max_rotat)
else(*Dans le cas d'un contrôle de la du couple et non de la rotation. Non recommandé de manière générale*)
  couple_objet_boost !ref_etat.ref_ship ship_max_tourn_boost;
etat_suivant ref_etat;;

let boost_droite ref_etat =
if ship_direct_rotat then
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
etat_suivant ref_etat;;

(* tir d'un nouveau projectile *)
let tir ref_etat =
  let etat = !ref_etat in
(*Tant que le cooldown est supérieur à 0, on ne tire pas.*)
(*Sauf si le temps que la prochaine frame arrive justifie qu'on puisse tirer entre temps*)
(*Plus le cooldown est faible, plus le tir devrait arriver tôt*)
(*Donc on laisse le hasard décider si le tir spawne maintenant ou à la frame suivante.*)
(*On considère que le temps de la prochaine frame sera celui de la dernière,
ce qui est une approximation généralement correcte*)
  if etat.cooldown < 0. || etat.cooldown < Random.float (!time_current_frame -. !time_last_frame)
  then (
    let ship = !(etat.ref_ship) in
    let position_tir = addtuple ship.position (polar_to_affine ship.orientation ship_radius) in
    (*Triple tir*)
    let velocity_tir1 = addtuple ship.velocity (polar_to_affine (((Random.float 1.) -. 0.5) *. projectile_deviation +. ship.orientation) (projectile_min_speed +. Random.float (projectile_max_speed -. projectile_min_speed))) in
    let velocity_tir2 = addtuple ship.velocity (polar_to_affine (((Random.float 1.) -. 0.5) *. projectile_deviation +. ship.orientation) (projectile_min_speed +. Random.float (projectile_max_speed -. projectile_min_speed))) in
    let velocity_tir3 = addtuple ship.velocity (polar_to_affine (((Random.float 1.) -. 0.5) *. projectile_deviation +. ship.orientation) (projectile_min_speed +. Random.float (projectile_max_speed -. projectile_min_speed))) in
    (*Trois projectiles*)
    let projectile1 = spawn_projectile position_tir velocity_tir1 in
    let projectile2 = spawn_projectile position_tir velocity_tir2 in
    let projectile3 = spawn_projectile position_tir velocity_tir3 in
    (* Muzzleflash. On utilise List.append pour que le Muzzleflash soit au dessus de la fumée déjà en jeu.*)
    etat.ref_smoke <- List.append etat.ref_smoke [spawn_muzzle (ref projectile1)];

    (*On ajoute les trois projectiles*)
    etat.ref_projectiles <- (ref projectile1) :: etat.ref_projectiles ;
    etat.ref_projectiles <- (ref projectile2) :: etat.ref_projectiles ;
    etat.ref_projectiles <- (ref projectile3) :: etat.ref_projectiles ;

    etat.cooldown <- etat.cooldown +. projectile_cooldown;
    ship.velocity <- addtuple ship.velocity (polar_to_affine (ship.orientation +. pi) projectile_recoil);

    etat.ref_ship := ship;
    ref_etat := etat;
    etat_suivant ref_etat;
) else ();;


(*Fonction  de contrôle souris*)
let controle_souris ref_etat =
  let etat = !ref_etat in
  let ship = !(etat.ref_ship) in
  let status = wait_next_event[Poll] in
  let (xv,yv) = ship.position in
  let (theta, r) =
    affine_to_polar
      ((float_of_int status.mouse_x) /. ratio_rendu -. xv,
      (float_of_int status.mouse_y) /. ratio_rendu -. yv) in
  ship.orientation <- theta;
  etat.ref_ship :=  ship;
  ref_etat := etat;
  if status.button then acceleration ref_etat else ();;


(*État une fois mort*)
let rec mort ref_etat =
  tir ref_etat;
  acceleration ref_etat;
  boost_droite ref_etat;
  game_speed_target := 0.2;
  game_exposure_target := 0.5;
  if key_pressed  ()then
  let status = wait_next_event[Key_pressed] in
    match status.key  with (* ...en fonction de la touche frappee *)
    | 'r' -> ref_etat := init_etat; (*R permet de recommencer une partie de zéro rapidement. TODO : Le faire fonctionner*)
    | 'q' -> print_endline "Bye bye!"; exit 0 (* on quitte le jeu *)
    | _ -> etat_suivant ref_etat; mort ref_etat;;

(* --- boucle d'interaction --- *)

let rec boucle_interaction ref_etat =
  if !(!ref_etat.ref_ship).health<0. then
    mort ref_etat
  else
    if mousecontrol then controle_souris ref_etat;
    if key_pressed  ()then
    let status = wait_next_event[Key_pressed] in
      match status.key  with (* ...en fonction de la touche frappee *)
      | 'w' -> ref_etat := init_etat; boucle_interaction ref_etat (*R permet de recommencer une partie de zéro rapidement. TODO : Le faire fonctionner*)
      | 'v' -> strafe_left ref_etat; boucle_interaction ref_etat (*strafe vers la gauche *)
      | 't' -> boost_gauche ref_etat; boucle_interaction ref_etat (* rotation vers la gauche *)
      | 'd' -> boost ref_etat;boucle_interaction ref_etat (* acceleration vers l'avant *)
      | 'r' -> boost_droite ref_etat;boucle_interaction ref_etat (* rotation vers la droite *)
      | 'l' -> strafe_right ref_etat; boucle_interaction ref_etat (*strafe vers la droite *)
      | ' ' -> tir ref_etat;boucle_interaction ref_etat (* tir d'un projectile *)
      | 'q' -> print_endline "Bye bye!"; exit 0 (* on quitte le jeu *)
      | _ -> etat_suivant ref_etat;boucle_interaction ref_etat
   else
    etat_suivant ref_etat;
    boucle_interaction ref_etat;;

(* --- fonction principale --- *)

let main () =
  Random.self_init ();
  open_graph (" " ^ string_of_int width ^ "x" ^ string_of_int height);
  auto_synchronize false;
  set_text_size (int_of_float (10. *. ratio_rendu));

  (* initialisation de l'etat du jeu *)
  let ref_etat = ref init_etat in

(*On s'assure d'avoir un repère temporel correct*)
  time_last_frame := Unix.gettimeofday();
  time_current_frame := Unix.gettimeofday();
  etat_suivant ref_etat;
  affiche_etat ref_etat;
  boucle_interaction ref_etat;; (* lancer la boucle d'interaction avec le joueur *)

let _ = main ();; (* demarrer le jeu *)
