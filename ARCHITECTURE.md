## bevy_replicon
This crate syncs components between server and client as fast as it can.
The components it syncs are:
- Position and Rotation (xpbd) of moving objects (player, asteroids, bullets)
- PlayerController
  - This contains the thruster input data

### Events
To facilitate spawning and despawning of things, a reliable transport of events is used:
- AsteroidBlueprint, PlayerBlueprint
	- Includes optimized/custom mesh
	- Automatically adds 'default' components, like SpatialBundle, AsyncCollider when recieved

Because AsteroidBlueprint and PlayerBlueprint need to spawn additional child components, they
can have fields of `Vec<BlockBlueprint< ... >>` and spawn the BlockBlueprint bundles.
Each bundle is generic over a specific type of block, for example 'natural blocks' or
'thruster blocks' or 'weapon blocks'. Spawning a BlockBlueprint (generically) adds the 
value of the generic type as a marker component, so the module which exports the marker component
can handle spawning and custom logic