# ADR 0024: Server-delivered mods

- Status: accepted (2026-11-05); built in the client, server side open; `item_model` naming a registry item works live, definition ids that exist only in a pack (`items/<namespace>/<name>.json`) work live too; an id nothing defines draws the stack's own item where Java draws the missing model

## Context

[ADR 0006](0006-mod-platform.md) plans mods that servers deliver, and [ADR 0019](0019-photon-server.md) found that an unmodified Pumpkin lets a plugin send and read custom payloads only in the play phase, never in configuration. A server must be able to change items, the HUD, animations, sounds, pictures and physics, the client must fetch what it lacks once and read the cache afterwards, and Photon servers must be able to refuse players who are not on Photon.

## Decision

- **What the vanilla path carries.** Pictures, sounds, models, translations and item definitions travel as a server resource pack, whose push carries a hash; the client already keeps packs by hash and does not download one it has ([#51](https://github.com/PhotonMC/photon-client/issues/51)). Items with new looks and behaviour come from item components (`item_model`, `equippable`, `custom_model_data`), and physics from entity attributes (gravity, jump strength, step height, scale, speed) that the client already predicts with. Mods do not duplicate these; a server on a vanilla or Pumpkin base gets them with no Photon code.
- **What mods add.** Logic and screens vanilla cannot express: a HUD, sounds chosen by game state, custom client rules. A mod is a WebAssembly component run by `photon-mods` with the budgets of ADR 0006.
- **Play-phase channels.** The server sends `photon:mods` after the player joins: `{"v":1,"mods":[{"id","hash","size","url","caps"}]}`. `hash` is the SHA-256 of the file in lower-case hex, `size` is at most 8 MiB, `url` is `https` (or this machine, for tests), `caps` are the capabilities the mod asks for. The client answers on `photon:hello` with `{"v":1,"client","loaded":[ids]}`, once when it joins and again after each manifest has been processed. A manifest with any wrong entry is refused whole.
- **Cache.** Files are kept under `<game dir>/cache/mods/<hash>.wasm`. A file is read from the cache without any request when its hash is there and still matches its name (a changed file is a miss and is deleted); an update is a new hash, so a mod the server did not change is never fetched again. At most 128 files are kept, the oldest go first. Every fetched byte is checked against hash and size before it is kept, redirects are not followed.
- **Capabilities.** The client grants `events.tick`, `hud.text` and `sound.play` itself, since they touch only the game's own screen and sound. A mod that asks for anything else (files, network, player data) is not loaded until the client can ask the player; there is no such prompt yet. A mod cannot call an import it did not ask for, and its effects per tick are capped.
- **Unloading.** A new manifest unloads mods it drops or whose hash changed. A mod that traps or runs past its allowance is unloaded and the others go on. The mods end with the connection.
- **Photon-only servers.** `photon:hello` is the gate: a Photon-only server kicks a player whose hello does not arrive in a few seconds. It keeps vanilla clients out; it is not a security boundary, since anyone can send the payload.
- **Physics.** Stays server-authoritative. The client mirrors it through attributes and the movement it already predicts. A mod does not move the player itself, or the server's corrections would undo it.

## Consequences

- A server delivers mods after the join, so the first seconds of a join run without them.
- The server side (sending the manifest, the hello check) is a `photon-server` plugin; it needs a wasm toolchain that is not set up here and is not built in this repository.
- Signing is still open (ADR 0006). Until it exists, a manifest is trusted exactly as far as the connection to the server is, and the hash only guards the file against the host that serves it.
