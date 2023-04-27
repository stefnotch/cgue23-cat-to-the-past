# Cat to the past

Computergraphics Project at the TU Vienna

## Game concept

Rewinding time of the world around you, _but it doesn't affect your character._

So you can totally walk up to a table, and pick up the fancy sword.

Then rewind to the pastwhere the sword was still on the table. And since that didn't affect your character, you now have a sword on the table, _and a sword in your hand!_

Should make for an interesting puzzle game. Or a stealth:tm: game, because you can always openly smack the guy in front of you, get to the next area...and then just load the past where the guard was still alive and didn't alert the entire facility. Or that mechanic could be used to make a cat petting simulator, where you basically have a cheatcode. You can indefinitely pet the cat, because as soon as the cat is satisfied and walks away, you just turn time back... :cat2:

## Controls

- WASD+Mouse for moving
- T for swiTching to freecam
- Right mouse button for time rewinding
- Left mouse button for interacting
- Esc to quit

## Technical Details

world space: +y up, -z forward, +x right (reasonable right-handed coordinate system)  
winding order: counter-clockwise
units: meter  
importer: gltf, we flatten the tree, we generate one axis aligned collider per model

## Compiling and Running

You'll need a [Rust toolchain installed](https://www.rust-lang.org/tools/install). After that, you can start the game with

```
cargo run
```

You can also [build](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html) the project with

```
cargo build
```

## Demos

```
cargo run --bin bloom_demo
```

where bloom_demo can be replaced with the name of any demo project in the `demos/src/bin` folder

## More concrete ideas

3D Platformer game:

- pull objects down, climb onto it and use the rewind mechanic to get the object back up

Balancing:

- falling down kills you
- getting shot kills you
- player can't move while rewinding but still gets affected by the environment (bullet traveling backwards kills you)

## Used sources

- https://github.com/vulkano-rs/vulkano/blob/master/vulkano-win/src/winit.rs#L17
- https://github.com/vulkano-rs/vulkano/tree/master/examples/src/bin/interactive_fractal and co
- https://learnopengl.com/Guest-Articles/2022/Phys.-Based-Bloom
- https://github.com/Shot511/RapidGL/blob/65d1202a5926acad9816483b141fb24480e81668/src/demos/26_bloom/bloom.cpp
- https://learnopengl.com/PBR/Lighting
- https://www.youtube.com/watch?v=RRE-F57fbXw
- https://johannesugb.github.io/gpu-programming/setting-up-a-proper-vulkan-projection-matrix/#perspective-projection-into-clip-space
- https://github.com/FlaxEngine/FlaxSamples/blob/master/FirstPersonShooterTemplate/Source/FirstPersonShooter/PlayerScript.cs
