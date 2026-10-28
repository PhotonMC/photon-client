# Documentation

| Page | What it covers |
| --- | --- |
| [Architecture overview](architecture/overview.md) | Crates, layering, threads, the main loop |
| [Rendering](architecture/rendering.md) | The Vulkan renderer today and its GPU-driven direction |
| [Decisions (ADRs)](adr/) | Why the stack and the plans are what they are |
| [Parity](parity/README.md) | How behaviour is proven against the Java client |
| [GUI oracle](parity/gui-oracle.md) | Screen parity through recorded draw calls |
| [Development setup](development/setup.md) | Requirements, reference setup, running, checks |
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
- [0010 One product](adr/0010-one-product.md)
- [0013 Multi-version content model](adr/0013-multi-version-content-model.md)
- [0014 Multiplayer-only first release](adr/0014-multiplayer-only-first-release.md)

`records/` holds legacy per-slice parity records that are being converted into case metadata ([ADR 0005](adr/0005-parity-v2.md)).
