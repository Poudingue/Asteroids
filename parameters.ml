(*Parameters of the whole game*)

open Graphics
let pi = 4. *. atan 1.(*Pi*)

(*Certaines valeurs par défaut ne suivent pas les instructions du tp pour une meilleure expérience de jeu.*)
(*Ces changements sont documentés dans les commentaires et peuvent être remis aux valeurs du pdf si nécessaire.*)

(******************************************************************************)
(*Paramètres affichage*)
let oldschool = ref false
let retro = ref false

(*Paramètres temporels*)

(*Le temps propre de l'observateur.
En l'occurrence, on récupère celui du vaisseau.
Cela permet d'avoir une relativité Einsteinienne.*)
(*TODO s'en servir.*)
let observer_proper_time = ref 1.(*En ratio du temps «absolu» de l'univers*)
(*Le game_speed_target est la vitesse à laquelle on veut que le jeu tourne en temps normal*)
let pause = ref false
let restart = ref false
let quit = ref false
let game_speed_target_pause = 0.  (*Vitesse du jeu en pause*)
let game_speed_target_death = 0.8 (*Vitesse du jeu après mort*)
let game_speed_target_boucle = 1.0 (*Vitesse du jeu par défaut*)
let game_speed_target = ref 1.
(*Le game_speed est la vitesse réelle à laquelle le jeu tourne à l'heure actuelle.*)
(*Cela permet notamment de faire des effets de ralenti ou d'accéléré*)
let game_speed = ref 1.
(*Le half_speed_change détermine à quelle «vitesse» le game speed se rapproche de game_speed_target (En demi-vie) *)
let half_speed_change = 0.5

(*Ratios de changement de vitesse en fonction des évènements*)
let ratio_time_explosion = 0.99
let ratio_time_destr_asteroid = 0.95

(*Timer pour la mort*)
let time_of_death = ref 0.
let time_stay_dead = 5.

(*La limitation de framerate est activable,
mais il semblerait que le gettimeofday et l'attente de Unix.select
ne soient pas assez précis pour que chaque frame dure juste le temps qu'il faut.
Mon conseil est de ne pas l'activer.*)
let locked_framerate = ref false
(*Le framerate demandé dans l'énoncé est de 20.
Un framerate plus élevé offre une meilleure expérience de jeu :
Des contrôles plus réactifs, un meilleur confort visuel, et une physique plus précise.
Bien sûr, il est possible de le changer ci-dessous
(n'a d'effet qu'avec le locked_framerate activé)*)
let framerate_limit = 300.
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
let width = 1360
let height = 760
let game_surface = 30. (*Détermine la taille du terrain de jeu.*)
let infinitespace = ref true
let max_dist = 6000.
(*Dimensions de l'espace physique dans lequel les objets évoluent.
On s'assure que la surface de jeu soit la même quelle que soit la résolution.
On conserve au passage le ratio de la résolution pour les dimensions de jeu
On a une surface de jeu de 1 000 000 par défaut*)
let projectile_number_default = 10

(*Shotgun*)
let shotgun_recoil = 100.
let shotgun_cooldown = 0.3
let shotgun_max_speed = 15000.
let shotgun_min_speed = 10000.
let shotgun_deviation = 0.3
let shotgun_radius = 15.
let shotgun_radius_hitbox = 50.
let shotgun_number = 50

(*Sniper*)
let sniper_recoil = 10000.
let sniper_cooldown = 1.
let sniper_max_speed = 20000.
let sniper_min_speed = 15000.
let sniper_deviation = 0.
let sniper_radius = 25.
let sniper_radius_hitbox = 75.
let sniper_number = 1

(*Machinegun*)
let machine_recoil = 10.
let machine_cooldown = 0.01
let machine_max_speed = 10000.
let machine_min_speed =  8000.
let machine_deviation = 0.2
let machine_radius = 10.
let machine_radius_hitbox = 25.
let machine_number = 1

(*Valeurs des explosions*)
let explosion_max_radius = 100.
let explosion_min_radius = 80.
let explosion_min_exposure = 1.(*Détermine la luminosité max et min des explosions au spawn*)
let explosion_max_exposure = 2.
let explosion_damages = 150.
(*Pour les explosions héritant d'un objet*)
let explosion_ratio_radius = 1.5
let explosion_saturate = 10.
let explosion_min_exposure_heritate = 6.(*Détermine la luminosité max et min des explosions héritant d'objets au spawn*)
let explosion_max_exposure_heritate = 8.

(*Valeurs des muzzleflashes*)
let muzzle_ratio_radius = 3.
let muzzle_ratio_speed = 0.05

(*Valeurs du feu à l'arrière du vaisseau*)
let fire_max_random = 20.
let fire_min_speed = 250.
let fire_max_speed = 500.
let fire_ratio_radius = 1.

(*Valeurs de la fumée*)
let smoke = ref true
let smoke_half_col = 0.2 (*Vitesse de la décroissance de la couleur*)
let smoke_half_radius = 1. (*Vitesse de la décroissance du rayon*)
let smoke_radius_decay = 10. (*Diminution du rayon des particules de fumée*)
let smoke_max_speed = 40.(*Vitesse random dans une direction random de la fumée*)

(*Valeurs des étincelles TODO*)


(*Effet de scanlines pour imiter les moniteurs crt qui projetait l'image ligne par ligne.*)
(*Activer l'effet animated_scanlines permet l'animation imitant les vidéos interlacées,
en activant une ligne sur deux une image sur deux, mais il passe mal
à cause du raffraichissement de l'image ne pouvant pas vraiment être
à 60 pile avec le moteur d'ocaml. Testez à vos risques et périls*)
let scanlines = ref false
let scanlines_period = 5
let animated_scanlines = true
let scanlines_offset = ref 0

(*La camera predictive oriente la camera vers l'endroit où le vaisseau va,
pour le garder tant que possible au centre de l'écran*)
let dynamic_camera = ref true
let camera_prediction = 1.9 (*En secondes de déplacement du vaisseau dans le futur.*)
let camera_half_depl = 1.5 (*Temps pour se déplacer de moitié vers l'objectif de la caméra*)
let camera_ratio_objects = 0.2 (*La caméra va vers la moyenne des positions des objets, pondérés par leur masse et leur distance au carré*)
let camera_ratio_vision = 0.2 (*La caméra va vers là où regarde le vaisseau, à une distance correspondant au ratio x la largeur du terrain*)

(*Le screenshake ajoute des effets de tremblements à l'intensité dépendant  des évènements*)
let screenshake = ref true
let screenshake_smooth = true (*Permet un screenshake moins agressif, plus lisse et réaliste physiquement. Sorte de passe-bas sur les mouvements*)
let screenshake_smoothness = 0.9 (*0 = aucun changement, 0.5 =  1 = lissage infini, screenshake supprimé.*)
let screenshake_tir_ratio = 200.
let screenshake_dam_ratio = 0.01
let screenshake_phys_ratio = 0.01
let screenshake_phys_mass = 20000.(*Masse de screenshake «normal». Des objets plus légers en provoqueront moins, les objets plus lourds plus*)
let screenshake = 0.2
let game_screenshake = ref 0.
let game_screenshake_pos = ref (0.,0.)
let game_screenshake_previous_pos = ref (0.,0.) (*Permet d'avoir un rendu correct des trainées de lumières lors du screenshake*)
(*Utilisation de l'augmentation du score pour faire trembler les chiffres*)
let shake_score = ref 0.
let shake_score_ratio = 0.25
let shake_score_half_life = 1.



(*L'antialiasing de jitter fait «trembler» l'espace de rendu.
C'est une forme de dithering spatial
afin de compenser la perte de précision due à la rastérisation
lors du placement des objets et du tracé des contours.*)
let dither_aa = true
(*La puissance du jitter détermine à quel point le rendu peut se décaler.*)
(*Déterminer à 1 ou moins pour éviter un effet de flou et de fatigue visuelle*)
let dither_power = 0.5 (*En ratio de la taille d'un pixel*)
let dither_power_radius = 0.5
(*Le jitter double courant permet de faire le même jitter sur les positions d'objets.
Cela permet de s'assurer une consistance spatiale dans tout le rendu.*)
let current_jitter_double = ref (0.,0.)


let filter_half_life = 1.
let filter_saturation = 0.5

let space_half_life = 1.

let ratio_rendu = ref (sqrt ((float_of_int width) *. (float_of_int height) /. (game_surface *. 1000000.)))
(*Tailles «physiques» du terrain*)
let phys_width = ref (float_of_int width /. !ratio_rendu)
let phys_height = ref (float_of_int height /. !ratio_rendu)


(******************************************************************************)
(*Paramètres graphiques avancés*)

(*Coleurs random par stage*)
let rand_min_lum = 0.5
let rand_max_lum = 1.5
let space_saturation = 2.
let star_saturation = 20.
let dyn_color = ref true

(*Couleurs des boutons*)
let truecolor = rgb 0 128 0
let falsecolor = rgb 128 0 0
let slidercolor = rgb 128 128 128
let buttonframe = rgb 64 64 64
let buttonframewidth = int_of_float (10. *. !ratio_rendu)

(*Paramètres de flou de mouvement*)
(*Implémenté correctement pour les bullets et étoiles,
dessine des trainées derrière les autres types d'objets,
mais de manière erratique, donc désactivé par défaut*)
let motion_blur = ref false
let shutter_speed = 1.

(******************************************************************************)
(*Paramètres de jeu*)

(*Permet le contrôle du vaisseau à la souris.
Viser avec la souris, clic pour accélérer, toujours barre d'espace pour tirer*)
let mousecontrol = ref true
(*Les contrôles directs ne contrôlent pas la vitesse et le moment mais directement la position et la rotation.
Les valeurs par défaut sont celles demandées dans le tp*)
(*TODO implémenter correctement toutes les méthodes de contrôle*)
let ship_direct_pos = ref false
let ship_direct_rotat = ref false
let ship_impulse_pos = ref true
let ship_impulse_rotat = ref true

(*Ratio pour conversion des dégats physiques depuis le changement de vélocité au carré*)
let ratio_phys_deg = ref 0.002
let advanced_hitbox = ref true

(*Let objets physiques en contact se repoussent un peu plus que normal pour éviter d'être imbriqués*)
let min_repulsion = 5.
let min_bounce = 50.

(*Paramètres des astéroïdes*)
let asteroid_spawn_delay = 0.01 (*Temps s'écoulant entre l'apparition de deux astéroïdes*)
let asteroid_max_spawn_radius = 700. (*Taille max d'astéroïde au spawn.*)
let asteroid_min_spawn_radius = 400. (*Taille min de spawn*)
let asteroid_min_size = 50. (*En dessous de la taille minimale, un asteroide se transforme en chunk*)
let asteroid_max_moment = 2. (*Rotation max d'un astéroïde au spawn (dans un sens aléatoire)*)
let asteroid_max_velocity = 2000. (*Velocité max au spawn*)
let asteroid_min_velocity = 1500. (*Velocité min au spawn*)
let asteroid_stage_velocity = 500. (*Permet aux astéroïdes de stages plus avancés d'aller plus vite*)
let asteroid_density = 1. (*Sert à déterminer la masse d'un astéroïde en se basant sur sa surface*)
let asteroid_min_health = 200. (*Évite les astéroïdes trop fragiles à cause d'une masse trop faible. S'additionne au calcul.*)
let asteroid_mass_health = 0.01(*Sert à déterminer la vie d'un astéroïde basé sur sa masse*)
(*Dam : dommmages. phys : dommages physiques. Ratio : Multiplicateur du dégat. res : résistance aux dégats (soustraction)*)
let asteroid_dam_ratio = 1. (*La sensibilité aux dégats d'explosions*)
let asteroid_dam_res = 0. (*La résistance aux dégats d'explosions*)
let asteroid_phys_ratio = 1. (*Sensibilité aux chocs physiques*)
let asteroid_phys_res = 100. (*Résistance aux chocs physiques*)
(*Paramètres pour les couleurs d'astéroïdes à la naissance*)
let asteroid_min_lum = 20.
let asteroid_max_lum = 150.
let asteroid_min_satur = 0.3
let asteroid_max_satur = 0.5
(*Paramètres de la hitbox et des visuels polygonaux*)
let asteroid_polygon_min_sides = 7(*Nombre minimum de côtés qu'un astéroïde peut avoir*)
let asteroid_polygon_size_ratio = 0.02 (*Permet de déterminer le nombre de côtés qu'un astéroïde aura pour sa hitbox et son rendu. Permet de rendre les gros projectiles plus détaillés, et les petits moins consommateurs en perfs.*)
let asteroid_polygon_min = 1. (*En ratio du rayon*)
let asteroid_polygon_max = 1.3 (*En ratio du rayon*)
(*Contrôle du nombre d'astéroïde apparaissant à chaque vague*)
let asteroid_min_nb = 1
let asteroid_stage_nb = 2
(*Paramètres pour rapprocher l'air de rien les objets trop lointains (plus utilisé)*)
let half_close = 10. (*Demi-temps de rapprochement d'un objet par rapport au centre de l'écran*)
let accel_close = 0.00001 (*acceleration appliquée aux objets unspawned vers le centre de l'écran*)

(*Caractéristiques des fragments. Principalement hérité des parents.*)
let fragment_max_velocity = 2000. (*Velocité max au spawn*)
let fragment_min_velocity = 1000. (*Velocité min au spawn*)
let fragment_max_size = 0.6 (*En ratio de la taille de l'astéroïde parent*)
let fragment_min_size = 0.3 (*En ratio de la taille de l'astéroïde parent*)
let fragment_min_exposure = 0.6 (*Pour les variations relative de luminosité par rapport à l'astéroïde parent*)
let fragment_max_exposure = 1.4 (*On ne met pas 2, pour qu'en moyenne, les astéroïdes deviennent plus sombres en rétrécissant*)
let fragment_number = ref 2
let chunk_radius_decay = 2. (*Pour la décroissance des particules n'ayant pas de collisions*)

(*Paramètres du vaisseau*)
(*valeurs du vaisseau*)
let ship_max_health = 100. (*health au spawn. Permet de l'appliquer au modèle physique.*)
let ship_max_healths = 3 (*Nombre de fois que le vaisseau peut réapparaître*)
let ship_density = 50. (*Pour calcul de la masse du vaisseau, qui a un impact sur la physique*)
let ship_radius = 20. (*Pour la hitbox et le rendu*)
(*Réduction des dégats et dégats physiques*)
let ship_dam_ratio = 0.8
let ship_dam_res = 10.
let ship_phys_ratio = 0.01
let ship_phys_res = 5.
let ship_death_max_momentum = 2.
(*Contrôles de déplacement*)
let ship_max_depl = 50. (*En px.s⁻¹. Utile si contrôle direct du déplacement.*)
let ship_max_accel = 10000. (*En px.s⁻² Utile si contrôle de l'accélération*)
let ship_max_boost = 2000. (*En px.s⁻¹. Utile si contrôle par boost.*)
let ship_half_stop = 10. (*En temps nécessaire pour perdre la moitié de l'inertie*)
(*Contrôles de rotation*)
let ship_max_tourn = 4. (*En radian.s⁻¹*)
let ship_max_moment = 0.5 (*En radian.s⁻²*)
let ship_max_tourn_boost = 3.(*En radians.s⁻¹*)
let ship_max_rotat = pi /. 6.(*En radians*)
let ship_half_stop_rotat = 0.2(*En temps nécessaire pour perdre la moitié du moment angulaire*)
(*Temps min entre deux téléportations aléatoires*)
let cooldown_tp = 10.
(*Pour l'autoregen*)
let autoregen = true
let autoregen_health = 2. (*Regain de vie par seconde*)

(*Valeurs du projectile*)
let projectile_recoil = ref 100. (*Recul appliqué au vaisseau*)
let projectile_cooldown = ref 0.3 (*Temps minimum entre deux projectiles*)
let projectile_max_speed = ref 15000.(*Vitesse relative au lanceur lors du lancement*)
let projectile_min_speed = ref 10000.
let projectile_deviation = ref 0.3(*Déviation possible de la trajectoire des projectiles*)
let projectile_radius = ref 15.
let projectile_radius_hitbox = ref 50. (*On fait une hitbox plus grande pour être généreux sur les collisions*)
let projectile_health = 0.(*On considère la mort quand la santé descend sous zéro. On a ici la certitude que le projectile se détruira*)
let projectile_number = ref 50

let projectile_number_default = 10

(*Shotgun*)
let shotgun_recoil = 1000.
let shotgun_cooldown = 0.3
let shotgun_max_speed = 15000.
let shotgun_min_speed = 10000.
let shotgun_deviation = 0.3
let shotgun_radius = 15.
let shotgun_radius_hitbox = 50.
let shotgun_number = 50

(*Sniper*)
let sniper_recoil = 10000.
let sniper_cooldown = 1.
let sniper_max_speed = 20000.
let sniper_min_speed = 15000.
let sniper_deviation = 0.
let sniper_radius = 25.
let sniper_radius_hitbox = 75.
let sniper_number = 1

(*Machinegun*)
let machine_recoil = 10.
let machine_cooldown = 0.01
let machine_max_speed = 10000.
let machine_min_speed =  8000.
let machine_deviation = 0.2
let machine_radius = 10.
let machine_radius_hitbox = 25.
let machine_number = 1

(*Valeurs des explosions*)
let explosion_max_radius = 100.
let explosion_min_radius = 80.
let explosion_min_exposure = 1.(*Détermine la luminosité max et min des explosions au spawn*)
let explosion_max_exposure = 2.
let explosion_damages = 150.
(*Pour les explosions héritant d'un objet*)
let explosion_ratio_radius = 1.5
let explosion_saturate = 5.
let explosion_min_exposure_heritate = 6.(*Détermine la luminosité max et min des explosions héritant d'objets au spawn*)
let explosion_max_exposure_heritate = 8.

(*Valeurs des muzzleflashes*)
let muzzle_ratio_radius = 3.
let muzzle_ratio_speed = 0.05

(*Valeurs du feu à l'arrière du vaisseau*)
let fire_max_random = 20.
let fire_min_speed = 250.
let fire_max_speed = 500.
let fire_ratio_radius = 1.

(*Valeurs de la fumée*)
let smoke = ref true
let smoke_half_col = 0.2 (*Vitesse de la décroissance de la couleur*)
let smoke_half_radius = 1. (*Vitesse de la décroissance du rayon*)
let smoke_radius_decay = 10. (*Diminution du rayon des particules de fumée*)
let smoke_max_speed = 40.(*Vitesse random dans une direction random de la fumée*)

(*Valeurs des étincelles TODO*)

(*Valeurs des étoiles*)
let star_min_prox = 0.4 (*Prox min des étoiles. 0 = étoile à l'infini, paraît immobile quel que soit le mouvement.*)
let star_max_prox = 0.8 (*Prox max. 1 = même profondeur que le vaisseau *)
let star_prox_lum = 10.(*Pour ajouter de la luminosité aux étoiles plus proches*)
let star_min_lum = 0.1
let star_max_lum = 1.
let star_rand_lum = 1. (*Effet de scintillement des étoiles*)
let stars_nb_default = 500
let stars_nb = ref 200
let stars_nb_previous = ref 200


(*Effet de scanlines pour imiter les moniteurs crt qui projetait l'image ligne par ligne.*)
(*Activer l'effet animated_scanlines permet l'animation imitant les vidéos interlacées,
en activant une ligne sur deux une image sur deux, mais il passe mal
à cause du raffraichissement de l'image ne pouvant pas vraiment être
à 60 pile avec le moteur d'ocaml. Testez à vos risques et périls*)
let scanlines = ref false
let scanlines_period = 5
let animated_scanlines = true
let scanlines_offset = ref 0

(*La camera predictive oriente la camera vers l'endroit où le vaisseau va,
pour le garder tant que possible au centre de l'écran*)
let dynamic_camera = ref true
let camera_prediction = 1.5 (*En secondes de déplacement du vaisseau dans le futur.*)
let camera_half_depl = 1. (*Temps pour se déplacer de moitié vers l'objectif de la caméra*)
let camera_ratio_objects = 0. (*La caméra va vers la moyenne des positions des objets, pondérés par leur masse et leur distance au carré*)
let camera_ratio_vision = 0.1 (*La caméra va vers là où regarde le vaisseau, à une distance correspondant au ratio x la largeur du terrain*)

(*Le screenshake ajoute des effets de tremblements à l'intensité dépendant  des évènements*)
let screenshake = ref true
let screenshake_smooth = true (*Permet un screenshake moins agressif, plus lisse et réaliste physiquement. Sorte de passe-bas sur les mouvements*)
let screenshake_smoothness = 0.8 (*0 = aucun changement, 0.5 =  1 = lissage infini, screenshake supprimé.*)
let screenshake_tir_ratio = 200.
let screenshake_dam_ratio = 0.001
let screenshake_phys_ratio = 0.001
let screenshake_phys_mass = 20000.(*Masse de screenshake «normal». Des objets plus légers en provoqueront moins, les objets plus lourds plus*)
let screenshake_half_life = 0.1
let game_screenshake = ref 0.
let game_screenshake_pos = ref (0.,0.)
let game_screenshake_previous_pos = ref (0.,0.) (*Permet d'avoir un rendu correct des trainées de lumières lors du screenshake*)
(*Utilisation de l'augmentation du score pour faire trembler les chiffres*)
let shake_score = ref 0.
let shake_score_ratio = 0.5
let shake_strength = 0.05
let shake_score_half_life = 1.



(*L'antialiasing de jitter fait «trembler» l'espace de rendu.
C'est une forme de dithering spatial
afin de compenser la perte de précision due à la rastérisation
lors du placement des objets et du tracé des contours.*)
let dither_aa = true
(*La puissance du jitter détermine à quel point le rendu peut se décaler.*)
(*Déterminer à 1 ou moins pour éviter un effet de flou et de fatigue visuelle*)
let dither_power = 0.5 (*En ratio de la taille d'un pixel*)
let dither_power_radius = 0.5
(*Le jitter double courant permet de faire le même jitter sur les positions d'objets.
Cela permet de s'assurer une consistance spatiale dans tout le rendu.*)
let current_jitter_double = ref (0.,0.)

(*L'exposition variable permet des variations de luminosité en fonction des évènements*)
let variable_exposure = true
let exposure_ratio_damage = 0.99
let exposure_tir = 0.97
let exposure_half_life = 1.
let game_exposure_target_death = 0.5
let game_exposure_target_boucle = 2.
let game_exposure_target_tp = 0.5
let game_exposure_target = ref 2.
let game_exposure = ref 0.

(*Flashes lumineux lors d'évènements*)
let flashes = ref true
let flashes_damage = 0.
let flashes_explosion = 0.05
let flashes_saturate = 8.
let flashes_normal_mass = 100000.
let flashes_tir = 0.5
let flashes_teleport = 1000.
let flashes_half_life = 0.05
