# T41f2 — Yerel oyuncunun ve mobların elytrası

**Amaç:** `WingsLayer`'ı yerel oyuncunun (üçüncü şahıs) ve insansı mobların (zombi, iskelet, piglin…) modeline de uygula.
**Bağımlılık:** T41f. **Boy:** S.

## Oku
- Java: `client/renderer/entity/layers/WingsLayer.java`, `client/renderer/entity/HumanoidMobRenderer.java`.
- Rust: `crates/voxel/src/{overlays,mobs}.rs`, `voxel-world/src/local_player.rs`.

## Oracle
Model ve durum T41f'in oracle'ında; burada yalnız bağlama (canlı gözlem).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: elytralı zombi.

## Belgeler
`docs/records/client-armor-parity-conformance.md`.

## Devir notu
- `LocalPlayer` kendi `ElytraState`'ini tikler (bu port'ta oyuncu süzülmez), `Armor::wings` yerel ve mob modellerine de bağlandı; canlıda elytralı zombinin arkasında kanatlar görüldü. Yerel oyuncu için karşılaştırma yok.
- Açık → [T41f3](T41f3-kafatasi-durbun-suzulme.md).
