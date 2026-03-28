# Asteroids — Game Manual

## Overview

You pilot a lone spacecraft through an asteroid field. Rocks come from every direction, spin, and shatter when hit. Your goal is simple: survive as long as possible and rack up score by destroying asteroids.

The game has no end state except death. Each destroyed asteroid advances your progress, and the waves get harder as you go.

---

## Controls

| Key / Input | Action |
|---|---|
| **Mouse** (aim) | The ship always faces the cursor |
| **Left Mouse Button** | Continuous thrust forward |
| **W** | Boost forward (single impulse per press) |
| **A** | Rotate left |
| **D** | Rotate right |
| **Q** | Strafe left (perpendicular boost) |
| **E** | Strafe right (perpendicular boost) |
| **Space** | Fire (hold to keep shooting) |
| **F** | Teleport to mouse cursor position |
| **P** / **Escape** | Pause / unpause |
| **R** | Restart |
| **K** | Quit immediately |
| **F11** / **Alt+Enter** | Toggle fullscreen |

**Note:** Controls use physical key positions (QWERTY layout assumed). On AZERTY keyboards: Z = W, Q = A.

---

## Ship

Your ship starts at the center of the screen with **100 HP** and **3 lives**.

### Movement

Movement is physics-based — the ship has real inertia and keeps drifting when you stop thrusting. There is no automatic brake.

- **Left click / W** thrusts in the direction you're facing. W fires a single impulse per keypress; left click thrusts continuously while held.
- **Q / E** strafe perpendicular to your heading — useful for dodging while keeping your aim.
- The ship coasts indefinitely but does lose velocity slowly over time (half-inertia decay in about 10 seconds).
- Rotation also has inertia; tapping A/D kicks angular momentum that continues until it decays.

### Health and Lives

- Max health: **100 HP**. Health regenerates automatically at **5 HP/s** while alive.
- Damage sources: asteroid collisions and explosion blast radius.
- The ship has some damage resistance — small physical impacts are reduced (physical damage ratio 0.5%), and explosion damage is reduced by a flat 10 points.
- On death, the game slows to **80% speed**, you lose one life, and the ship burns for 1–5 seconds before respawning.
- When all **3 lives** are exhausted, the game resets.

---

## Weapons

### Default (Shotgun)

At game start you fire in shotgun mode: **50 pellets per shot**, spread in a ~17° cone, at 0.3s cooldown. Each pellet travels at 10,000–15,000 units/s and inherits your ship's velocity. Recoil pushes you backward slightly.

### Sniper

Single precision round. 1s cooldown, no spread, high speed (15,000–20,000 u/s), heavy recoil. One shot, one kill on most asteroids.

### Shotgun / Machine Gun

The machine gun fires single fast rounds at very high rate (0.01s cooldown, ~100 rounds/s) with moderate spread. Low recoil.

> **Weapon switching** is planned but not yet implemented. The game launches in shotgun mode.

### Projectile Behavior

- Projectiles inherit your ship's velocity on top of their own speed — firing forward while moving fast sends them significantly faster.
- Projectiles that hit asteroids detonate as small explosions, dealing area damage to nearby objects.
- Projectiles despawn when they leave the visible area or health drops to zero.

---

## Asteroids

### Sizes

Asteroids spawn in at **350–650 units** radius. Anything smaller than 100 units is considered a fragment too small to fragment further — it just gets destroyed.

Asteroid shapes are randomly generated polygons with 7+ sides and slightly irregular edges.

### Health

Health is mass-based: `mass × 0.01 + 50`. Larger asteroids are much tougher. A full-size asteroid has significantly more HP than a small fragment.

### Fragmentation

When an asteroid is destroyed, it **spawns 5 fragments** per dead asteroid. Each fragment is **40–70% the size** of its parent and inherits the parent's color but gains random scatter velocity (1,500–2,500 u/s on top of parent velocity). Fragments that are too small (under 100 units) skip the fragmentation step and simply disappear.

Fragments can themselves be destroyed, producing more fragments — chain reactions are possible.

### Damage Resistance

Asteroids have **100 points of physical collision resistance** (collisions below that threshold deal no damage) but no resistance to explosion damage.

### Rotation

Each asteroid spawns with a random spin (up to ±1 rad/s) and maintains it throughout its lifetime.

---

## Stages

Stages track how many asteroids have been spawned in the current wave. A new asteroid enters the field every **2 seconds**. Once the required count for the current stage is reached, the stage advances.

| Stage | Asteroids required | Max spawn speed |
|---|---|---|
| 0 | 2 | 2,000 u/s |
| 1 | 3 | 2,500 u/s |
| 2 | 4 | 3,000 u/s |
| N | 2 + N | 2,000 + 500×N u/s |

The formula: each stage requires `2 + stage_number` asteroids to spawn before advancing, and the max asteroid speed increases by **500 u/s per stage**.

Stage number is shown in the HUD. On each new stage, the background color scheme changes.

---

## Scoring

Score increases by **1 point per asteroid destroyed**. Only full asteroids and fragments count — chunks and debris do not.

The score display on the HUD shakes briefly each time it increases.

---

## Visual Effects

All effects are toggleable from the pause menu.

| Option | What it does |
|---|---|
| **Screenshake** | Camera jolts on impacts, explosions, and death. Intensity scales with mass of colliding objects. |
| **Smoke particles** | Engine exhaust trail when thrusting. Muzzle smoke on firing. Disable for better performance. |
| **Chunk particles** | Debris chunks fly out from explosions and destroyed asteroids. Disable for better performance. |
| **Light Flashes** | Bright color flashes on firing, explosions, and teleport. |
| **Color Effects** | Dynamic background color shifts per stage, brightness variation over time. |
| **Retro visuals** | White vectors on black — classic arcade look. |
| **Scanlines** | CRT monitor scanline overlay. Reduces brightness slightly. |
| **Advanced hitbox** | More precise polygon collision detection (default: on). |

### Time Dilation

The game speed briefly slows down on dramatic events:
- Destroying an asteroid: game slows to **95%** then recovers
- Your death: game slows to **80%** speed
- Teleporting: game momentarily **freezes** (speed → 0)

Speed recovers toward normal with a half-life of ~0.1 seconds.

---

## Tips

- **Momentum is your friend and enemy.** Get in the habit of strafing to dodge rather than stopping — you can rarely fully brake in time.
- **Don't fight inertia, use it.** Fire a boost toward an asteroid, then swing your aim to another target while you drift.
- **Aim slightly ahead of moving asteroids.** Projectiles travel fast but not instantly; fast asteroids at close range can dodge a slow reaction.
- **Fragmentation cascades hurt.** Destroying a large asteroid creates 5 fragments flying in random directions — they can be more dangerous than the original. Clear space before triggering a chain.
- **The teleport (F) is a lifesaver** — it resets your velocity to zero and puts you exactly on cursor. It has a **5-second cooldown** and causes a brief moment freeze, so use it for emergency repositioning, not rapid movement. Aim far from asteroids before pressing.
- **Health regenerates** at 5 HP/s. If you can avoid damage for a few seconds, you'll recover. Don't panic-spend all your shots — take cover behind asteroids if needed.
- **Later stages spawn faster, bigger asteroids.** Prioritize shooting small fragments before they scatter off-screen — they'll wrap around and come back.
- **Screenshake is calibrated for feel.** If it's disorienting at high intensity, you can turn it off in the pause menu without affecting gameplay.
