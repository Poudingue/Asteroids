open Graphics;;
#load "unix.cma";;
#load "graphics.cma";;

(*Certaines valeurs par défaut ne suivent pas les instructions du tp pour une meilleure expérience de jeu.*)
(*Ces changements sont documentés dans les commentaires et peuvent être remis aux valeurs du pdf si nécessaire.*)



(******************************************************************************)
(*Paramètres graphiques*)


(*Paramètres temporels*)

(*Le game_speed_target est la vitesse à laquelle on veut que le jeu tourne en temps normal*)
let game_speed_target = 1.0 ;;
(*Le game_speed est la vitesse réelle à laquelle le jeu tourne à l'heure actuelle.*)
(*Cela permet notamment de faire des effets de ralenti ou d'accéléré*)
let game_speed = ref 1.;;
(*Le game_speed_change détermine à quelle «vitesse» le game speed se rapproche de game_speed_target (en ratio par seconde)*)
let game_speed_change = 0.8;;
(*Le temps propre de l'observateur. *)
(*En l'occurrence, on récupère celui du vaisseau.*)
(*Cela permet d'avoir une relativité Einsteinienne.*)
let observer_proper_time = ref 1.0;;(*En ratio du temps «absolu» de l'univers*)

(*Le framerate demandé dans l'énoncé est de 20*)
(*Un framerate plus élevé offre une meilleure expérience de jeu :*)
(*Des contrôles plus réactifs, un meilleur confort visuel, et une physique plus précise.*)
(*Bien sûr, il est possible de le changer ci-dessous :)*)
let framerate_limit = 60.;;

(*On stocke le moment auquel la dernière frame a été calculée
pour synchroniser correctement le moment de calcul de la frame suivante*)
let time_last_frame = ref 0.;;
let time_current_frame = ref 0.;;


(*Dimensions fenêtre graphique.*)

let width = 1280;;
let height = 500;;

(*Dimensions de l'espace physique dans lequel les objets évoluent.*)
(*On s'assure que la surface de jeu soit la même quelle que soit la résolution.*)
(*On conserve au passage le ratio de la résolution pour les dimensions de jeu*)
(*On a une surface de jeu de 1 000 000*)
let ratio_rendu = sqrt ((float_of_int width) *. (float_of_int height) /. 1000000.);;
let phys_width = float_of_int width /. ratio_rendu;;
let phys_height = float_of_int height /. ratio_rendu;;


(*L'antialiasing de dither fait «trembler» l'espace de rendu*)
(*afin de compenser la perte de précision due à la rastérisation*)
(*lors du placement des objets et du tracé des contours.*)
let dither_aa = true;;
(*La puissance du jitter détermine à quel point le rendu peut se décaler.*)
(*Déterminer à 1 ou moins pour éviter un effet de flou et de fatigue visuelle*)
let dither_power = 0.5;;(*En ratio de la taille d'un pixel*)



(******************************************************************************)
(*Paramètres de jeu*)

(*Le mode infinitespace permet de se passer de limites physiques au jeu.*)
let infinitespace = false;;
(*Distance max du ship à laquelle un objet peut exister.*)
(*N'a d'impact sur le jeu qu'en mode infinitespace,*)
(*Dans lequel les objets peuvent continuer d'exister en dehors de l'espace de jeu*)
let objmaxdist = 10.;;

(*Paramètres des astéroïdes*)

(*Taille max d'astéroïde au spawn, en pixels non entiers.*)
let asteroid_max_spawn_radius = 100.;;
(*Taille minimale d'un astéroïde lors du spawn.*)
let asteroid_min_spawn_radius = 80.;;(*En ratio de la taille de spawn max*)
(*En dessous de la taille minimale, un asteroide meurt*)
let asteroid_min_size = 10.;;
(*Rotation max d'un astéroïde au spawn (dans un sens aléatoire)*)
let asteroid_max_moment = 1.;;
(*Sert à déterminer la masse d'un astéroïde en se basant sur son radius, ou l'inverse*)
let asteroid_density = 1.;;
(*Sert à déterminer la health d'un astéroïde basé sur son rayon*)
let asteroid_radius_health = 1.;;


(*Paramètres ship*)

(*Les contrôles directs ne contrôlent pas la vitesse et le moment mais directement la position et la rotation*)
(*Les valeurs par défaut sont celles demandées*)
let ship_direct_pos = false;;
let ship_direct_rotat = true;;

(*valeurs du ship*)
let ship_max_health = 100.;; (*health au spawn. Permet de l'appliquer au modèle physique.*)
let ship_max_healths = 3;; (*Nombre de fois que le vaisseau peut réapparaître*)
let ship_max_depl = 50.;; (*En px.s⁻¹. Utile si contrôle direct du déplacement.*)
let ship_max_accel = 100.;; (*En px.s⁻² Utile si contrôle de l'accélération*)
let ship_friction = 0.;;
let ship_max_tourn = 0.5;; (*En radian.s⁻¹*)
let ship_max_moment = 0.5;; (*En radian.s⁻²*)
let ship_rotat_friction = 0.1;; (*En ratio de la rotation par seconde*)
let ship_mass = 100.;; (*Pour les calculs de recul lors de collisions physiques*)
let ship_radius = 20.;; (*Pour la hitbox*)
let ship_phys_res = 10.;;
let ship_dam_res =1.;;
let ship_death_damages = 50.;;
let ship_death_radius = 500.;;

(*Valeurs du projectile*)
let projectile_cooldown = 0.5;;
let projectile_max_vitesse = 500.;;(*Vitesse relative au lanceur lors du lancement*)
let projectile_damage = 20.;;
let projectile_radius = 5.;;
let projectile_mass = 1.;;(*Pour calculate le recul du vaisseau lors du tir*)
let projectile_health = 0.;;(*On considère la mort quand la santé descend sous zéro. On a ici la certitude que le projectile se détruira*)


(*Paramètres caméra*)

(*Ces options sont utiles dans tous les modes*)
let camera_screenshake = true;; (*Effet de secousse d'écran lors de chocs*)
let camera_screenshake_physics_ship = 1.;; (*En ratio des dégats physiques subis par le vaisseau*)
let camera_screenshake_physics_other = 0.1;; (*En ratio des dégats physiques subis par d'autres objets*)
let camera_screenshake_damage = 1.;; (*En ratio des dégats bruts.*)
let camera_screenshake_destruction = 0.1;; (*En ratio de la mass totale disparue lors de la destruction*)
let camera_screenshake_death_ship = 10.;; (*En ratio de la mass totale disparue*)

let camera_accel_max = 0.5;; (*En ratio de la distance entre centre et objectif*)
let camera_friction = 0.01;; (*En ratio de la vitesse*)

(*Ces options de camera ne sont utiles qu'en mode infinitespace*)
(*La camera predictive oriente la camera vers l'endroit où le vaisseau va*)
let camera_prediction = 1.;; (*En secondes de déplacement dans le futur.*)
let camera_zoom_min = 0.1;; (*En grossissement*)
let camera_zoom_max = 10.;; (*En grossissement*)
let vitesse_zoom = 0.5;; (*En ratio de la différence entre zoom actuel et zoom voulu par seconde.*)

(******************************************************************************)
(*Définition types pour état du jeu*)

(*Système de caméra*)

type camera = {
  position : (float*float);
  velocity : (float*float); (*La vitesse actuelle déplace la position de la caméra*)
  position_target : (float*float);
  zoom : float;
  zoom_target : float;
};;


(*On pourrait ajouter des types différents, par exemple des astéroïdes à tête chercheuse, des vaisseaux ennemis…*)
(*À faire plus tard si le temps*)
type type_object = Asteroid | Projectile | Ship ;;

type objet_physique = {
    (*La valeur spawned est fausse tant qu'un objet spawné n'a pas son centre dans l'espace de jeu.*)
    (*Et du système de rendu qui rend les objets à la fois en haut et en bas.*)
    mutable spawned : bool;
    objet : type_object;
    (*TODO : Si le temps, définir une hitbox autre que circulaire*)
    radius : float;
    mass : float;
    mutable health : float;
    (*Réduction des dégats bruts.*)
    resistance_damage : float;
    (*Réduction des dégats physiques, pour que les collisions à faible vitesse ne fassent pas de dégats*)
    resistance_physics : float;

    (*damage infliges en plus des interactions du modele physique*)
    (*ne fonctionne correctementy que si les radiuss sont supérieurs aux radiuss physiques de l'objet*)
    damage_contact : float;
    contact_radius : float;
    (*Les objets peuvent infliger des dégats lors de leur destruction. Par exemple un missile.*)
    damage_death : float;
    radius_death : float;

    (*L'inertie*masse se communique d'un objet à l'autre lors des collisions*)
    (*Les objets ne subissent pas d'accélération, on change directement leur inertie pour des raisons de simplicité*)
    mutable position : (float*float);(*En pixels non entiers*)
    (*On stocke l'inertie en tuples, les calculs sont plus simples que direction + vitesse, aussi bien pour l'humain que pour la machine.*)
    mutable velocity : (float*float);(*En pixels.s⁻¹*)
    (*La friction est simplement une accélération négative s'opposant à l'inertie. Ralentit progressivement les objets par rapport au terrain.*)
    (*Une friction de 0 n'a aucun effet, une friction de 1 imobilise un objet.*)
    mutable friction : float;(*En ratio de l'inertie par seconde.*)

    (*orientation en radians, moment en radians.s⁻¹*)
    mutable orientation : float;
    mutable moment : float;
    (*Friction de la rotation, en ratio du moment*)
    friction_moment : float;
    proper_time : float;

    color : color;
};;


(*Définition fonctions générales*)

(*Pi*)
let pi = 4. *. atan 1. ;;

(*Fonction de carre, pour écrire plus jolimment les formules de pythagore*)
let carre v = v *. v;;

(*Toujours pratique pour faire du joli pythagore*)
let hypothenuse (x, y) = sqrt (carre x +. carre y);;

(*Permet l'addition de deux tuples*)
let addtuple (x1, y1) (x2, y2) = (x1 +. x2, y1 +. y2);;

(*Permet la multiplication d'un tuple par un float*)
let multuple (x, y) ratio = (x *. ratio, y *. ratio);;


let dither fl =
  int_of_float (fl +. Random.float dither_power);;


(*Permet l'addition de deux tuples, en poundérant le second par le ratio*)
let proj (x1, y1) (x2, y2) ratio = addtuple (x1, y1) (multuple (x2, y2) ratio);;

(*Transfert d'un vecteur en angle*valeur en x*y*)
let polar_to_affine angle valeur = (valeur *. cos angle, valeur *. sin angle);;

(*Transfert d'un vecteur en x*y en angle*valeur *)
let affine_to_polar (x, y) =
let r = hypothenuse (x, y) in
if r = 0. then (0., 0.) (*Dans le cas où le rayon est nul, on ne peut pas déterminer d'angle donné*)
else (r, 2. *. atan (y /. (x +. r)));;

(*La fonction distancecarre est plus simple niveau calcul qu'une fonction distance,*)
(*Car on évite la racine carrée, mais n'en reste pas moins utile pour les hitbox circulaires*)
let distancecarre (x1, y1) (x2, y2) = carre (x2 -. x1) +. carre (y2 -. y1);;

let modulo_float value modulo = if value < 0. then value +. modulo else if value >= modulo then value -. modulo else value;;

let modulo_reso (x, y) = (modulo_float x phys_width, modulo_float y phys_height);;

  let render_objet ref_objet =
  let objet = !ref_objet in
  set_color objet.color;
  let (x,y) = multuple objet.position ratio_rendu in
  fill_circle (dither x) (dither y) (dither (ratio_rendu *. objet.radius));(*Dessiner au centre de l'écran*)
  (*Lorsqu'on est pas en mode infinispace, les objets allant d'un côté de l'écran repartent de l'autre*)
  if (infinitespace = false) then
  (*Dessiner les modulos de l'objet des cotés*)
    fill_circle ((dither x) + width) (dither y)  (dither (ratio_rendu *. objet.radius));
    fill_circle ((dither x) - width) (dither y)  (dither (ratio_rendu *. objet.radius));
    fill_circle (dither x) ((dither y) + height) (dither (ratio_rendu *. objet.radius));
    fill_circle (dither x) ((dither y) - height) (dither (ratio_rendu *. objet.radius));
  (*Dessiner les modulos dans les angles*)
    fill_circle ((dither x) + width) ((dither y) + height)  (dither (ratio_rendu *. objet.radius));
    fill_circle ((dither x) + width) ((dither y) - height)  (dither (ratio_rendu *. objet.radius));
    fill_circle ((dither x) - width) ((dither y) + height)  (dither (ratio_rendu *. objet.radius));
    fill_circle ((dither x) - width) ((dither y) - height)  (dither (ratio_rendu *. objet.radius));


  set_color white;
  let (x2, y2) = multuple (polar_to_affine objet.orientation objet.radius) ratio_rendu in
  Graphics.draw_segments (Array.of_list [dither x, dither y, dither (x +. x2),dither (y +. y2)]);;

  let render_objets ref_objets = List.iter render_objet ref_objets;;


(* États, positions, déplacements, etc… *)

type etat = {
  mutable score : int;
  (*Le cooldown est le temps restant avant de pouvoir de nouveau tirer*)
  mutable cooldown : float;
  mutable ref_ship : objet_physique ref;
(*Une liste de références est plus facile à utiliser avec les fonctions de liste qu'une référence de liste*)
  mutable ref_objets : objet_physique ref list;
};;

(*Fonction déplaçant un objet selon une vélocitée donnée.*)
(*On tient compte du framerate et de la vitesse de jeu,*)
(*mais également du temps propre de l'objet et de l'observateur*)
let deplac_objet ref_objet (dx, dy) =
let objet = !ref_objet in
objet.position <- modulo_reso (proj objet.position (dx, dy) ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time));
ref_objet := objet;;

(*Fonction accélérant un objet selon une accélération donnée.*)
(*On tient compte du framerate et de la vitesse de jeu,*)
(*mais également du temps propre de l'objet et de l'observateur*)
let accel_objet ref_objet (ddx, ddy) =
let objet = !ref_objet in
objet.velocity <- proj objet.velocity (ddx, ddy) ((!time_current_frame -. !time_last_frame) *. !game_speed *. !observer_proper_time /. objet.proper_time);
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

(*Fonction de calcul de changement de position inertiel d'un objet physique.*)
let inertie_objet ref_objet = deplac_objet ref_objet (!ref_objet).velocity;;

(*On calcule le changement de position inertiel de tous les objets en jeu*)
let inertie_objets ref_objets =
List.iter inertie_objet ref_objets;;

(*On calcule l'inertie en rotation des objets*)
let moment_objet ref_objet = rotat_objet ref_objet (!ref_objet).moment;;

(*D'un groupe d'objets*)
let moment_objets ref_objets = List.iter moment_objet ref_objets;;

(*La racine carrée est une opération assez lourde,*)
(*Donc plutôt que de comparer la distance entre deux objets avec la somme de leur radius,*)
(*On compare le carré de leur distance avec le carré de la somme de leurs radiuss.*)
(*On travaille par hitbox circulaire pour 1-La simplicité du calcul 2-La proximité avec les formes réelles*)

(*Fonction vérifiant la collision entre deux objets*)
let collision objet1 objet2 = distancecarre objet1.position objet2.position < carre (objet1.radius +. objet2.radius);;

(*Fonction appelée en cas de collision de deux objets.*)
(*Conséquences à compléter et améliorer*)
(*TODO*)
let consequences_collision ref_objet1 ref_objet2 =
  let objet1 = !ref_objet1 in
  let objet2 = !ref_objet2 in
  if objet1.spawned && objet2.spawned then
  let obj1cinet = multuple objet1.velocity objet1.mass in
  let total_phys_damage = (hypothenuse objet1.velocity) *. objet1.mass +. (hypothenuse objet2.velocity) *. objet2.mass in
  objet1.velocity <- multuple objet2.velocity objet2.mass;
  objet2.velocity <- obj1cinet;
  objet1.health <- objet1.health +. max 0. (total_phys_damage -. objet1.resistance_physics);
  objet2.health <- objet2.health +. max 0. (total_phys_damage -. objet2.resistance_physics);
  ref_objet1 := objet1;
  ref_objet2 := objet2;
else ();;

(*Fonction vérifiant la collision entre un objet et les autres objets*)
(*Dès la première collision détectée, déclencher les conséquences, on considère qu'un objet ne peut avoir qu'une collision à la fois*)
let rec calculate_collisions_objet ref_objet ref_objets =
if List.length ref_objets < 2 then ()
else if collision !ref_objet !(List.hd ref_objets)
then consequences_collision ref_objet (List.hd ref_objets)
else calculate_collisions_objet ref_objet (List.tl ref_objets);;

let rec calculate_collisions_objets ref_objets =
if List.length ref_objets < 2 then ()
else calculate_collisions_objet (List.hd ref_objets) (List.tl ref_objets);
calculate_collisions_objets (List.tl ref_objets);;


(* --- initialisations etat --- *)

(* A DEFINIR : generation positions, deplacements initiaux ... *)

let spawn_ship = {
    spawned = false;
    objet = Ship;
    radius = ship_radius;
    mass = ship_mass;
    health = ship_max_health;
    resistance_damage = ship_dam_res;
    resistance_physics = ship_phys_res;
    damage_contact = 0.;
    contact_radius = 0.;
    damage_death = ship_death_damages ;
    radius_death = ship_death_radius ;
    position = (phys_width /. 2., phys_height /. 2.);
    velocity = (0.,0.);
    friction = ship_friction;
    orientation = pi /. 2.;
    moment = 0.;
    friction_moment = ship_rotat_friction;
    proper_time = 1.;
    color = red ;
};;


let spawn_projectile (x,y) (dx,dy) orientation = {
    spawned = false;
    objet = Projectile;
    radius = projectile_radius;
    mass = projectile_mass;
    health = projectile_health;
    resistance_damage = 0.;
    resistance_physics = 0.;
    damage_contact = 0.;
    contact_radius = 0.;
    damage_death = projectile_damage;
    radius_death = projectile_radius;(*Pour l'instant, le missile ne fera de dégats qu'aux objets avec lesquels il est en collision*)
    position = (x,y);
    velocity = (dx, dy);
    friction = 0.;
    orientation = orientation;
    moment = 0.;
    friction_moment = 0.;
    proper_time = 0.;

    color = yellow;
};;

let spawn_asteroid (x, y) (dx, dy) radius = {
    spawned = false;
    objet = Asteroid;
    radius = radius;
    mass = radius *. asteroid_density;
    health = radius *. asteroid_radius_health;
    resistance_damage = 0.;
    resistance_physics = 0.;
    damage_contact = 0.;
    contact_radius = 0.;
    damage_death = 0.;
    radius_death = 0.;
    position = (x, y);
    velocity = (dx, dy);
    friction = 0.;
    orientation = Random.float (2. *. pi);
    moment = Random.float (2. *. asteroid_max_moment) -. asteroid_max_moment ;
    friction_moment = 0.;
    proper_time = 1.;

    color = rgb (64 + Random.int 64) (64 + Random.int 64) (64 + Random.int 64);
};;

let spawn_random_asteroid ref_etat =
let etat = !ref_etat in
etat.ref_objets <- (ref(spawn_asteroid (Random.float phys_width, Random.float phys_height) ((Random.float 10.) -. 5., (Random.float 10.) -. 5.) ( asteroid_min_spawn_radius +. (Random.float (asteroid_max_spawn_radius -. asteroid_min_spawn_radius))) )) :: etat.ref_objets;
ref_etat := etat ;;



let init_etat = {
score=0;
cooldown = 0.;
ref_ship = ref spawn_ship;
ref_objets = [];
};;



(* --- changements d'etat --- *)


let affiche_etat ref_etat =
  set_color (rgb 0 0 64);
  fill_rect 0 ~-1 width height;
  render_objet !ref_etat.ref_ship;
  render_objets !ref_etat.ref_objets;
  set_color red;
  (*fill_circle (Random.int width) (Random.int height) (Random.int 300);*)
  synchronize ();;


  (* calcul de l'etat suivant, apres un pas de temps *)
  (* Cette fonction est de type unit, elle modifie l'etat mais ne rend rien*)
let etat_suivant ref_etat =
  (*On calcule tous les déplacements naturels dus à l'inertie des objets*)
  time_last_frame := !time_current_frame;
  time_current_frame := Unix.gettimeofday();

  inertie_objets !ref_etat.ref_objets;
  inertie_objet !ref_etat.ref_ship;

  moment_objets !ref_etat.ref_objets;
  moment_objet !ref_etat.ref_ship;

  (*On calcule les collisions avec le vaisseau seulement après les autres objets,*)
  (*car dans le cas exceptionnel où un objet est détruit par une autre collision
  avant de toucher le vaisseau, cela permet au joueur d'être sauvé in extremis*)
  (*et cela participe à une expérience de jeu plaisante.*)

(*Fonction retournant un problème de tl pour l'instant
  calculate_collisions_objets !ref_etat.ref_objets;
*)

  calculate_collisions_objet !ref_etat.ref_ship !ref_etat.ref_objets;
  (*if Random.float framerate_limit < 1. then spawn_random_asteroid ref_etat;*)
  affiche_etat ref_etat;
  let elapsed_time = !time_current_frame -. !time_last_frame in
(*Équivalent bidouillé de sleepf en millisecondes, pour que le programme fonctionne aussi avec les anciennes versions d'Ocaml*)
  ignore (Unix.select [] [] [] (max 0. (1. /. (framerate_limit -. elapsed_time))));;


(* acceleration du vaisseau *)
let acceleration ref_etat =
if ship_direct_pos then
deplac_objet  !ref_etat.ref_ship (polar_to_affine (!(!ref_etat.ref_ship).orientation) ship_max_depl)
else
(*Dans le cas d'un contrôle de la vélocité et non de la position.*)
(*C'est à dire en respectant le TP, et c'est bien mieux en terme d'expérience de jeu :) *)
accel_objet !ref_etat.ref_ship (polar_to_affine (!(!ref_etat.ref_ship).orientation) ship_max_accel);
etat_suivant ref_etat;;

(* rotation vers la gauche et vers la droite du ship *)
let rotation_gauche ref_etat =
if ship_direct_rotat then
rotat_objet !ref_etat.ref_ship ship_max_tourn
else
(*Dans le cas d'un contrôle de la du couple et non de la rotation.
Non recommandé de manière générale*)
couple_objet !ref_etat.ref_ship ship_max_tourn;
etat_suivant ref_etat;;

let rotation_droite ref_etat =
if ship_direct_rotat then
rotat_objet !ref_etat.ref_ship (0. -. ship_max_tourn)
else
(*Dans le cas d'un contrôle de la du couple et non de la rotation.
Non recommandé de manière générale*)
couple_objet !ref_etat.ref_ship (0. -. ship_max_tourn);
etat_suivant ref_etat;;

(* tir d'un nouveau projectile *)
let tir ref_etat =
let etat = !ref_etat in
etat.ref_objets <- (ref (spawn_projectile !(etat.ref_ship).position !(etat.ref_ship).velocity !(etat.ref_ship).orientation)) :: etat.ref_objets ;
etat.cooldown <- projectile_cooldown;
ref_etat := etat;
etat_suivant ref_etat;;


(* --- affichages graphiques --- *)

(* fonctions d'affichage du ship, d'un asteroide, etc. *)




(* --- boucle d'interaction --- *)

let rec boucle_interaction ref_etat =
  let status = wait_next_event[Poll] in
  if status.keypressed then begin
    match read_key () with (* ...en fonction de la touche frappee *)
     | 'u' -> rotation_gauche ref_etat; boucle_interaction ref_etat (* rotation vers la gauche *)
     | 'p' -> acceleration ref_etat;boucle_interaction ref_etat (* acceleration vers l'avant *)
     | 'e' -> rotation_droite ref_etat;boucle_interaction ref_etat (* rotation vers la droite *)
     | ' ' -> tir ref_etat;boucle_interaction ref_etat (* tir d'un projectile *)
     | 'q' -> print_endline "Bye bye!"; exit 0 (* on quitte le jeu *)
     | _ -> etat_suivant ref_etat;boucle_interaction ref_etat
end else
etat_suivant ref_etat;
  boucle_interaction ref_etat;; (* on se remet en attente de frappe clavier *)

(* --- fonction principale --- *)

let main () =
  (* initialisation du generateur aleatoire *)
  Random.self_init ();
  (* initialisation de la fenetre graphique et de l'affichage *)
  open_graph (" " ^ string_of_int width ^ "x" ^ string_of_int height);
  auto_synchronize false;
  (* initialisation de l'etat du jeu *)
  let ref_etat = ref init_etat in
(*On s'assure d'avoir un repère temporel correct*)
  time_last_frame := Unix.gettimeofday();
  time_current_frame := Unix.gettimeofday();
  etat_suivant ref_etat;
  affiche_etat ref_etat;
  boucle_interaction ref_etat;; (* lancer la boucle d'interaction avec le joueur *)

let _ = main ();; (* demarrer le jeu *)
