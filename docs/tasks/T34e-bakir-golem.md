# T34e — Bakır golem

**Amaç:** `CopperGolemModel` (anahtar kare animasyonları, eşya taşıma, oksitlenme aşamaları, antenin çiçek açma durumu), `CopperGolemRenderer`.
**Bağımlılık:** T43b (anahtar kare oynatıcı). **Boy:** M–L.

## Oku
- Java: `client/model/animal/golem/CopperGolemModel.java`, `client/renderer/entity/CopperGolemRenderer.java` ve katmanları, `world/entity/animal/golem/CopperGolem.java`.
- Rust: `crates/voxel/src/golems.rs` (kalıp), `voxel-render/src/mob_model.rs`.

## Oracle
`ironGolemModel` kalıbı: gerçek model + `setupAnim`; animasyon durumları T43b'nin oynatıcısıyla.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, düğme/heykel dönüşümü.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
