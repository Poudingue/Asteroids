open Graphics
open Parameters
open Functions
open Colors
open Objects
(******************************************************************************)


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

  let button_color={
    pos1 = ((10./.16.) *. !phys_width,(3./.24.) *. !phys_height);
    pos2 = ((12./.16.) *. !phys_width,(5./.24.) *. !phys_height);
    text = "Color Effects";
    text_over = "Color changes and correction";
    boolean = dyn_color;
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
