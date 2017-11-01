(*spécifique à windows*)
(*
#load "unix.cma"
#load "graphics.cma"
*)
open Graphics
let pi = 4. *. atan 1.(*Pi*)

(*Certaines valeurs par défaut ne suivent pas les instructions du tp pour une meilleure expérience de jeu.*)
(*Ces changements sont documentés dans les commentaires et peuvent être remis aux valeurs du pdf si nécessaire.*)



(******************************************************************************)
(*Paramètres affichage*)


(*Paramètres temporels*)

(*Le temps propre de l'observateur.
En l'occurrence, on récupère celui du vaisseau.
Cela permet d'avoir une relativité Einsteinienne.*)
(*TODO s'en servir.*)
let observer_proper_time = ref 1.(*En ratio du temps «absolu» de l'univers*)
(*Le game_speed_target est la vitesse à laquelle on veut que le jeu tourne en temps normal*)
let game_speed_target_death = 0.5
let game_speed_target_boucle = 1.
let game_speed_target = ref 1.
(*Le game_speed est la vitesse réelle à laquelle le jeu tourne à l'heure actuelle.*)
(*Cela permet notamment de faire des effets de ralenti ou d'accéléré*)
let game_speed = ref 1.
(*Le half_speed_change détermine à quelle «vitesse» le game speed se rapproche de game_speed_target (En demi-vie) *)
let half_speed_change = 0.5

(*Ratios de changement de vitesse en fonction des évènements*)
let ratio_time_explosion = 0.99
let ratio_time_destr_asteroid = 0.9

(*La limitation de framerate est activable,
mais il semblerait que le gettimeofday et l'attente de Unix.select
ne soient pas assez précis pour que chaque frame dure juste le temps qu'il faut.
Mon conseil est de ne pas l'activer.*)
let locked_framerate = false
(*Le framerate demandé dans l'énoncé est de 20.
Un framerate plus élevé offre une meilleure expérience de jeu :
Des contrôles plus réactifs, un meilleur confort visuel, et une physique plus précise.
Bien sûr, il est possible de le changer ci-dessous*)
let framerate_limit = 200.
(*Le framerate de rendu permet de déterminer la longueur du motion blur.
Réglez au framerate réel de votre écran,
et shutter_speed contrôle la longueur du flou comme une caméra réelle*)
let framerate_render = 60.

(*On stocke le moment auquel la dernière frame a été calculée
pour synchroniser correctement le moment de calcul de la frame suivante*)
let time_last_frame = ref 0.
let time_current_frame = ref 0.

(*Pour le calcul des fps, on stocke le dernier moment auquel on comptait les images, et on actualise régulièrement.*)
let time_last_count = ref 0.
let time_current_count = ref 10.
let last_count = ref 0
let current_count = ref 0

(*Dimensions fenêtre graphique.*)
let width = 1200
let height = 600
let game_surface = 1.
(*Dimensions de l'espace physique dans lequel les objets évoluent.
On s'assure que la surface de jeu soit la même quelle que soit la résolution.
On conserve au passage le ratio de la résolution pour les dimensions de jeu
On a une surface de jeu de 1 000 000 par défaut*)
let ratio_rendu = sqrt ((float_of_int width) *. (float_of_int height) /. (game_surface *. 1000000.))
let phys_width = float_of_int width /. ratio_rendu
let phys_height = float_of_int height /. ratio_rendu


(******************************************************************************)
(*Paramètres graphiques avancés*)

let space_r = 0.
let space_g = 0.
let space_b = 20.

(*Paramètres de flou de mouvement*)
(*Fonctionne déjà pour les bullets
TODO l'implémenter pour les autres objets*)
let motion_blur = true
let shutter_speed = 1.

(******************************************************************************)
(*Paramètres de jeu*)

(*Permet le contrôle du vaisseau à la souris.
Viser avec la souris, clic pour accélérer, toujours barre d'espace pour tirer*)
let mousecontrol = true
(*Les contrôles directs ne contrôlent pas la vitesse et le moment mais directement la position et la rotation.
Les valeurs par défaut sont celles demandées dans le tp*)
(*TODO implémenter correctement toutes les méthodes de contrôle*)
let ship_direct_pos = false
let ship_direct_rotat = false

(*Ratio pour conversion des dégats physiques depuis le changement de vélocité au carré*)
let ratio_phys_deg = ref 0.002

(*Paramètres des astéroïdes*)
let asteroid_max_spawn_radius = 150. (*Taille max d'astéroïde au spawn.*)
let asteroid_min_spawn_radius = 30. (*Taille min de spawn*)
let asteroid_min_size = 20. (*En dessous de la taille minimale, un asteroide ne se divise pas à sa mort*)
let asteroid_max_moment = 1. (*Rotation max d'un astéroïde au spawn (dans un sens aléatoire)*)
let asteroid_max_velocity = 100. (*Velocité max au spawn*)
let asteroid_min_velocity = 10. (*Velocité min au spawn*)
let asteroid_density = 1. (*Sert à déterminer la masse d'un astéroïde en se basant sur sa surface*)
let asteroid_min_health = 20. (*Évite les astéroïdes trop fragiles à cause d'une masse trop faible. S'additionne au calcul.*)
let asteroid_mass_health = 0.01(*Sert à déterminer la vie d'un astéroïde basé sur sa masse*)
(*Dam : dommmages. phys : dommages physiques. Ratio : Multiplicateur du dégat. res : résistance aux dégats (soustraction)*)
let asteroid_dam_ratio = 1.
let asteroid_dam_res = 0.
let asteroid_phys_ratio = 1.
let asteroid_phys_res = 20.

(*Caractéristiques des fragments. Principalement hérité des parents.*)
let fragment_max_velocity = 200. (*Velocité max au spawn*)
let fragment_min_velocity = 50.  (*Velocité min au spawn*)
let fragment_max_size = 0.7(*En ratio de la taille de l'astéroïde parent*)
let fragment_min_size = 0.4 (*En ratio de la taille de l'astéroïde parent*)
let fragment_min_exposure = 0.5
let fragment_max_exposure = 2.0


(*Paramètres du vaisseau*)

(*valeurs du vaisseau*)
let ship_max_health = 100. (*health au spawn. Permet de l'appliquer au modèle physique.*)
let ship_max_healths = 3 (*Nombre de fois que le vaisseau peut réapparaître*)
let ship_density = 50. (*Pour calcul de la masse du vaisseau, qui a un impact sur la physique*)
let ship_radius = 10. (*Pour la hitbox et le rendu*)
(*Réduction des dégats et dégats physiques*)
let ship_dam_ratio = 0.8
let ship_dam_res = 10.
let ship_phys_ratio = 1.
let ship_phys_res = 5.
let ship_death_max_momentum = 2.
(*Contrôles de déplacement*)
let ship_max_depl = 50. (*En px.s⁻¹. Utile si contrôle direct du déplacement.*)
let ship_max_accel = 1200. (*En px.s⁻² Utile si contrôle de l'accélération*)
let ship_max_boost = 100. (*En px.s⁻¹. Utile si contrôle par boost.*)
let ship_half_stop = 10. (*En temps nécessaire pour perdre la moitié de l'inertie*)
(*Contrôles de rotation*)
let ship_max_tourn = 4. (*En radian.s⁻¹*)
let ship_max_moment = 0.5 (*En radian.s⁻²*)
let ship_max_tourn_boost = 3.
let ship_max_rotat = pi /. 6.
let ship_half_stop_rotat = 0.2(*En temps nécessaire pour perdre la moitié du moment angulaire*)

(*Valeurs du projectile*)
let projectile_recoil = 10.
let projectile_cooldown = 0.1
let projectile_max_speed = 1800.(*Vitesse relative au lanceur lors du lancement*)
let projectile_min_speed = 1000.
let projectile_deviation = 0.2(*Déviation possible de la trajectoire des projectiles*)
let projectile_radius = 6.
let projectile_health = 0.(*On considère la mort quand la santé descend sous zéro. On a ici la certitude que le projectile se détruira*)

(*Valeurs des explosions*)
let explosion_max_radius = 40.
let explosion_min_radius = 20.
let explosion_min_exposure = 1.(*Détermine la luminosité max et min des explosions au spawn*)
let explosion_max_exposure = 2.
let explosion_damages = 6.
(*Pour les explosions héritant d'un objet*)
let explosion_ratio_radius = 1.5
let explosion_saturate = 6.
let explosion_min_exposure_heritate = 30.(*Détermine la luminosité max et min des explosions héritant d'objets au spawn*)
let explosion_max_exposure_heritate = 40.

(*Valeurs des muzzleflashes*)
let muzzle_ratio_radius = 0.5
let muzzle_ratio_speed = 0.01

(*Valeurs du feu à l'arrière du vaisseau*)
let fire_max_random = 20.
let fire_min_speed = 250.
let fire_max_speed = 500.
let fire_ratio_radius = 1.

(*Valeurs de la fumée*)
let smoke = true
let smoke_half_life = 0.1 (*Vitesse de la décroissance de la couleur*)
let smoke_radius_decay = 20. (*Diminution du rayon des particules de fumée*)
let smoke_max_speed = 40.(*Vitesse random dans une direction random de la fumée*)

(*Valeurs des étincelles TODO*)

(*Valeurs des étoiles*)
let star_radius = 10.
let star_min_prox = 0.4
let star_max_prox = 0.9
let star_min_lum = 5.
let star_max_lum = 10.
let star_rand_lum = 5. (*Effet de scintillement des étoiles*)
let stars_nb = 100


(*Effet de scanlines pour imiter les moniteurs crt*)
let scanlines = false
let scanlines_period = 2
let animated_scanlines = false
let scanlines_offset = ref 0

(*La camera predictive oriente la camera vers l'endroit où le vaisseau va,
pour le garder tant que possible au centre de l'écran*)
let dynamic_camera = true
let camera_prediction = 1.4 (*En secondes de déplacement du vaisseau dans le futur.*)
let camera_half_depl = 1.2 (*Temps pour se déplacer de moitié vers l'objectif de la caméra*)

(*Le screenshake ajoute des effets de tremblements à l'intensité dépendant  des évènements*)
let screenshake = true
let screenshake_dam_ratio = 0.05
let screenshake_phys_ratio = 0.03
let screenshake_phys_mass = 4000.(*Masse de screenshake «normal». Des objets plus légers en provoqueront moins, les objets plus lourds plus*)
let screenshake_half_life = 0.1
let game_screenshake = ref 0.
let game_screenshake_pos = ref (0.,0.)
let game_screenshake_previous_pos = ref (0.,0.) (*Permet d'avoir un rendu correct des trainées de lumières lors du screenshake*)


(*L'antialiasing de jitter fait «trembler» l'espace de rendu.
C'est une forme de dithering spatial
afin de compenser la perte de précision due à la rastérisation
lors du placement des objets et du tracé des contours.*)
let dither_aa = true
(*La puissance du jitter détermine à quel point le rendu peut se décaler.*)
(*Déterminer à 1 ou moins pour éviter un effet de flou et de fatigue visuelle*)
let dither_power = 0.5 (*En ratio de la taille d'un pixel*)
(*Le jitter courant permet de faire le même jitter sur tous les rayons avant de les convertir en entier.*)
let current_jitter = ref 0.
(*Le jitter double courant permet de faire le même jitter sur les positions d'objets.
Cela permet de s'assurer une consistance spatiale dans tout le rendu.*)
let current_jitter_double = ref (0.,0.)

(*L'exposition variable permet des variations de luminosité en fonction des évènements*)
let variable_exposure = true
let exposure_ratio_damage = 0.99
let exposure_half_life = 0.5
let game_exposure_target_death =0.2
let game_exposure_target_boucle = 1.5
let game_exposure_target = ref 1.5
let game_exposure = ref ~-.1.


(******************************************************************************)
(*Définition des fonctions d'ordre général*)

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

(*Permet la multiplication de deux termes séparés au sein d'un tuple*)
let multuple_parallel (x1,y1) (x2,y2) = (x1 *. x2, y1 *. y2)

(*Permet de convertir un tuple de float en tuple de int*)
let inttuple (x, y) = (int_of_float x, int_of_float y)

(*Permet de convertir un tuple de int en float*)
let floattuple (x, y) = (float_of_int x, float_of_int y)

let dither_indep fl = if dither_aa then int_of_float (fl +. Random.float dither_power) else int_of_float fl

let dither fl = if dither_aa then int_of_float (fl +. !current_jitter) else int_of_float fl

let dither_tuple (x,y) = if dither_aa then (dither x, dither y) else inttuple (x,y)

(*Permet l'addition de deux tuples, en poundérant le second par le ratio*)
let proj (x1, y1) (x2, y2) ratio = addtuple (x1, y1) (multuple (x2, y2) ratio)

(*Transfert d'un vecteur en angle*valeur en x*y*)
let polar_to_affine angle valeur = (valeur *. cos angle, valeur *. sin angle)

(*Transfert d'un vecteur en angle*valeur en x*y*)
let polar_to_affine_tuple (angle, valeur) = (valeur *. cos angle, valeur *. sin angle)

(*Transfert d'un vecteur en x*y en angle*valeur *)
let affine_to_polar (x, y) =
let r = hypothenuse (x, y) in
if r = 0. then (0., 0.) (*Dans le cas où le rayon est nul, on ne peut pas déterminer d'angle donné*)
else (2. *. atan (y /. (x +. r)),r)

(*La fonction distancecarre est plus simple niveau calcul qu'une fonction distance,*)
(*Car on évite la racine carrée, mais n'en reste pas moins utile pour les hitbox circulaires*)
let distancecarre (x1, y1) (x2, y2) = carre (x2 -. x1) +. carre (y2 -. y1)

let modulo_float value modulo = if value < 0. then value +. modulo else if value >= modulo then value -. modulo else value

let modulo_reso (x, y) = (modulo_float x phys_width, modulo_float y phys_height)

let modulo_3reso (x, y) = (modulo_float x (phys_width *. 3.), modulo_float y (phys_height *. 3.))



(******************************************************************************)
(*Définition types pour état du jeu*)


(*Fonctions sur les couleurs*)

(*Système de couleur*)
(*Pas de limite arbitraire de luminosité. Les négatifs donnent du noir et sont acceptés.*)
type hdr = {r : float ; v : float ; b : float;}

(*Normalisation pour un espace normal de couleurs*)
let normal_color fl = max 0 (min 255 (int_of_float fl))

(*Conversion de couleur_hdr vers couleur*)
let rgb_of_hdr hdr = rgb (normal_color hdr.r) (normal_color hdr.v) (normal_color hdr.b)

(*Fonction d'intensité lumineuse d'une couleur hdr*)
let intensify hdr_in i = {r = i*. hdr_in.r ; v = i *. hdr_in.v ; b = i *. hdr_in.b}

(*Fonction de saturation de la couleur*)
(*i un ratio entre 0 (N&B) et ce que l'on veut comme intensité des couleurs.*)
(*1 ne change rien*)
let saturate hdr_in i =
  let value = (hdr_in.r +. hdr_in.v +. hdr_in.b) /. 3. in
  {r = i *. hdr_in.r +. ((1. -. i) *. value); v = i *. hdr_in.v +. ((1. -. i) *. value); b= i *. hdr_in.b +. ((1. -. i) *. value)}


(*On pourrait ajouter des types différents, par exemple des missiles à tête chercheuse, des vaisseaux ennemis…*)
(*TODO plus tard si le temps*)
type type_object = Asteroid | Projectile | Ship | Explosion | Smoke | Spark

type objet_physique = {
  objet : type_object;
  (*TODO : Si le temps, définir une hitbox autre que circulaire*)
  mutable radius : float;
  mutable mass : float;
  mutable health : float;
  mutable max_health : float;
  (*Fonction de résistance physique et aux dommages*)
  dam_ratio : float; (*ratio des degats bruts réellements infligés*)
  dam_res : float; (*Réduction des dégats bruts.*)
  phys_ratio : float; (*ratio des dégats physiques réellement infligés*)
  phys_res : float; (*Réduction des dégats physiques, pour que les collisions à faible vitesse ne fassent pas de dégats*)

  mutable last_position : (float*float);(*Sert à stocker l'emplacement précédent pour calcul correct du motion blur*)
  mutable position : (float*float);(*En pixels non entiers*)
  (*On stocke l'inertie en tuples, les calculs sont plus simples que direction + vitesse, aussi bien pour l'humain que pour la machine.*)
  mutable velocity : (float*float);(*En pixels.s⁻¹*)
  half_stop : float;(*Friction en temps de demi arrêt*)

  (*orientation en radians, moment en radians.s⁻¹*)
  mutable orientation : float;
  mutable moment : float;
  half_stop_rotat : float;(*Friction angulaire, en temps de demi-arrêt*)

  proper_time : float;

  polygon : (float*float) list; (*Polygone pour le rendu. Liste de points en coordonées polaires autour du centre de l'objet.*)
  mutable hdr_color : hdr;
  mutable hdr_exposure : float;
}


type star = {
  mutable last_pos : (float*float);(*La position précédente permet de calculer correctement le motion_blur*)
  mutable pos : (float*float); (*Si on l'appelle pos, toutes les fonctions appelant objet_physique.position ralent comme quoi star n'est pas un objet physique.*)
  proximity : float;(*Proximité avec l'espace de jeu.
  À 1, se situe sur le même plan que le vaisseau, à 0, à une distance infinie.
  Correspond simplement au ratio de déplacement lors du mouvement caméra*)
  lum : float;
}


type etat = {
  mutable score : int;
  mutable cooldown : float; (*Le cooldown est le temps restant avant de pouvoir de nouveau tirer*)
  mutable last_health : float;
  mutable ref_ship : objet_physique ref;
(*Les objets sont des listes de référence, pour la simplicité de la gestion*)
(*Il est plus simple de gérer la physique en séparant les objets par type
plutôt que de vérifier le type d'objet à la volée*)
  mutable ref_fragments : objet_physique ref list; (*On fait apparaître les fragments dans une liste séparée pour éviter qu'ils ne s'entre-collisionnent*)
  mutable ref_fragments_unspawned : objet_physique ref list; (*Pour les fragments d'objets unspawned, pour éviter de les recentrer*)
  mutable ref_objets : objet_physique ref list;
  mutable ref_objets_unspawned : objet_physique ref list;
  mutable ref_projectiles : objet_physique ref list;
  mutable ref_explosions : objet_physique ref list;
  mutable ref_smoke : objet_physique ref list;
  mutable ref_sparks : objet_physique ref list;
  mutable ref_stars : star ref list;
}


(*TODO Rendu personnalisé des textes pour affichage du score*)


(*Système de rotation de polygone pour rendu.*)
let rotat_point (theta,rayon) rotat = (theta +. rotat,rayon)
let rec rotat_poly poly rotat =
  if poly = [] then [] else List.append [(rotat_point (List.hd poly) rotat)] (rotat_poly (List.tl poly) rotat)
let scale_point (theta,rayon) scale = (theta, rayon *. scale)
let rec scale_poly poly scale =
  if poly = [] then [] else List.append [(scale_point (List.hd poly) scale)] (scale_poly (List.tl poly) scale)
let poly_to_affine poly rotat scale = List.map polar_to_affine_tuple (scale_poly (rotat_poly poly rotat) scale)
let rec depl_affine_poly poly pos = if poly = [] then [] else (addtuple (List.hd poly) pos) :: (depl_affine_poly (List.tl poly) pos)
let render_poly poly pos rotat color =
  let poly_to_render = depl_affine_poly (poly_to_affine poly rotat ratio_rendu) pos in
  set_color color; set_line_width 0;
  fill_poly (Array.of_list (List.map dither_tuple poly_to_render))

(*Permet de rendre un polygone ayant des points déterminés en pourcentage de largeur et hauteur
en points en int. (Avec dither le cas échéant)*)
let rec relative_poly points_list =
  if points_list = [] then [] else dither_tuple (multuple_parallel (List.hd points_list) (float_of_int width,float_of_int height)) :: (relative_poly (List.tl points_list))



(*permet le rendu de motion blur sur des objets sphériques*)
(*Part de l'endroit où un objet était à l'état précédent pour décider*)
let render_light_trail radius last_pos pos velocity hdr_color =
(*TODO corriger le fait que le shutter_speed ne semble pas avoir d'influence sur la longueur des trainées de lumière dues au screenshake*)
  set_line_width (dither (2.*.radius));
  let pos1 = (multuple (addtuple pos !game_screenshake_pos) ratio_rendu) in
  let veloc = multuple velocity ~-. (!game_speed *. (max (1. /. framerate_render) (1. *.(!time_current_frame -. !time_last_frame)))) in
  let last_position = (multuple (addtuple (addtuple last_pos !game_screenshake_previous_pos) veloc) ratio_rendu) in
  let pos2 = addtuple (multuple last_position shutter_speed) (multuple pos1 (1. -. shutter_speed)) in
  set_color (rgb_of_hdr (intensify hdr_color (!game_exposure *. 0.5 *. (sqrt (radius /. (radius +. hypothenuse (soustuple pos1 pos2)))))));(*Plus la trainée de lumière est grande par rapport au rayon de l'objet, moins la lumière est intense*)
  let (x1,y1) = dither_tuple pos1 in
  let (x2,y2) = dither_tuple pos2 in
  moveto x1 y1 ; lineto x2 y2;;

let render_star_trail ref_star =
  let star = !ref_star in
  let pos1 = (multuple (addtuple star.pos !game_screenshake_pos) ratio_rendu) in
  let last_position = (multuple (addtuple star.last_pos (!game_screenshake_previous_pos)) ratio_rendu) in
  let pos2 = addtuple (multuple last_position shutter_speed) (multuple pos1 (1. -. shutter_speed))in
  let (x1,y1) = dither_tuple pos1 in
  let (x2,y2) = dither_tuple pos2 in
  let lum = star.lum +. Random.float star_rand_lum in
  if (x1 = x2 && y1 = y2) then (
    set_color (rgb_of_hdr (intensify {r=lum *. 25.;v=lum *. 50.;b=lum *. 100.} !game_exposure ));
    plot x1 y1;
      set_color (rgb_of_hdr (intensify {r=lum *. 25.;v=lum *. 50.;b=lum *. 100.} (0.25 *. !game_exposure)));
      plot (x1+1) y1;
      plot (x1-1) y1;
      plot x1 (y1+1);
      plot x1 (y1-1);
  )else (
    set_color (rgb_of_hdr (intensify {r=lum *. 25.;v=lum *. 50.;b=lum *. 100.} (!game_exposure *. (sqrt (1. /. (1. +. hypothenuse (soustuple pos1 pos2)))))));(*Plus la trainée de lumière est grande par rapport au rayon de l'objet, moins la lumière est intense*)
    moveto x1 y1 ; lineto x2 y2);;


let render_motion_blur ref_objet = (*TODO : Fonction ajouter, pour fondre avec le background*)
  let objet = !ref_objet in
  render_light_trail (ratio_rendu *. objet.radius) objet.last_position objet.position objet.velocity (intensify objet.hdr_color (0.5 *. !game_exposure *. objet.hdr_exposure))
  (*Pour garder le motion blur discret, on rend les trainées plus sombres que l'objet.
  De même, on ne tient pas combte du déplacement de la caméra, car l'œuil humain va suivre ce type de mouvements.
  Le motion blur ne doit être visible que pour les mouvements violents de type screenshake.*)

let render_modulo x y radius color =
  set_color color;
  (*On calcule déjà le calcul de dithering de x, y et radius plutôt que de le calculer 9 fois.*)
  let x = dither x in
  let y = dither y in
  let radius = dither radius in
  (*Dessiner l'objet*)
  fill_circle x y radius;
  (*Dessiner les modulos de l'objet des cotés. Pas très élégant, mais c'est le plus simple.*)
  fill_circle (x + width) y radius;
  fill_circle (x - width) y  radius;
  fill_circle x (y + height) radius;
  fill_circle x (y - height) radius;
  (*Dessiner les modulos dans les angles*)
  fill_circle (x + width) (y + height) radius;
  fill_circle (x + width) (y - height) radius;
  fill_circle (x - width) (y + height)  radius;
  fill_circle (x - width) (y - height)  radius


(*TODO nettoyage de code à faire ici, et des optimisations.*)
let render_objet ref_objet =
  let objet = !ref_objet in
  let (x,y) = multuple (addtuple objet.position !game_screenshake_pos) ratio_rendu in
  (*On dessine le polygone de l'objet.*)
  if objet.polygon != [] then
    render_poly objet.polygon (x, y) objet.orientation (rgb_of_hdr (intensify objet.hdr_color (!game_exposure *. objet.hdr_exposure)));

  (*Rendu de la vie de l'objet -> Deux cercles l'un dans l'autres, plus ou moins sombres*)
  render_modulo x y (ratio_rendu *. objet.radius) (rgb_of_hdr (intensify objet.hdr_color (!game_exposure *. 0.75 *. objet.hdr_exposure)));
(*Partie intérieure de la vie*)
  render_modulo x y (max 0.  ((objet.health /. objet.max_health) *. ratio_rendu *. objet.radius)) (rgb_of_hdr (intensify objet.hdr_color (!game_exposure *. objet.hdr_exposure)));

  if objet.objet = Ship then (
    set_color (rgb_of_hdr (intensify objet.hdr_color (10. *. !game_exposure *. objet.hdr_exposure)));
    set_line_width 0;
    let (x2, y2) = multuple (polar_to_affine objet.orientation objet.radius) ratio_rendu in
    Graphics.draw_segments (Array.of_list [dither x, dither y, dither (x +. x2),dither (y +. y2)]);
    fill_circle (dither x) (dither y) (dither (ratio_rendu *. objet.radius *. 0.3)))


(*Rendu des objets non spawnés - ne rend pas de duplicatas modulo l'écran*)
let render_unspawned ref_objet =
  let objet = !ref_objet in
  let (x,y) = multuple (addtuple objet.position !game_screenshake_pos) ratio_rendu in
  (*Rendu de la vie de l'objet*)
  set_color (rgb_of_hdr (intensify objet.hdr_color (0.5 *. !game_exposure *. objet.hdr_exposure)));
  fill_circle (dither x) (dither y) (dither (ratio_rendu *. objet.radius));
  set_color (rgb_of_hdr (intensify objet.hdr_color (!game_exposure *. objet.hdr_exposure)));
  fill_circle (dither x) (dither y) (max 0 (dither ((objet.health /. objet.max_health) *. ratio_rendu *. objet.radius)))


(*Rendu des projectiles. Dessine des trainées de lumière.*)
let render_projectile ref_projectile =
  let objet = !ref_projectile in
  let full_size_bullet = ratio_rendu *. (0.5 *. objet.radius +. 0.5 *. (Random.float objet.radius)) in
  (*On rend plusieurs traits concentriques pour un effet de dégradé*)
  render_light_trail full_size_bullet objet.last_position objet.position objet.velocity  (intensify objet.hdr_color ( 0.25 *. objet.hdr_exposure *. !game_exposure));
  render_light_trail (full_size_bullet *. 0.75) objet.last_position objet.position objet.velocity (intensify objet.hdr_color (0.5 *. objet.hdr_exposure *. !game_exposure));
  render_light_trail (full_size_bullet *. 0.5) objet.last_position objet.position objet.velocity (intensify objet.hdr_color (objet.hdr_exposure *. !game_exposure));
  render_light_trail (full_size_bullet *. 0.25) objet.last_position objet.position objet.velocity (intensify objet.hdr_color (2. *. objet.hdr_exposure *. !game_exposure)
);;

let render_spark ref_spark =
  let objet = !ref_spark in
  render_light_trail objet.radius objet.last_position objet.position objet.velocity (intensify objet.hdr_color (objet.hdr_exposure *. !game_exposure));;


(*Fonction déplaçant un objet instantanémment sans prendre en compte le temps de jeu*)
let deplac_objet_abso ref_objet velocity =
let objet = !ref_objet in
objet.last_position <- objet.position;
objet.position <- proj objet.position velocity 1.;(*TODO sans doute moins sale de modifier direct la position*)
ref_objet := objet;;

(*Même chose pour plusieurs objets*)
let rec deplac_objets_abso ref_objets velocity =
if List.length ref_objets > 0 then (
deplac_objet_abso (List.hd ref_objets) velocity;
deplac_objets_abso (List.tl ref_objets) velocity)
else ();;

(*Déplacement des étoiles en tenant compte de leur proximité*)
let deplac_star ref_star velocity =
  let star = !ref_star in
  star.last_pos <- star.pos;
  let (next_x, next_y) = addtuple star.pos (multuple velocity star.proximity) in
  star.pos <- modulo_reso (next_x, next_y);
  if (next_x > phys_width || next_x < 0. || next_y > phys_height || next_y < 0.) then star.last_pos <- star.pos; (*On évite le motion blur incorrect causé par une téléportation d'un bord à l'autre de l'écran.*)
  ref_star := star;;



(*Déplacement d'un ensemble d'étoiles*)
let rec deplac_stars ref_stars velocity =
  if ref_stars=[] then [] else (deplac_star (List.hd ref_stars) velocity) :: (deplac_stars (List.tl ref_stars) velocity);;


(*Fonction déplaçant un objet selon une vélocitée donnée.
On tient compte du framerate et de la vitesse de jeu,
mais également du temps propre de l'objet et de l'observateur*)
let deplac_objet ref_objet (dx, dy) =
let objet = !ref_objet in
  (*Si l'objet est un projectile, il despawne une fois au bord de l'écran*)
  objet.position <- proj objet.position (dx, dy) ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
ref_objet := objet;;

(*Fonction accélérant un objet selon une accélération donnée.
On tient compte du framerate et de la vitesse de jeu,
mais également du temps propre de l'objet et de l'observateur*)
let accel_objet ref_objet (ddx, ddy) =
  let objet = !ref_objet in
  objet.velocity <- proj objet.velocity (ddx, ddy) ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
ref_objet := objet;;

(*Fonction boostant un objet selon une accélération donnée.*)
(*Utile pour le contrôle clavier par petites impulsions.*)
let boost_objet ref_objet boost =
  let objet = !ref_objet in objet.velocity <- (proj objet.velocity boost 1.);
ref_objet := objet;;

(*Fonction de rotation d'objet, avec rotation en radian*s⁻¹*)
let rotat_objet ref_objet rotation =
  let objet = !ref_objet in objet.orientation <- objet.orientation +. rotation *. ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
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
List.iter inertie_objet ref_objets;; (*TODO laisser tomber cette fonction, l'écrire direct telle-quelle dans la boucle de jeu.*)

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
let moment_objets ref_objets = List.iter moment_objet ref_objets;; (*TODO supprimer cette fonction et appeler direct telle-quelle dans la boucle principale.*)

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
  if variable_exposure then game_exposure := !game_exposure *. exposure_ratio_damage;
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
  (x < 1.01 *. phys_width) && (x > 0. -. (0.01 *.phys_width)) && (y < 1.01 *. phys_height) && (y > 0. -. (0.01 *. phys_height));;

let positive_radius ref_objet = !ref_objet.radius > 0.;;

let big_enough ref_objet = !ref_objet.radius > asteroid_min_size;;

(*Fonction despawnant les objets trop lointains et morts, ou avec rayon négatif*)
let despawn ref_etat =
  let etat = !ref_etat in
    (*Pas besoin de checker close_enough pour les objets spawnés, on les recentre.*)
    etat.ref_objets <- (List.filter is_alive etat.ref_objets);
    etat.ref_objets <- (List.filter big_enough etat.ref_objets);

    etat.ref_fragments <- (List.filter is_alive etat.ref_fragments);
    etat.ref_fragments <- (List.filter big_enough etat.ref_fragments);

    etat.ref_fragments_unspawned <- (List.filter is_alive etat.ref_fragments_unspawned);
    etat.ref_fragments_unspawned <- (List.filter big_enough etat.ref_fragments_unspawned);
    etat.ref_fragments_unspawned <- (List.filter close_enough etat.ref_fragments_unspawned);

    etat.ref_objets_unspawned <- (List.filter is_alive etat.ref_objets_unspawned);
    etat.ref_objets_unspawned <- (List.filter close_enough etat.ref_objets_unspawned);

    etat.ref_projectiles <- (List.filter is_alive etat.ref_projectiles);
    (*TODO permettre un missile ne despawnant pas après mort, mais provoquant plusieurs explosions sur son passage*)
    etat.ref_projectiles <- (List.filter close_enough_bullet etat.ref_projectiles);

    etat.ref_smoke <- (List.filter positive_radius etat.ref_smoke);
  ref_etat := etat;;


(*Recentrer les objets débordant de l'écran d'un côté de l'écran ou de l'autre*)
let recenter_objet ref_objet =
  let objet = !ref_objet in
  let (next_x, next_y) = modulo_reso objet.position in
  objet.position <- (next_x, next_y);
  if (next_x > phys_width || next_x < 0. || next_y > phys_height || next_y < 0.)
    then objet.last_position <- objet.position;(*On évite d'avoir du flou incorrect d'un côté à l'autre de l'écran*)
ref_objet := objet;;

(*Un objet non spawné doit être recentré par 3 fois *)
(*Ne semble par marcher pour l'instant
let recenter_objet_unspawned ref_objet =
  let objet = !ref_objet in
  objet.position <- (soustuple (modulo_3reso (addtuple objet.position (phys_width, phys_height))) (phys_width, phys_height));
ref_objet := objet;;*)

(*La racine carrée est une opération assez lourde,
Donc plutôt que de comparer la distance entre deux objets avec la somme de leur radii,
On compare le carré de leur distance avec le carré de la somme de leurs radii..
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
let rec calculate_collisions_objet ref_objet ref_objets =
if List.length ref_objets > 0 then (
  if collision !ref_objet !(List.hd ref_objets) then consequences_collision ref_objet (List.hd ref_objets);
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

(*Fonction permettant aux objets simultanément à plusieurs endroits de l'écran de réagir correctement au niveau physique*)
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


let spawn_ship () = {
    objet = Ship;
    radius = ship_radius;
    mass =  pi *. (carre ship_radius) *. ship_density;
    health = ship_max_health;
    max_health = ship_max_health;

    dam_ratio = ship_dam_ratio;
    dam_res = ship_dam_res;
    phys_ratio = ship_phys_ratio;
    phys_res = ship_phys_res;

    last_position = (phys_width /. 2., phys_height /. 2.);
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

    last_position = position;
    position = position;
    velocity = velocity;
    half_stop = ~-.1.;(*On le définit négatif pour l'ignorer lors du calcul*)

    orientation = 0.;
    moment = 0.;
    half_stop_rotat = ~-.1.;(*On le définit négatif pour l'ignorer lors du calcul*)

    proper_time = 1.;

    polygon = [];
    hdr_color = {r=2000.;v=400.;b=200.};
    hdr_exposure = 1.8;
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

  last_position = !ref_projectile.position;
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

  last_position = !ref_objet.position;
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

  last_position = !ref_projectile.position;
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

  last_position = !ref_ship.position;
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

  last_position = (x,y);
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


(*TODO maintenant, faire spawner les astéroïdes seulement en dehors de l'écran de jeu*)
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


let spawn_random_star () = {
  last_pos = (Random.float phys_width, Random.float phys_height);
  pos = (Random.float phys_width, Random.float phys_height);
  proximity = (Random.float star_min_prox +. Random.float (star_max_prox -. star_min_prox)) ** 4.;
  lum = Random.float star_min_lum +. Random.float (star_max_lum -. star_min_lum);
}

let rec add_n_stars ref_stars n =
  if n=0 then ref_stars else (ref (spawn_random_star ())) :: add_n_stars ref_stars (n-1) ;;


let init_etat () = {
  score = 0;
  cooldown = 0.;
  last_health = ship_max_health;
  ref_ship = ref (spawn_ship ());
  ref_objets = [];
  ref_objets_unspawned = [];
  ref_fragments = [];
  ref_fragments_unspawned = [];
  ref_projectiles = [];
  ref_explosions = [];
  ref_smoke = [];
  ref_sparks = [];
  ref_stars = add_n_stars [] stars_nb;
};;


(* Affichage des états*)

(*Fonction d'affichage de barre de vie. Nécessite un quadrilatère comme polygone d'entrée.
Les deux premiers points correspondent à une valeur de zéro, et les deux derniers à la valeur max de la barre.*)
let affiche_barre ratio [(x0,y0);(x1,y1);(x2,y2);(x3,y3)] color_bar =
  set_color color_bar;
  fill_poly (Array.of_list (relative_poly
  [(x0,y0);(x1,y1);
  (ratio *. x2 +. (1. -. ratio) *. x1, ratio *. y2 +. (1. -. ratio) *. y1);
  (ratio *. x3 +. (1. -. ratio) *. x0, ratio *. y3 +. (1. -. ratio) *. y0)]))

  let rec render_scanlines nb=
    set_color black;
    set_line_width 0;
    if nb < height then (
    moveto 0 nb;
    lineto width nb;
    render_scanlines (nb + scanlines_period));;

let affiche_hud ref_etat =
  let etat = !ref_etat in

  moveto 0 (height/2);
  draw_string (string_of_int etat.score);

  let ship = !(etat.ref_ship) in
  etat.last_health <- (max 0. ship.health) +. (exp_decay (etat.last_health -. (max 0. ship.health)) 0.5);
  affiche_barre 1. [(0.95,0.9);(0.95,0.85);(0.6,0.85);(0.55,0.9)] black;
  affiche_barre (etat.last_health /. ship_max_health) [(0.95,0.9);(0.95,0.85);(0.6,0.85);(0.55,0.9)] yellow;
  affiche_barre ((max 0. ship.health) /. ship_max_health) [(0.95,0.9);(0.95,0.85);(0.6,0.85);(0.55,0.9)] red;


  if scanlines then (
    if animated_scanlines then
      (render_scanlines (0 + !scanlines_offset);scanlines_offset := 1- !scanlines_offset)
    else
      render_scanlines 0);

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
  draw_string (string_of_int !last_count);

  etat.ref_ship := ship;
  ref_etat := etat;
();;

let affiche_etat ref_etat =
  let etat = !ref_etat in
  (*On actualise la caméra en fonction du vaisseau.
  Dans les faits on bouge les objets, mais tous de la même valeur donc pas de réel impact*)
  if dynamic_camera then (
    (*On calcule les déplacements de la caméra pour le rendu de caméra dynamique*)
    let (next_x, next_y) = addtuple !(etat.ref_ship).position  (multuple !(etat.ref_ship).velocity camera_prediction) in
    (*move_camera décrit plutôt un déplacement de la totalité des objets en jeu.*)
    let move_camera = (((phys_width /. 2.) -. next_x) -. (exp_decay ((phys_width /. 2.) -. next_x) camera_half_depl), ((phys_height/. 2.) -. next_y) -. (exp_decay ((phys_height/. 2.) -. next_y) camera_half_depl)) in
    ignore (deplac_stars etat.ref_stars move_camera);
    deplac_objet_abso etat.ref_ship move_camera;
    deplac_objets_abso  etat.ref_objets move_camera;
    deplac_objets_abso  etat.ref_objets_unspawned move_camera;
    deplac_objets_abso  etat.ref_fragments move_camera;
    deplac_objets_abso  etat.ref_projectiles move_camera;
    deplac_objets_abso  etat.ref_explosions move_camera;
    deplac_objets_abso  etat.ref_smoke move_camera;
  );
  (*Fond d'espace*)
  set_color (rgb_of_hdr (intensify {r=space_r; v=space_g; b=space_b} !game_exposure));
  fill_rect 0 ~-1 width height;


  if motion_blur then (
    List.iter render_motion_blur etat.ref_fragments_unspawned;
    List.iter render_motion_blur etat.ref_fragments;
    List.iter render_motion_blur etat.ref_objets_unspawned;
    List.iter render_motion_blur etat.ref_objets;
    set_line_width 0;
    List.iter render_star_trail etat.ref_stars;(*On rend les étoiles derrière la fumée, mais derrière les autres objets moins lumineux.*)
    (*List.iter render_motion_blur etat.ref_smoke;*)(*TODO régler le fait que le blur soit appliqué qu'une fois sur deux.*)
  )else (
    set_line_width 0; List.iter render_star_trail etat.ref_stars);(*Avec ou sans motion blur, on rend les étoiles comme il faut*)
  set_line_width 0;

  List.iter render_objet etat.ref_smoke;
  List.iter render_projectile etat.ref_projectiles;
  render_objet etat.ref_ship;
  List.iter render_unspawned etat.ref_fragments_unspawned;
  List.iter render_objet etat.ref_fragments;
  List.iter render_unspawned etat.ref_objets_unspawned;
  List.iter render_objet etat.ref_objets;
  List.iter render_objet etat.ref_explosions;

  affiche_hud ref_etat;
  synchronize ();;


(********************************************************************************************************************)
(*WHERE THE MAGIC HAPPENS*)
(* calcul de l'etat suivant, apres un pas de temps *)
let etat_suivant ref_etat =
  let etat = !ref_etat in

  (*On calcule le changement de vitesse naturel du jeu. Basé sur le temps réel et non le temps ingame pour éviter les casi-freeze*)
  game_speed := !game_speed_target +. abso_exp_decay (!game_speed -. !game_speed_target) half_speed_change;
  (*On calcule la puissance du screenshake. Basé sur le temps en jeu.*)
  game_screenshake := exp_decay !game_screenshake screenshake_half_life;
  (*On calcule l'emplacement caméra pour le screenshake,
  en mémorisant l'emplacement précédent du screenshake (Pour le rendu correct des trainées de lumière et du flou)*)
  game_screenshake_previous_pos := !game_screenshake_pos;
  if screenshake then game_screenshake_pos := (!game_screenshake *. ((Random.float 2.) -. 1.), !game_screenshake *. ((Random.float 2.) -. 1.));
  (*On calcule le jitter, pour l'appliquer de manière uniforme sur tous les objets et tous les rayons.*)
  current_jitter := Random.float dither_power;
  current_jitter_double := (Random.float dither_power, Random.float dither_power);
  (*On calcule le changement d'exposition du jeu. Basé sur le temps en jeu *)
  game_exposure := !game_exposure_target +. exp_decay (!game_exposure -. !game_exposure_target) exposure_half_life;


  (*On calcule tous les déplacements naturels dus à l'inertie des objets*)
  time_last_frame := !time_current_frame;
  time_current_frame := Unix.gettimeofday ();

  inertie_objet etat.ref_ship;
  inertie_objets etat.ref_objets;
  inertie_objets etat.ref_objets_unspawned;
  inertie_objets etat.ref_fragments;
  inertie_objets etat.ref_fragments_unspawned;
  inertie_objets etat.ref_projectiles;
  inertie_objets etat.ref_smoke;

  moment_objet etat.ref_ship;
  moment_objets etat.ref_objets;
  moment_objets etat.ref_objets_unspawned;
  moment_objets etat.ref_fragments;
  moment_objets etat.ref_fragments_unspawned;
  (*Inutile de calculer le moment des projectiles, explosions ou fumée, comme leur rotation n'a aucune importance*)

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
  (*Collisions entre objets spawnés et «non spawnés» - modulo pour le coup*)
  calculate_collisions_listes_objets etat.ref_objets etat.ref_objets_unspawned;
  (*Collisions entre objets et fragments*)
  calculate_collisions_modulo_listes etat.ref_objets etat.ref_fragments;
  (*Collisions entre objets et fragments non spawnés*)
  calculate_collisions_listes_objets etat.ref_objets etat.ref_fragments;

  (*Les explosions sont ajoutées à la fumée, et la fumée précédente avec decay. Uniquement si smoke = true.*)
  if smoke then etat.ref_smoke <- List.append (List.map decay_smoke etat.ref_smoke) etat.ref_explosions;
  (*On fait apparaitre les explosions correspondant aux projectiles détruits*)
  etat.ref_explosions <- List.map spawn_explosion (List.filter is_dead etat.ref_projectiles);

  (*On fait apparaitre les explosions correspondant aux objets détruits*)
  etat.ref_explosions <- List.append etat.ref_explosions (List.map spawn_explosion_object (List.filter is_dead etat.ref_objets));
  etat.ref_explosions <- List.append etat.ref_explosions (List.map spawn_explosion_object (List.filter is_dead etat.ref_objets_unspawned));
  (*On ne fait pas exploser les fragments, car ils sont tous superposés, ça ne leur permet pas d'entrer dans le jeu, et fait des chutes de performances terribles*)
  (*Le vaisseau génère aussi une trainée d'explosions après sa mort*)
  if (is_dead etat.ref_ship) then etat.ref_explosions <- (spawn_explosion etat.ref_ship) :: etat.ref_explosions;

  (*On ralentit le temps selon le nombre d'explosions*)
  game_speed := !game_speed *. ratio_time_explosion ** (float_of_int (List.length etat.ref_explosions));

(*TODO voir si je peux pas faire ça de manière plus élégante. Parce que bon, c'est pas très beau, ça.*)
  (*On fait apparaitre 5 fragments des astéroïdes détruits*)
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets)) etat.ref_fragments ;
  (*Pareil pour les astéroïdes «non spawnés»*)
  etat.ref_fragments_unspawned <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets_unspawned)) etat.ref_fragments_unspawned ;
  etat.ref_fragments_unspawned <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets_unspawned)) etat.ref_fragments_unspawned ;
  etat.ref_fragments_unspawned <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets_unspawned)) etat.ref_fragments_unspawned ;
  etat.ref_fragments_unspawned <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets_unspawned)) etat.ref_fragments_unspawned ;
  etat.ref_fragments_unspawned <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_objets_unspawned)) etat.ref_fragments_unspawned ;
  (*Pareil pour les fragments déjà cassés*)
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments)) etat.ref_fragments ;
  etat.ref_fragments <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments)) etat.ref_fragments ;
  (*Pareil pour les fragments unspawned déjà cassés*)
  etat.ref_fragments_unspawned <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments_unspawned)) etat.ref_fragments_unspawned ;
  etat.ref_fragments_unspawned <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments_unspawned)) etat.ref_fragments_unspawned ;
  etat.ref_fragments_unspawned <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments_unspawned)) etat.ref_fragments_unspawned ;
  etat.ref_fragments_unspawned <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments_unspawned)) etat.ref_fragments_unspawned ;
  etat.ref_fragments_unspawned <- List.append (List.map frag_asteroid (List.filter is_dead etat.ref_fragments_unspawned)) etat.ref_fragments_unspawned ;

  (*On ralentit le temps selon le nombre d'astéroïde détruits*)
  (*TODO : Baser le ralentissement de temps non sur la quantité d'astéroïdes détruits et le nombre d'explosions,
  mais selon les dégats et dégats physiques, comme pour le screenshake.*)
  let nb_destroyed = List.length (List.filter is_dead etat.ref_objets) + List.length (List.filter is_dead etat.ref_objets_unspawned) + List.length (List.filter is_dead etat.ref_fragments)in
  game_speed := !game_speed *. ratio_time_destr_asteroid ** (float_of_int nb_destroyed);
  etat.score <- etat.score + nb_destroyed; (*TODO meilleure version du score avec multiplicateurs*)

  (*On transfère les fragments qui ne sont pas en collision avec les autres dans les objets physiques.
  On les considère comme unspawned pour éviter qu'ils ne se téléportent dans l'écran de jeu alors qu'ils sont hors de l'écran*)
  etat.ref_objets <- List.append (no_collisions_liste etat.ref_fragments) etat.ref_objets ;
  etat.ref_objets_unspawned <- List.append (no_collisions_liste etat.ref_fragments_unspawned) etat.ref_objets_unspawned ;

  etat.ref_fragments <- collisions_sein_liste etat.ref_fragments;
  etat.ref_fragments_unspawned <- collisions_sein_liste etat.ref_fragments_unspawned;


  (*Recentrage des objets sortis de l'écran*)
  recenter_objet etat.ref_ship;
  List.iter recenter_objet etat.ref_objets;
  List.iter recenter_objet etat.ref_fragments;
  (*On reboucle les objets arrivant aux extrémités
  pour éviter que le joueur se débarrasse d'unspawned objects en s'écartant simplement*)
(* TODO revoir le recentrage des objets unspawned. En attendant on supprime.
  List.iter recenter_objet_unspawned etat.ref_objets_unspawned;
  List.iter recenter_objet_unspawned etat.ref_fragments;
*)
  (*On ne recentre pas les projectiles car ils doivent despawner une fois sortis de l'espace de jeu*)

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
  (*TODO trouver pourquoi ça ne marche pas sur les systèmes linux. Penser à le régler avant de le rendre.*)
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
if !(etat.ref_ship).health > 0. then (let list_fire = [spawn_fire etat.ref_ship;spawn_fire etat.ref_ship] in
if smoke then etat.ref_smoke <- etat.ref_smoke @ list_fire;);
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
    let position_tir = addtuple ship.position (polar_to_affine ship.orientation (2. *. ship_radius)) in
    (*Quintuple tir*) (*TODO une fonction nuple tir, qui crée un tableau avec n projectiles.
    Sera utile pour des fonctions générales d'arme pour les powerups.*)
    let velocity_tir1 = addtuple ship.velocity (polar_to_affine (((Random.float 1.) -. 0.5) *. projectile_deviation +. ship.orientation) (projectile_min_speed +. Random.float (projectile_max_speed -. projectile_min_speed))) in
    let velocity_tir2 = addtuple ship.velocity (polar_to_affine (((Random.float 1.) -. 0.5) *. projectile_deviation +. ship.orientation) (projectile_min_speed +. Random.float (projectile_max_speed -. projectile_min_speed))) in
    let velocity_tir3 = addtuple ship.velocity (polar_to_affine (((Random.float 1.) -. 0.5) *. projectile_deviation +. ship.orientation) (projectile_min_speed +. Random.float (projectile_max_speed -. projectile_min_speed))) in
    let velocity_tir4 = addtuple ship.velocity (polar_to_affine (((Random.float 1.) -. 0.5) *. projectile_deviation +. ship.orientation) (projectile_min_speed +. Random.float (projectile_max_speed -. projectile_min_speed))) in
    let velocity_tir5 = addtuple ship.velocity (polar_to_affine (((Random.float 1.) -. 0.5) *. projectile_deviation +. ship.orientation) (projectile_min_speed +. Random.float (projectile_max_speed -. projectile_min_speed))) in
    (*Cinq projectiles*)
    let projectile1 = ref (spawn_projectile position_tir velocity_tir1) in
    let projectile2 = ref (spawn_projectile position_tir velocity_tir2) in
    let projectile3 = ref (spawn_projectile position_tir velocity_tir3) in
    let projectile4 = ref (spawn_projectile position_tir velocity_tir4) in
    let projectile5 = ref (spawn_projectile position_tir velocity_tir5) in
    (* Muzzleflash. On utilise List.append pour que le Muzzleflash soit au dessus de la fumée déjà en jeu.*)
    if smoke then etat.ref_smoke <- List.append etat.ref_smoke [spawn_muzzle projectile1];

    (*On ajoute les cinq projectiles*)
    etat.ref_projectiles <- List.append [projectile1 ; projectile2 ; projectile3 ; projectile4 ; projectile5] etat.ref_projectiles;

    etat.cooldown <- etat.cooldown +. projectile_cooldown;
    ship.velocity <- addtuple ship.velocity (polar_to_affine (ship.orientation +. pi) projectile_recoil);

    etat.ref_ship := ship;
    ref_etat := etat;
    etat_suivant ref_etat;
) else ();;


let random_teleport ref_etat =
  let etat = !ref_etat in
  let ship = !(etat.ref_ship) in
  ship.position <- (Random.float phys_width, Random.float phys_height);
  ship.velocity <- (0.,0.);
  etat.ref_ship := ship;
  ref_etat:=etat;;


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
let rec mort ref_etat =(*
  tir ref_etat;*)
  rotation_droite ref_etat;
  acceleration ref_etat;
  !(!ref_etat.ref_ship).mass <- 100000.;
  game_speed_target := 0.2;
  game_exposure_target := 0.5;
  if key_pressed  ()then
  let status = wait_next_event[Key_pressed] in
    match status.key  with (* ...en fonction de la touche frappee *)
    | 'r' -> ref_etat := init_etat (); (*R permet de recommencer une partie de zéro rapidement. TODO : Le faire fonctionner*)
    | 'q' -> print_endline "Bye bye!"; exit 0 (* on quitte le jeu *)
    | _ -> etat_suivant ref_etat; mort ref_etat;;

(* --- boucle d'interaction --- *)

let rec boucle_interaction ref_etat =
  game_speed_target := game_speed_target_boucle;
  game_exposure_target := game_exposure_target_boucle;

  if !(!ref_etat.ref_ship).health<0. then
    mort ref_etat
  else
    if mousecontrol then controle_souris ref_etat;
    if key_pressed () then
    let status = wait_next_event[Key_pressed] in
      match status.key  with (* ...en fonction de la touche frappee *)
      | 'r' -> ref_etat := init_etat (); boucle_interaction ref_etat (*R permet de recommencer une partie de zéro rapidement. TODO : Le faire fonctionner*)
      | 'v' -> strafe_left ref_etat; boucle_interaction ref_etat (*strafe vers la gauche *)
      | 'd' -> boost_gauche ref_etat; boucle_interaction ref_etat (* rotation vers la gauche *)
      | 'l' -> boost ref_etat;boucle_interaction ref_etat (* acceleration vers l'avant *)
      | 'j' -> boost_droite ref_etat;boucle_interaction ref_etat (* rotation vers la droite *)
      | 'z' -> strafe_right ref_etat; boucle_interaction ref_etat (*strafe vers la droite *)
      | 't' -> random_teleport ref_etat; boucle_interaction ref_etat
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
(*set_text_size ne semble être implémenté correctement sur aucun système, est-ce depuis de nombreuses années. On fera sans.*)
(*set_text_size (int_of_float (10. *. ratio_rendu));*)

  (* initialisation de l'etat du jeu *)
  let ref_etat = ref (init_etat ()) in

(*On s'assure d'avoir un repère temporel correct*)
  time_last_frame := Unix.gettimeofday();
  time_current_frame := Unix.gettimeofday();
  etat_suivant ref_etat;
  affiche_etat ref_etat;
  boucle_interaction ref_etat;; (* lancer la boucle d'interaction avec le joueur *)

let _ = main ();; (* demarrer le jeu *)
