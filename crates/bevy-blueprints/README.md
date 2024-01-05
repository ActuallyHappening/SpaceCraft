## `bevy_blueprints`: A framework for serializing / deserializing visual components of `bevy` entities
- The blueprint marker component expands into all necessary components
- Sync necessary components like `Transform` in realtime, using external crate/s (for example `bevy_replicon`)
- Only mutates entities with the `BlueprintNeedsUpdating` component