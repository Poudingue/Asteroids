open Graphics;;

#load "graphics.cma";;

(*Certaines valeurs par défaut ne suivent pas les instructions du tp pour une meilleure expérience de jeu.*)
(*Ces changements sont documentés dans les commentaires et peuvent être remis aux valeurs du pdf si nécessaire.*)



(******************************************************************************)
(*Paramètres graphiques*)


(*Paramètres temporels*)

(*Le game_speed_target est la vitesse à laquelle on veut que le jeu tourne en temps normal*)
let game_speed_target = 1. ;;
(*Le game_speed est la vitesse réelle à laquelle le jeu tourne à l'heure actuel.*)
(*Cela permet notamment de faire des effets de ralenti ou d'accéléré*)
let game_speed = 0.5 ;;
(*Le game_speed_change détermine à quelle «vitesse» le game speed se rapproche de game_speed_target (en ratio par seconde)*)
let game_speed_change = 0.8;;
(*Le temps propre de l'observateur. *)
(*En l'occurrence, on récupère celui du ship.*)
(*Cela permet d'avoir une relativité Einsteinienne.*)
let observer_proper_time = 1.0;;(*En ratio du temps «absolu» de l'univers*)

(*Le framerate demandé dans l'énoncé est de 20*)
(*Un framerate plus élevé offre une meilleure expérience de jeu :*)
(*Des contrôles plus réactifs, un meilleur confort visuel, et une physique plus précise.*)
let framerate_limit = 60. ;;


(*Dimensions fenêtre graphique.*)

let width = 1280;;
let height = 720;;

(*Dimensions de l'espace physique dans lequel les objets évoluent.*)
(*On s'assure que la surface de jeu soit la même quelle que soit la résolution.*)
(*On conserve au passage le ratio de la résolution pour les dimensions de jeu*)
(*On a une surface de jeu de 1 000 000*)
let phys_width = 1000. *. float_of_int width /. float_of_int height;;
let phys_height = 1000. *. float_of_int height /. float_of_int width;;


(*L'antialiasing de jitter fait «trembler» l'espace de rendu*)
(*afin de compenser la perte de précision de la rastérisation*)
(*dans le placement des objets et le tracé des contours.*)
let jitter_aa = true;;
(*La puissance du jitter détermine à quel point le rendu peut se décaler.*)
(*Déterminer à 1 ou moins pour éviter un effet de flou et de fatigue visuelle*)
let jitter_aa_power = 1.;;(*En ratio de la taille d'un pixel*)



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
let asteroid_min_spawn_radius = 0.8;;(*En ratio de la taille de spawn max*)
(*En dessous de la taille minimale, un asteroide meurt*)
let asteroid_min_size = 10.;;
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
let ship_max_accel = 20.;; (*Toujours en px.s⁻¹. px.s⁻² si mode accélération*)
let ship_max_tourn = 6.283;; (*En radian par seconde.*)
let ship_mass = 100.;; (*Pour les calculs de recul lors de collisions physiques*)
let ship_radius = 20.;; (*Pour la hitbox*)

(*Valeurs du projectile*)
let projectile_max_vitesse = 500.;;(*Vitesse relative au lanceur lors du lancement*)
let projectile_damage = 20.;;
let projectile_radius = 5.;;
let projectile_mass = 1.;;(*Pour calculate le recul du vaisseau lors du tir*)


(*Paramètres caméra*)

(*Ces options sont utiles dans tous les modes*)
let camera_screenshake = true;; (*Effet de secousse d'écran lors de chocs*)
let camera_screenshake_physics_ship = 1.;; (*En ratio des dégats physiques subis par le vaisseau*)
let camera_screenshake_physics_other = 0.1;; (*En ratio des dégats physiques subis par d'autres objets*)
let camera_screenshake_damage = 1.;; (*En ratio des dégats bruts.*)
let camera_screenshake_destruction = 0.1;; (*En ratio de la mass totale disparue lors de la destruction*)
let camera_screenshake_death_ship = 10.;; (*En ratio de la mass totale disparue*)

let camera_accel_max = 0.1;; (*En ratio de la distance entre centre et objectif*)
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
    spawned : bool;
    objet : type_object;
    (*TODO : Si le temps, définir une hitbox autre que circulaire*)
    radius : float;
    mass : float;
    health : float;
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
    orientation : float;
    moment : float;
    (*Friction de la rotation, en ratio du moment*)
    friction_moment : float;
    proper_time : float;

    color : color;
};;



(*Fonction de carre, pour écrire plus jolimment les formules de pythagore*)
let carre v = v *. v;;

(*Toujours pratique pour faire du joli pythagore*)
let hypothenuse (x, y) = sqrt (carre x +. carre y);;

(*Permet l'addition de deux tuples, en poundérant le second par le ratio*)
let proj (x1, y1) (x2, y2) ratio = (x1 +. ratio *. x2, y1 +. ratio *. y2);;

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

(* États, positions, déplacements, etc… *)

type etat = {
  score : int;
  (*Le cooldown est le temps restant avant de pouvoir de nouveau tirer*)
  cooldown : float;
  ship : objet_physique;
  objets : objet_physique list;
};;

(*Fonction déplaçant un objet selon une vélocitée donnée.*)
(*On tient compte du framerate et de la vitesse de jeu,*)
(*mais également du temps propre de l'objet et de l'observateur*)
let deplac_objet objet (x, y) = objet.position <- proj objet.position (x, y) ((game_speed /. framerate_limit) *. (objet.proper_time /. observer_proper_time));;

(*Fonction accélérant un objet selon une accélération donnée.*)
(*On tient compte du framerate et de la vitesse de jeu,*)
(*mais également du temps propre de l'objet et de l'observateur*)
let accel_objet objet (x, y) = objet.velocity <- proj objet.velocity (x, y) ((game_speed /. framerate_limit) *. (objet.proper_time /. observer_proper_time));;

(*Fonction de calcul de changement de position inertiel d'un objet physique.*)
let inertie_objet objet = deplac_objet objet objet.velocity;;

(*On calcule le changement de position de tous les objets en jeu*)
let inertie_objects objets =  List.iter inertie_objet objets;;

(*On obtient le carré de la distance avec le théorème de pythagore*)
(*a²+b²=c²*)
(*La racine carrée est une opération assez lourde,*)
(*Donc plutôt que de comparer la distance entre deux objets avec la somme de leur radius,*)
(*On compare le carré de leur distance avec le carré de la somme de leurs radiuss.*)
(*On travaille par hitbox circulaire pour 1-La simplicité du calcul 2-La proximité avec les formes réelles*)

(*Fonction vérifiant la collision entre deux objets*)
let collision objet1 objet2 = distancecarre objet1.position objet2.position < carre (objet1.radius +. objet2.radius);;

(*Fonction appelée en cas de collision de deux objets.*)
(*Conséquences à déterminer.*)
(*TODO*)
let consequences_collision objet1 objet2 = objet1.velocity <- objet2.velocity;;(*Faire les conséquences*)

(*Fonction vérifiant la collision entre un objet et les autres objets*)
(*Dès la première collision détectée, déclencher les conséquences, on considère qu'un objet ne peut avoir qu'une collision à la fois*)
let rec calculate_collisions_objet objet objets =
if List.length objets = 0 then ()
else if collision objet (List.hd objets) then consequences_collision objet (List.hd objets)
else calculate_collisions_objet objet (List.tl objets);;

let rec calculate_collisions_objets objets =
if List.length objets <= 1 then ()
else calculate_collisions_objet (List.hd objets) (List.tl objets);
  calculate_collisions_objets (List.tl objets);;


(* --- initialisations etat --- *)

(* A DEFINIR : generation positions, deplacements initiaux ... *)

let init_objet = {

}

let init_etat = {
score=0;
cooldown = 0.;
ship = init_ship;
objets = [] list;
};;

(* --- changements d'etat --- *)

(* acceleration du vaisseau *)
let acceleration etat =
if ship_direct_pos then
(*Dans le cas du contrôle direct de la position, ce qui est pas génial*)
deplac_objet etat.ship (polar_to_affine etat.ship.orientation max_accel)
else
(*Dans le cas d'un contrôle de la vélocité et non de la position.*)
(*C'est à dire en respectant le TP, et c'est bien mieux en terme d'expérience de jeu :) *)
accel_objet etat.ship (polar_to_affine etat.ship.orientation max_accel);;

(* rotation vers la gauche et vers la droite du ship *)
let rotation_gauche etat = etat;; (* A REDEFINIR *)
let rotation_droite etat = etat;; (* A REDEFINIR *)

(* tir d'un nouveau projectile *)
let tir etat = etat;; (* A REDEFINIR *)

(* calcul de l'etat suivant, apres un pas de temps *)
(* Cette fonction est de type unit, elle modifie l'etat mais ne rend rien*)
let etat_suivant etat =
  calculate_positions etat.objets;
  calculate_position etat.ship;
  calculate_collisions_objets etat.objets;
(*On calcule les collisions avec ship seulement après les autres objets,*)
(*car dans le cas exceptionnel où un objet est détruit par une autre collision*)
(*avant de toucher le ship, cela permet au joueur d'être sauvé in extremis*)
(*et cela participe à une expérience de jeu plaisante.*)
  calculate_collisions_objet etat.ship etat.objets;
  etat;;

(* --- affichages graphiques --- *)

(* fonctions d'affichage du ship, d'un asteroide, etc. *)

let affiche_etat etat = ();; (* A REDEFINIR *)


(* --- boucle d'interaction --- *)

let rec boucle_interaction ref_etat =
  let status = wait_next_event [Key_pressed] in (* on attend une frappe clahealthr *)
  let etat = !ref_etat in (* on recupere l'etat courant *)
  let nouvel_etat = etat_suivant etat;(* on definit le nouvel etat... *)
    match status.key with (* ...en fonction de la touche frappee *)
    | '1' | 'j' -> rotation_gauche etat (* rotation vers la gauche *)
    | '2' | 'k' -> acceleration etat (* acceleration vers l'avant *)
    | '3' | 'l' -> rotation_droite etat (* rotation vers la droite *)
    | ' ' -> tir etat (* tir d'un projectile *)
    | 'q' -> print_endline "Bye bye!"; exit 0 (* on quitte le jeux *)
    | _ -> etat in (* sinon, rien ne se passe *)
  ref_etat := nouvel_etat; (* on enregistre le nouvel etat *)
  boucle_interaction ref_etat;; (* on se remet en attente de frappe clahealthr *)

(* --- fonction principale --- *)

let main () =
  (* initialisation du generateur aleatoire *)
  Random.self_init ();
  (* initialisation de la fenetre graphique et de l'affichage *)
  open_graph (" " ^ string_of_int width ^ "x" ^ string_of_int height);
  Graphics.background = black;
  foreground white;
  auto_synchronize false;
  (* initialisation de l'etat du jeu *)
  let ref_etat = ref (init_etat ()) in
  (* programmation du refraichissement periodique de l'etat du jeu et de son affichage *)
  let _ = Unix.setitimer Unix.ITIMER_REAL
    { Unix.it_interval = 1. /. framerate_limit ; (* tous les 1/20eme de seconde par défaut. *)
      Unix.it_value = 1. /. framerate_limit } in
  Sys.set_signal Sys.sigalrm
    (Sys.Signal_handle (fun _ ->
      affiche_etat !ref_etat; (* ...afficher l'etat courant... *)
      synchronize ();
      ref_etat := etat_suivant !ref_etat)); (* ...puis calculate l'etat suivant *)
  boucle_interaction ref_etat;; (* lancer la boucle d'interaction avec le joueur *)

let _ = main ();; (* demarrer le jeu *)
