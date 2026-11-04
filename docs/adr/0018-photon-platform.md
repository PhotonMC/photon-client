# ADR 0018: Photon platform

- Status: accepted (2026-11-04); supersedes [ADR 0010](0010-one-product.md) and [ADR 0014](0014-multiplayer-only-first-release.md)

## Context

The client now matches the Java client closely in gameplay, protocol, rendering and sound. The remaining parity gaps are small and no longer decide the project's direction. The project owner wants Photon to become a platform, not a drop-in Java client: players find servers on a website, join them through a link, and use a menu, social and customization experience of Photon's own design, in the style of platforms such as Roblox (the experience, never their names, logos or art).

## Decision

- **Parity stays below the UI.** Packets, world state, movement, entity and terrain drawing, sound selection and the screens a server can open (containers, books, signs, dialogs, the death screen, chat, command blocks) keep matching the Java client, and their parity cases stay.
- **The menu flow is Photon's own.** The title screen, server list, server form, connect and disconnected screens, pause menu and the options screens are replaced by Photon's own UI. The vanilla versions of these screens are deleted, together with their parity cases. The stored options model (`options.txt` and its order) stays, because servers and gameplay read it.
- **The HUD stays vanilla** for now. Servers will later be able to change it.
- **Multiplayer only, no singleplayer.** There is no Singleplayer, Open to LAN or Realms entry, not even an inactive one.
- **Servers come from the Photon directory.** The home screen lists servers from Photon's backend. A separate screen still allows adding a server by address, including servers that do not run Photon's server.
- **Mods come from servers only.** A player cannot install mods. A server delivers its mods as described in [ADR 0006](0006-mod-platform.md).
- **Platform features.** These are part of the product: website integration through `photon://` links ([ADR 0020](0020-accounts-and-launch-links.md)), friends, presence, invites, platform chat beside server chat, customization and a cosmetics marketplace. Their backend is a separate service.
- **Servers.** Photon's server work lives in a separate repository, `photon-server` ([ADR 0019](0019-photon-server.md)). This repository still contains no server.
- **Design.** The client and the website share one set of design tokens. The client uses the "Classic" skin by default: dark, square and bevelled, close to the game. The website uses the light and dark skins ([ADR 0021](0021-client-ui-technology.md)).

## Consequences

- AGENTS.md's goal section describes the platform: parity covers what lies below the UI, and the menu flow is designed by us.
- The parity families for deleted screens are removed in the same commits that delete the screens. The README progress metrics are regenerated with them.
- Open parity issues are parked, not deleted. They are picked up only where they affect gameplay that players notice.
- The marketplace starts with free cosmetics. Paid items wait for a separate review of the Minecraft EULA and commercial usage guidelines.
