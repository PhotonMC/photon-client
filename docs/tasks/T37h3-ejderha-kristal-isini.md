# T37h3 — Ender ejderhasının kristal ışını

**Amaç:** `EnderCrystal` varlığı (çizim: çekirdek ve çerçeveler, `EndCrystalRenderer.getY` sıçraması) ve ejderhanın `nearestCrystal` seçimi (`checkCrystals`, 32 blok, rastgele), `submitCrystalBeams` (`endCrystalBeam`) ve kristalin kendi ışını (`BeamTarget`).
**Bağımlılık:** T37h. **Boy:** M.

## Oku
- Java: `world/entity/boss/enderdragon/{EnderDragon,EndCrystal}.java`, `client/renderer/entity/{EnderDragonRenderer,EndCrystalRenderer}.java`, `client/model/object/crystal/EndCrystalModel.java`.
- Rust: `crates/voxel/src/mobs/monsters/ender_dragons.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `setup_end_crystal` (+ `ModelPart.rotateBy`: `Matrix3.rotation_zyx`, euler açıları; `Quaternion::{set_angle_axis, rotate_axis}`), `crystal_beam.rs` (geometri, kesilen tekrarlar, siyahtan beyaza renk), `end_crystal.rs` (zaman, ışın hedefi, taban), `end_crystals.rs` çizici, ejderhanın en yakın kristali ve ışını, entity gölgelendiricisinde tek yan ve siyah işareti. Kayıt: `docs/records/client-end-crystal-parity-conformance.md`.
- Düzeltme: ölürken ejderhanın konumu artık yaklaşmıyor (T37h'de yanlıştı).
- Sapma: ışık/sis ve gölgelendirici dalları oracle'sız; ışının görünür canlı karesi yok; ejderha can artışı zamanlaması karşılaştırılmaz.
