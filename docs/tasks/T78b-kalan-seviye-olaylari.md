# T78b — Kalan seviye olayları

**Amaç:** `LevelEventHandler`'ın T78'de kalan kolları: 1500 kompostçu, 1504 sarkıt damlası,
1505 kemik tozu (`BoneMealItem.addGrowthParticles`), 2002/2007 sıçrayan iksir (eşya ve büyü
parçacıkları), 2003 ender gözü, 2004 spawner, 2006 ejderha nefesi, 2011–2013, 3000–3021
(geçit, sculk, titreşim, deneme spawner'ı, kasa…). Ayrıca küresel olaylar 1023/1028/1038
(kameradan 2 blok), 1032 portal yolculuğu (yerel ortam sesi), jukebox 1010/1011.
**Bağımlılık:** T78, T77b. **Boy:** L.

## Oku
Java: `client/renderer/LevelEventHandler.java` (330–700), `world/item/BoneMealItem.java`,
`world/level/block/{ComposterBlock,PointedDripstoneBlock}.java`, `util/ParticleUtils.java`.
Rust: `crates/photon-world/src/particles/level_events.rs`.

## Oracle
`movement/particle-engine` `level_event` olayları (T78 satırları örnek).

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-level-event-parity-conformance.md`.

## Devir notu
Yapılan: `particles/level_events_more.rs`: 1500, 2002/2007, 2003, 2004, 2011/2012, 2013. Oracle motoru
tohum denemelerinde seviye rastgelesini de geri sarar; `item` sağlayıcısı ilk sonda resmini kullanır.
Oracle: `level-event-more` bir `sin` ulp'i dışında eşit (Q04 sınıfı); 2013 satırda yok.
Bölündü: kalan olaylar → T78e.
