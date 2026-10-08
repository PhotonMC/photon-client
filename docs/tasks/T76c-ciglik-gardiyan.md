# T76c — Çığlık ve elder gardiyan parçacıkları

**Amaç:** `ShriekParticle` (gecikmeli, yere yatık iki yüzlü dörtgen; `ShriekParticleOption` gecikmesi)
ve `ElderGuardianParticle` (`ELDER_GUARDIANS` grubunda elder gardiyan modelinin ekrana çizimi).
**Bağımlılık:** T76b. **Boy:** M.

## Oku
Java: `client/particle/{ShriekParticle,ElderGuardianParticle,ElderGuardianParticleGroup}.java`,
`core/particles/ShriekParticleOption.java`. Rust: `crates/photon-world/src/particles/hit.rs`.

## Oracle
`particle-ticks` satırları (çığlık); gardiyan için çizim durumu.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-particle-hit-parity-conformance.md`.

## Devir notu
