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

`ShriekParticle` bitti: `ParticleData::Shriek{delay}`, `Mark::Shriek` (`crates/photon-world/src/
particles/hit.rs`), gecikme sıfıra inene dek hiçbir şey yapmayan (yaş artmaz, fizik yok), sonra
normal `base_tick`'e düşen özel erken dönüş (`Kind::Shriek{delay} if delay>0`,
`crates/photon-world/src/particles/particle.rs`). `getQuadSize`/`getLightCoords` örtmeleri
`look.rs`'de. Ağ çözümü `photon-net` (tek varint gecikme). Oracle: `MovementParticleOracle`'ın
`registered` listesine/`optionsOf`'a eklendi, 2 yeni `particle-ticks` satırı (gecikmeli/gecikmesiz).
Kanıt: `python3 tools/photon.py parity --family movement` yeşil (yalnız bilinen Q04 farkı).

`ElderGuardianParticle`/`ElderGuardianParticleGroup` (kameraya bağlı model çizimi, ayrı render
grubu) bu oturuma sığmadı: [T76c2](T76c2-elder-gardiyan.md)'ye bölündü.
