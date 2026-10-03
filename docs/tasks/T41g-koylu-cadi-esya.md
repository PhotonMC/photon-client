# T41g — Köylü, tüccar ve cadının tuttuğu ve taktığı eşya

**Amaç:** `CrossedArmsItemLayer`, `WitchItemLayer` ve köylü/tüccar `CustomHeadLayer`'ı (T41d'den ayrıldı: kullanım tiki istemeyen kısım).
**Bağımlılık:** T34b, T41c. **Boy:** S.

## Oku
- Java: `client/renderer/entity/layers/{CrossedArmsItemLayer,WitchItemLayer,CustomHeadLayer}.java`, `client/renderer/entity/state/HoldingEntityRenderState.java`, `VillagerRenderer.CUSTOM_HEAD_TRANSFORMS`.
- Rust: `crates/voxel-render/src/held_item.rs`, `crates/voxel/src/{held_items,villagers}.rs`.

## Yapılacaklar
1. Üç katmanın pozları (oracle ile) ve `Place::Ground` önbelleği.
2. Köylü, tüccar ve cadı eşyalarını `held` listesine toplamak.

## Oracle
`villager_held_layers`: gerçek katmanlar, yakalayıcı toplayıcı.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon` ile eşyalı köylü, cadı, tüccar.

## Kapsam dışı
Kullanım pozları (T41d).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: `crossed_arms_item_pose`, `witch_potion_pose`, `Place::Ground`, `HeldItem::stack`; köylü/tüccar/cadı eşyaları ve kafa eşyası çizilir. Kayıt: `docs/client-villager-held-parity-conformance.md`.
- Oracle: `entity/villager-held-layers` 9 satır. Canlı: zümrüt, iksir, elmas ve balkabağı başlıklar `shot` ile görüldü.
- Sapma: yok (zombi köylü eşyası T28/T41 yolunda).
