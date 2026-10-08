# T76b — Çığlık, elder gardiyan parçacığı ve `Explode` paketi

**Amaç:** `ShriekParticle` (gecikmeli, yere yatık iki yüzlü dörtgen), `ElderGuardianParticle`
(`ELDER_GUARDIANS` grubunda model çizimi) ve `ClientboundExplodePacket` (oyuncu itişi, patlama
parçacığı ve ağırlıklı blok parçacığı listesi, ses).
**Bağımlılık:** T76. **Boy:** M.

## Oku
Java: `client/particle/{ShriekParticle,ElderGuardianParticle,ElderGuardianParticleGroup}.java`,
`network/protocol/game/ClientboundExplodePacket.java`, `ClientPacketListener.handleExplosion`.
Rust: `crates/photon-world/src/particles/hit.rs`.

## Oracle
`particle-ticks` satırları (çığlık), `particle-engine` için `Explode` olayı.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-particle-hit-parity-conformance.md`.

## Devir notu
Yapılan: `Explode` paketi (36; parçacık seçeneği okuyucusu ortak), oyuncu itmesi, `particles/explosion.rs`
(ses, merkez parçacığı, `ClientExplosionTracker`); patlama seviye olayı kuyruğunda, izleyici tik başında.
Oracle: `explosions` satırı bit eşit; paket okuyucu birim testli.
Bölündü: çığlık ve elder gardiyan → T76c.
