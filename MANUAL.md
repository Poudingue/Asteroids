# ASTEROIDS — Game Manual

---

## The Field

You are alone out there. Rocks the size of cities drift in from every direction, spin, and shatter when hit. Shoot the big ones and they break into smaller ones. Shoot those too. The field never runs out — it just gets faster.

Your goal is to survive and rack up score. There is no finish line, only the next stage.

---

## Controls

### Keyboard + Mouse

| Input | Action |
|---|---|
| **W / A / S / D** | Move up / left / down / right (world space) |
| **Mouse** | Aim — the ship nose follows your cursor |
| **Left Mouse Button** | Fire (hold to keep shooting) |
| **F** | Teleport |
| **Escape** | Pause / unpause |
| **R** | Restart |
| **K** | Quit |
| **F11** / **Alt+Enter** | Toggle fullscreen |

### Gamepad

| Input | Action |
|---|---|
| **Left Stick** | Move |
| **Right Stick** | Aim |
| **A Button / Right Trigger** | Fire |
| **B Button / Left Trigger** | Teleport |
| **Start** | Pause |

---

## How It Plays

Movement and aiming are fully decoupled. You can fly left while shooting right, strafe past a rock while tracking it with your crosshairs. It's twin-stick — your direction of travel and your aim direction are independent at all times.

The ship's nose smoothly rotates to face wherever you're aiming. This is cosmetic — firing is always instant, no waiting for the rotation to catch up.

Diagonal movement is normalized. Holding W+D gets you the same speed as holding W alone — no diagonal speed exploit.

---

## Teleport

Teleport is your big move. Aim at an asteroid and press F (or B / Left Trigger). If a large enough asteroid is within a narrow cone around your crosshairs, you warp directly into it — destroying it from the inside in a blue explosion.

If nothing is in the cone, nothing happens. Aim counts.

Teleport has a cooldown. Use it for high-value targets or last-ditch escapes, not as a movement trick.

---

## Asteroids

Asteroids spawn at the edges and drift in. They spin, they're tough, and they don't stop.

Destroy a large one and it fragments into smaller pieces that scatter in all directions. Those fragments can be destroyed too, cascading into smaller chunks. A single well-placed shot into a cluster can trigger a chain reaction — which looks spectacular and can also kill you.

Smaller fragments are faster and harder to track. Don't assume a field is clear just because the big rocks are gone.

---

## Stages

The game tracks stages. As more asteroids are destroyed and spawned, the stage advances. Each stage brings faster asteroids and more of them. The HUD shows your current stage.

There is no ceiling — the field keeps accelerating until you don't.

---

## Scoring

One asteroid destroyed = one point. Fragments count. The score display in the HUD pulses every time it ticks up.

---

## Visual Effects

The game renders in HDR with tonemapping. Explosions bloom, engine fire trails behind you, debris arcs off shattered asteroids. Screen shake hits on impacts. Blue flash on teleport.

The background color scheme shifts per stage. It's subtle until it isn't.

---

## Tips

- **Keep moving.** A stationary ship is an easy target. Drift across the field and reposition constantly.
- **Strafe past threats.** Move perpendicular to an incoming rock while keeping your aim on it.
- **Teleport on the biggest rock in sight.** It destroys it instantly and resets your position — high value, high drama.
- **Aim your teleport carefully.** The targeting cone is narrow. If the crosshairs aren't on the asteroid, nothing happens.
- **Chain reactions are double-edged.** Shooting a large asteroid in a tight cluster sends five fragments outward at speed. Clear the area first, or make sure you have room to move.
- **Later stages accelerate hard.** What felt manageable at stage 3 is chaos by stage 6. Prioritize the fast small fragments before they wrap around and come back from behind.
