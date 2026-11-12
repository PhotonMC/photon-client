# Documentation

| Page | What it covers |
| --- | --- |
| [Architecture overview](architecture/overview.md) | Crates, layering, threads, the main loop |
| [Rendering](architecture/rendering.md) | The Vulkan renderer today and its GPU-driven direction |
| [Decisions (ADRs)](adr/) | Why the stack and the plans are what they are |
| [Parity](parity/README.md) | How behaviour is proven against the Java client |
| [GUI oracle](parity/gui-oracle.md) | Screen parity through recorded draw calls |
| [Development setup](development/setup.md) | Requirements, reference setup, running, checks |
| [Releasing](development/releasing.md) | Release key, signing and the release workflow |
| [Coding standards](development/coding-standards.md) | Enforced limits and conventions |
| [History](history.md) | What has been built so far |

Decisions:

- [0001 Vulkan renderer](adr/0001-vulkan-renderer.md)
- [0002 Audio stack](adr/0002-audio-stack.md)
- [0003 `bevy_ecs`](adr/0003-bevy-ecs.md)
- [0004 Version support](adr/0004-version-support.md)
- [0005 Parity v2](adr/0005-parity-v2.md)
- [0006 Mod platform](adr/0006-mod-platform.md)
- [0007 Assets and accounts](adr/0007-assets-and-accounts.md)
- [0008 Vanilla and Enhanced](adr/0008-vanilla-and-enhanced.md) (superseded by 0010)
- [0009 Downloads and archives in xtask](adr/0009-xtask-downloads.md)
- [0010 One product](adr/0010-one-product.md) (superseded by 0018)
- [0013 Multi-version content model](adr/0013-multi-version-content-model.md)
- [0014 Multiplayer-only first release](adr/0014-multiplayer-only-first-release.md) (superseded by 0018)
- [0015 Signed updates](adr/0015-signed-updates.md)
- [0016 Instanced entity drawing](adr/0016-instanced-entity-drawing.md)
- [0017 Packed terrain vertices](adr/0017-packed-terrain-vertices.md)
- [0018 Photon platform](adr/0018-photon-platform.md)
- [0019 photon-server on an unmodified Pumpkin](adr/0019-photon-server.md)
- [0020 Accounts and launch links](adr/0020-accounts-and-launch-links.md)
- [0021 Client UI technology and design tokens](adr/0021-client-ui-technology.md)
- [0022 Backend services](adr/0022-backend-services.md)
- [0023 Website sign-in](adr/0023-website-sign-in.md)
- [0024 Server-delivered mods](adr/0024-server-delivered-mods.md)
- [0025 Native video on game pages](adr/0025-native-video.md)
- [0026 Join tickets for Photon servers](adr/0026-join-tickets.md) (proposed)

`records/` holds legacy per-slice parity records that are being converted into case metadata ([ADR 0005](adr/0005-parity-v2.md)).
