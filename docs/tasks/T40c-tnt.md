# T40c — Yanan TNT

**Amaç:** `PrimedTnt` (`TntRenderer`): fitil verisi (`DATA_FUSE_ID` 8, `DATA_BLOCK_STATE_ID` 9) ve istemci sayacı, `fuse < 10` iken şişme, `isLit` ile beyaz yanıp sönme, blok durumunun çizimi (`TntMinecartRenderer.submitWhiteSolidBlock`).
**Bağımlılık:** T40, T38d. **Boy:** M.

## Oku
- Java: `client/renderer/entity/TntRenderer.java`, `world/entity/item/PrimedTnt.java`.
- Rust: `crates/voxel/src/minecarts.rs` (aynı beyaz blok), `crates/voxel-render/src/minecart.rs` (`tnt_swell_amount`, `tnt_is_lit`), `crates/voxel-world/src/entity.rs` (`tick_primed_tnt`).

## Oracle
Gerçek renderer/model sınıfı ve durumu (örnek: `docs/records/client-arrow-parity-conformance.md`, `docs/records/client-minecart-parity-conformance.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Sesler, partiküller, dış çizgi.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: `primed_tnt.rs` (fitil, blok, sayaç; fitil bitince entity silinir), `primed_tnt_pose`, `remote_primed_tnt`. Kayıt: `docs/records/client-primed-tnt-parity-conformance.md`.
- Sapma: sayaç gerçek tike karşı karşılaştırılmadı (birim test); partikül yok.
