## Syncing and offline mode
1. 2 modes: offline does not connect to internet and continually polls to switch to online mode. online mode syncs


## blueprints
Blueprints are bevy_replicon sync-able components that encode serializable information about
children / interactions.
Whenever the blueprint changes, all children are removed and re-spawned. Hence, using 
`BlockId`s instead of `Entity`s, since `BlockId` is consistently serializable.
Only the `NetworkedBlueprint`s are serialized and synced, all others can be
derived from these. 