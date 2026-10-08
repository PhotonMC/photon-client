# T37f2 — Sniffer

**Amaç:** `SnifferModel`/`SniffletModel` (koklama, kazma, yürüme, ayağa kalkma, mutlu ve bebek animasyonları), `SnifferRenderer` ve `Sniffer` istemci durumu (`SNIFFER_STATE` verisi, animasyon durumları, yavru dönüşümü).
**Bağımlılık:** T37f. **Boy:** M.

## Oku
- Java: `world/entity/animal/sniffer/Sniffer.java`, `client/model/animal/sniffer/*.java`, `client/renderer/entity/{SnifferRenderer,state/SnifferRenderState}.java`.
- Rust: kalıp olarak `crates/photon-world/src/mobs/armadillo.rs`, `crates/photon/src/mobs/animals/armadillos.rs`, `setup_armadillo` (`crates/photon-render/src/entity/mob_model.rs`). Animasyon tabloları (`Sniffer.*`) ve modeller (`sniffer#main`, `sniffer_baby#main`) üretilmiş durumda.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `setup_sniffer` (iki model aynı sınıf), `photon-world/sniffer.rs` (`DATA_STATE` 17/35, animasyon durumları), `sniffers.rs` çizici. Kayıt: `docs/records/client-sniffer-parity-conformance.md`.
- Sapma: kazarken vuruş kutusu küçülmesi (`DIGGING_DIMENSIONS`), ses, parçacık, tohum bırakma yok; canlı gözlem yapılmadı.
