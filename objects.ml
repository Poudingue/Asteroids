(*Pour le type et la création d'objets*)
open Graphics
open Parameters
open Functions
open Colors
(******************************************************************************)

(*On pourrait ajouter des types différents, par exemple des missiles à tête chercheuse, des vaisseaux ennemis…*)
(*TODO plus tard si le temps*)
type type_object = Asteroid | Projectile | Ship | Explosion | Smoke | Spark | Shotgun | Sniper | Machinegun

(*Polygone pour le rendu et les collisions. Liste de points en coordonées polaires autour du centre de l'objet.*)
type polygon = (float*float) list

(*Pour les calculs de collision*)
type hitbox = {
  mutable ext_radius : float;
  mutable int_radius : float;
  mutable points : polygon; (*Liste des points pertinents pour calculer la collision. Angle * distance *)
}


(*Pour les calculs de visuels*)
type visuals = {
  mutable color : hdr;
  mutable radius : float;
  mutable shapes : (hdr*polygon) list;
}

type objet_physique = {
  objet : type_object;
  hitbox : hitbox;
  visuals : visuals;
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

  mutable proper_time : float;

  mutable hdr_exposure : float;
}

(*Pour l'arrière-plan étoilé*)
type star = {
  mutable last_pos : (float*float);(*La position précédente permet de calculer correctement le motion_blur*)
  mutable pos : (float*float); (*Si on l'appelle pos, toutes les fonctions appelant objet_physique.position ralent comme quoi star n'est pas un objet physique.*)
  proximity : float;(*Proximité avec l'espace de jeu.
  À 1, se situe sur le même plan que le vaisseau, à 0, à une distance infinie.
  Correspond simplement au ratio de déplacement lors du mouvement caméra*)
  lum : float;
}





(*Aspect visuel du vaisseau*)
let visuals_ship = {
  color = {r=1000.;v=100.;b=25.};
  radius = ship_radius *. 0.9;
  shapes =
    [({r=200.;v=20.;b=20.},
      [(0.,3.*.ship_radius);
      (3. *. pi /. 4.,2.*.ship_radius);
      (pi,ship_radius);
      (~-.3. *. pi /. 4.,2.*.ship_radius)]);

    ({r=250.;v=25.;b=25.},
      [(0.,3.*.ship_radius);
      (pi,ship_radius);
      (~-.3. *. pi /. 4.,2.*.ship_radius)]);

    ({r=120.;v=5.;b=5.},
      [(0.,3.*.ship_radius);
      (3. *. pi /. 4.,2.*.ship_radius);
      (pi,ship_radius)]);

    ({r=10.;v=10.;b=10.},
      [(pi,ship_radius/.3.);
      (pi,ship_radius);
      (~-.3. *. pi /. 4.,2.*.ship_radius)]);

    ({r=30.;v=30.;b=30.},
      [(pi,ship_radius/.3.);
      (3. *. pi /. 4.,2.*.ship_radius);
      (pi,ship_radius)]);

    ({r=200.;v=180.;b=160.},
      [(0.,3.*.ship_radius);
      (0.,1.5*.ship_radius);
      (~-.pi /. 8.,1.5*.ship_radius)]);

    ({r=20.;v=30.;b=40.},
      [(0.,3.*.ship_radius);
      (pi /. 8.,1.5*.ship_radius);
      (0.,1.5*.ship_radius)])
    ];
}

let hitbox_ship = {
  ext_radius = 3. *. ship_radius;
  int_radius = ship_radius;
  points = [(0.,3.*.ship_radius);
  (3. *. pi /. 4.,2.*.ship_radius);
  (pi,ship_radius);
  (~-.3. *. pi /. 4.,2.*.ship_radius)];
}

(*Création du vaisseau*)
let spawn_ship () = {
    objet = Ship;
    visuals = visuals_ship;
    hitbox = hitbox_ship;
    mass = pi *. (carre ship_radius) *. ship_density;
    health = ship_max_health;
    max_health = ship_max_health;

    dam_ratio = ship_dam_ratio;
    dam_res = ship_dam_res;
    phys_ratio = ship_phys_ratio;
    phys_res = ship_phys_res;

    last_position = (!phys_width /. 2., !phys_height /. 2.);
    position = (!phys_width /. 2., !phys_height /. 2.);
    velocity = (0.,0.);
    half_stop = ship_half_stop;

    orientation = pi /. 2.;
    moment = 0.;
    half_stop_rotat = ship_half_stop_rotat;
    (*C'est ici que l'on détermine la forme du vaisseau. *)

    proper_time = 1.;
    hdr_exposure = 1.;
}


let spawn_projectile position velocity = {
    objet = Projectile;

    visuals = {
      color = {r=2000.;v=400.;b=200.};
      radius = !projectile_radius;
      shapes = [];
    };

    hitbox = {
      int_radius = !projectile_radius_hitbox;
      ext_radius = !projectile_radius_hitbox;
      points = [];
    };

    mass = 10000.;
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
    hdr_exposure = 4.;
}

(*Permet de créer n projectiles*)
let rec spawn_n_projectiles ship n =
  if n = 0 then [] else (
  let vel = if projectile_herit_speed
    then addtuple ship.velocity (polar_to_affine (((Random.float 1.) -. 0.5) *. !projectile_deviation +. ship.orientation) (!projectile_min_speed +. Random.float (!projectile_max_speed -. !projectile_min_speed)))
    else (polar_to_affine (((Random.float 1.) -. 0.5) *. !projectile_deviation +. ship.orientation) (!projectile_min_speed +. Random.float (!projectile_max_speed -. !projectile_min_speed)))
  and pos = addtuple ship.position (polar_to_affine ship.orientation ship.hitbox.ext_radius) in (*On fait spawner les projectiles au bout du vaisseau*)
  ref (spawn_projectile pos vel) :: spawn_n_projectiles ship (n-1))



  let spawn_chunk_explo position velocity color= {
      objet = Asteroid;

      visuals = {
        color = color;
        radius = chunks_explo_min_radius +. Random.float (chunks_explo_max_radius -. chunks_explo_min_radius);
        shapes = [];
      };

      hitbox = {
        int_radius = 0.;
        ext_radius = 0.;
        points = [];
      };

      mass = 100.;
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
      hdr_exposure = 4.;
  }

  (*Permet de créer n projectiles*)
  let rec spawn_n_chunks ship n color =
    if n = 0 then [] else (
    let vel = addtuple ship.velocity (polar_to_affine (Random.float (2. *. pi)) (chunks_explo_min_speed +. Random.float (chunks_explo_max_speed -. chunks_explo_min_speed)))
      and pos = ship.position in
    ref (spawn_chunk_explo pos vel color) :: spawn_n_chunks ship (n-1) color)


(*Spawne une explosion d'impact de projectile*)
let spawn_explosion ref_projectile =
  let rad = explosion_min_radius +. (Random.float (explosion_max_radius -. explosion_min_radius)) in
  let rand_lum = (randfloat explosion_min_exposure explosion_max_exposure) in
  ref {
  objet = Explosion;
  visuals = {
    color = intensify {
      r = 2000.;
      v = 500. ;
      b = 200.}
    rand_lum;
    radius = rad;
    shapes = [];
  };
  hitbox = {
    int_radius = rad;
    ext_radius = rad;
    points = [];
  };
  mass = explosion_damages_projectile;
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

  proper_time = 1.;
  hdr_exposure = 1.;
}


(*Spawne une explosion héritant d'un objet d'une taille au choix.*)
let spawn_explosion_object ref_objet =
  let rad = explosion_ratio_radius *. !ref_objet.hitbox.int_radius in (*On récupère le rayon de l'objet*)
  if !flashes then add_color := hdr_add !add_color (intensify (saturate !ref_objet.visuals.color flashes_saturate) (!ref_objet.mass *. flashes_explosion *. (randfloat explosion_min_exposure_heritate explosion_max_exposure_heritate) /. flashes_normal_mass));
  if variable_exposure then game_exposure := !game_exposure *. exposure_ratio_explosions;
  ref {
  objet = Explosion;
  visuals = {
    color = intensify (saturate !ref_objet.visuals.color explosion_saturate) (randfloat explosion_min_exposure_heritate explosion_max_exposure_heritate);
    radius = rad;
    shapes = [];
  };
  hitbox = {
    int_radius = rad;
    ext_radius = rad;
    points = [];
  };
  mass = explosion_damages_objet;
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

  proper_time = 1.;
(*La nouvelle exposition est partagée entre couleur et exposition, pour que la fumée ne finisse pas trop sombre*)

  hdr_exposure = randfloat explosion_min_exposure_heritate explosion_max_exposure_heritate ;
}

(*Spawne une explosion héritant du vaisseau lors de sa mort*)
let spawn_explosion_death ref_objet time =
  let rad = explosion_death_ratio_radius *. !ref_objet.hitbox.int_radius in (*On récupère le rayon de l'objet*)
  if !flashes then add_color := hdr_add !add_color (intensify (saturate !ref_objet.visuals.color flashes_saturate) (!ref_objet.mass *. flashes_explosion *. (randfloat explosion_min_exposure_heritate explosion_max_exposure_heritate) /. flashes_normal_mass));
  if variable_exposure then game_exposure := !game_exposure *. exposure_ratio_explosions;
  ref {
  objet = Explosion;
  visuals = {
    color = intensify (saturate !ref_objet.visuals.color explosion_saturate) (randfloat explosion_min_exposure_heritate explosion_max_exposure_heritate);
    radius = rad;
    shapes = [];
  };
  hitbox = {
    int_radius = rad;
    ext_radius = rad;
    points = [];
  };
  mass = time *. explosion_damages_death;
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

  proper_time = 1.;
(*La nouvelle exposition est partagée entre couleur et exposition, pour que la fumée ne finisse pas trop sombre*)

  hdr_exposure = randfloat explosion_min_exposure_heritate explosion_max_exposure_heritate ;
}


let spawn_explosion_chunk ref_objet =
  let rad = explosion_ratio_radius *. !ref_objet.visuals.radius in (*On récupère le rayon de l'objet*)
  if !flashes then add_color := hdr_add !add_color (intensify (saturate !ref_objet.visuals.color flashes_saturate) (!ref_objet.mass *. flashes_explosion *. (randfloat explosion_min_exposure_heritate explosion_max_exposure_heritate) /. flashes_normal_mass));
  if variable_exposure then game_exposure := !game_exposure *. exposure_ratio_explosions;
  ref {
  objet = Explosion;
  visuals = {
    color = !ref_objet.visuals.color;
    radius = rad;
    shapes = [];
  };
  hitbox = {
    int_radius = rad;
    ext_radius = rad;
    points = [];
  };
  mass = explosion_damages_chunk (*Replace with a function of time spent on frame*);
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

  proper_time = 1.;
(*La nouvelle exposition est partagée entre couleur et exposition, pour que la fumée ne finisse pas trop sombre*)

  hdr_exposure = explosion_min_exposure +. (Random.float (explosion_max_exposure -. explosion_min_exposure));
}

(*Spawne un muzzleflash à la position donnée*)
let spawn_muzzle ref_projectile = ref {
  objet = Smoke;
  visuals = {
    color = intensify !ref_projectile.visuals.color (randfloat explosion_min_exposure_heritate explosion_max_exposure_heritate);
    radius = muzzle_ratio_radius *. !ref_projectile.visuals.radius;
    shapes = [];
  };
  hitbox = {
    int_radius = 0.;
    ext_radius = 0.;
    points = [];
  };
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
  hdr_exposure = explosion_min_exposure +. (Random.float (explosion_max_exposure -. explosion_min_exposure));
}


(*Spawne du feu à l'arrière d'un vaisseau accélérant*)
let spawn_fire ref_ship = ref {
  objet = Smoke;
  visuals = {
    color = {r = 1500. ; v = 400. ; b = 200. };
    radius = fire_ratio_radius *. !ref_ship.hitbox.int_radius;
    shapes = [];
  };
  hitbox = {
    int_radius = 0.;
    ext_radius = 0.;
    points = [];
  };
  mass = 0.;
  health = 0.;
  max_health = 0.;

  dam_res = 0.;
  dam_ratio = 0.;
  phys_res = 0.;
  phys_ratio = 0.;

  last_position = !ref_ship.position;
  position = addtuple !ref_ship.position (polar_to_affine (!ref_ship.orientation +. pi) !ref_ship.hitbox.int_radius);
  velocity = addtuple !ref_ship.velocity (addtuple (polar_to_affine (!ref_ship.orientation +. pi) (fire_min_speed +. (Random.float (fire_max_speed -. fire_min_speed)))) (polar_to_affine (Random.float 2. *. pi) (Random.float fire_max_random)));
  half_stop = 0.;
  orientation = 0.;
  moment = 0.;
  half_stop_rotat = 0.;

  proper_time = 1.;
  hdr_exposure = explosion_min_exposure +. (Random.float (explosion_max_exposure -. explosion_min_exposure));
}



let rec polygon_asteroid radius n =
  let nb_sides = max asteroid_polygon_min_sides (int_of_float (asteroid_polygon_size_ratio *. radius)) in
  if n = 1
    then ([(2. *. pi *. (float_of_int n) /. (float_of_int nb_sides)), radius *. (randfloat asteroid_polygon_min asteroid_polygon_max)])
    else ((2. *. pi *. (float_of_int n) /. (float_of_int nb_sides)), radius *. (randfloat asteroid_polygon_min asteroid_polygon_max)) :: polygon_asteroid radius (n-1);;


let spawn_asteroid (x, y) (dx, dy) radius =
  let shape = polygon_asteroid radius (max asteroid_polygon_min_sides (int_of_float (asteroid_polygon_size_ratio *. radius)))
  and color = saturate {
    r = randfloat asteroid_min_lum asteroid_max_lum ;
    v = randfloat asteroid_min_lum asteroid_max_lum ;
    b = randfloat asteroid_min_lum asteroid_max_lum}
      (randfloat asteroid_min_satur asteroid_max_satur);
  in
{
  objet = Asteroid;
  visuals = {
    color = color;
    radius = radius;
    shapes =  [(color,shape)];
  };
  hitbox = {
    int_radius = radius;
    ext_radius = radius *. asteroid_polygon_max;
    points = shape;
  };
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
  hdr_exposure = 1.;
}


(*Permet de donner des coordonées telles que l'objet n'apparaisse pas dans l'écran de jeu.*)
let rec random_out_of_screen radius =
  let (x,y) = ((Random.float ( 3. *. !phys_width)) -. !phys_width, (Random.float ( 3. *. !phys_height)) -. !phys_height) in
  if (y +. radius > 0. && y -. radius < !phys_height && x +. radius > 0. && x -. radius < !phys_width) then  random_out_of_screen radius else (x,y)
