# Cat to the past
Computergraphics Project at the TU Vienna

## Game concept

Restoring any point in the past, *but it doesn't affect your character.*

So you can totally walk up to a table, and pick up the fancy sword.

Then load the past (save) where the sword was still on the table. And since that didn't affect your character, you now have a sword on the table, *and a sword in your hand!*

Should make for an interesting puzzle game. Or a stealth:tm: game, because you can always openly smack the guy in front of you, get to the next area...and then just load the past where the guard was still alive and didn't alert the entire facility. Or that mechanic could be used to make a cat petting simulator, where you basically have a cheatcode. You can indefinitely pet the cat, because as soon as the cat is satisfied and walks away, you just turn time back... :cat2:

## Technical Details

world space: +y up, -z forward, +x right (reasonable right-handed coordinate system)  
winding order: counter-clockwise
units: meter  
importer: gltf, we flatten the tree, we generate one axis aligned collider per model

## More concrete ideas

3D Platformer game: 
- pull objects down, climb onto it and use the rewind mechanic to get the object back up

Balancing:
- falling down kills you
- getting shot kills you
- player can't move while rewinding but still gets affected by the environment (bullet traveling backwards kills you)
