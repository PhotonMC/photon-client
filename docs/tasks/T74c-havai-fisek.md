# T74c — Havai fişek parçacıkları

**Amaç:** `FireworkParticles` (başlatıcı: patlama biçimleri, sesler; kıvılcım: renk solması, iz,
parıltı; parlama), `ClientLevel.createFireworks` ve roketin `EntityEvent` 17'si, roket yığınındaki
`fireworks` bileşeninin patlama listesi (biçim, renkler, solma renkleri, iz, parıltı).
**Bağımlılık:** T74b. **Boy:** L.

## Oku
Java: `client/particle/FireworkParticles.java`, `client/multiplayer/ClientLevel.java`
(`createFireworks`), `world/entity/projectile/FireworkRocketEntity.java` (`handleEntityEvent`),
`world/item/component/{Fireworks,FireworkExplosion}.java`. Rust: `crates/photon-world/src/particles/hit.rs`.

## Oracle
`particle-engine` satırları (başlatıcı bir patlama listesiyle).

## Kabul
`CHECK OK`.

## Belgeler
Yeni kayıt.

## Devir notu
