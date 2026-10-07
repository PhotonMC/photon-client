# T35i — Gardiyanın ışını

**Amaç:** `GuardianRenderer.renderBeam` (özel geometri: dönen dört yüz ve uç, dokulu, tam aydınlık). Hedefe bakış denetimi, parçacık çekilişleri ve karadaki çırpınış T35j'ye ayrıldı.
**Bağımlılık:** T35h. **Boy:** M.

## Oku
- Java: `client/renderer/entity/GuardianRenderer.java` (`submit`, `renderBeam`, `extractRenderState`).
- Rust: `crates/voxel/src/guardians.rs`, `crates/voxel-render/src/{joml,mob_model}.rs`.

## Oracle
Gerçek `renderBeam`, `submitCustomGeometry` ile verilen köşe tüketicisinden yakalanır (`guardian_beam`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacık çizimi, saldırı davranışı; hedefe bakış ve çırpınış (T35j).

## Devir notu
- Yapıldı: `voxel-render/guardian_beam.rs` (`guardian_beam`: gerçek köşeler; `beam_triangles`: doku tekrarını keserek entity boru hattına), `ışın` çizimi `crates/voxel/src/guardians.rs`, saldırı ölçeği. Kayıt: `docs/records/client-guardian-beam-parity-conformance.md`.
- Bulgu yok; geometri 14 satırda ilk denemede eşleşti (`acos`/`atan2` Rust `f64`).
- Bölündü: hedefe bakış, parçacık çekilişleri ve karadaki çırpınış T35j'de.
- Sapma: görüş alanı elemesi yok; yerel oyuncunun boyu kutudan. Canlı gözlem yapılmadı.
