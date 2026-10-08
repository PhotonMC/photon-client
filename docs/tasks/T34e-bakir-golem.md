# T34e — Bakır golem

**Amaç:** `CopperGolemModel` (anahtar kare animasyonları, eşya taşıma, oksitlenme aşamaları, antenin çiçek açma durumu), `CopperGolemRenderer`.
**Bağımlılık:** T43b (anahtar kare oynatıcı). **Boy:** M–L.

## Oku
- Java: `client/model/animal/golem/CopperGolemModel.java`, `client/renderer/entity/CopperGolemRenderer.java` ve katmanları, `world/entity/animal/golem/CopperGolem.java`.
- Rust: `crates/photon/src/mobs/humanoids/golems.rs` (kalıp), `photon-render/src/mob_model.rs`.

## Oracle
`ironGolemModel` kalıbı: gerçek model + `setupAnim`; animasyon durumları T43b'nin oynatıcısıyla.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, düğme/heykel dönüşümü.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: `CopperGolemAnimations` (beş `AnimationState`, boşta bekleme, `Mob.baseTick` çekilişi), veri 16/17 (hava, durum), `setup_copper_golem` (bakış, yürüyüş ve eşyalı yürüyüş, kol sınırı, beş animasyon), `copper_golems.rs` (aşamaya göre doku, parlayan gözler). Kayıt: `docs/records/client-copper-golem-parity-conformance.md`.
- Oracle: `entity/copper-golem-model` 45, `living-entity-animations`'a 19 gerçek `CopperGolem` tiki satırı, `data-ids` 2 satır. Oracle'a başsız ses için kamera kuklası eklendi.
- Ayrıldı → T34f: eldeki eşya, antendeki blok, kafa eşyası katmanları. Sapma: gölge ve sesler yok; canlı gözlem yapılmadı.
