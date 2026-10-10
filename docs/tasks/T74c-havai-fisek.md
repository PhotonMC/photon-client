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
Yapıldı: `particles/firework.rs` (başlatıcı, kıvılcım, parlama), `fireworks` bileşeni ve olay 17, `LevelEvent::Fireworks`; kayıt [client-particle-firework-parity-conformance.md](../records/client-particle-firework-parity-conformance.md). Oracle `particle-engine` 11 satır (iz hariç). Sapma/açık: mesafe gecikmesi, canlı gözlem, iz yalnız birim testli.
