# T37f2 — Sniffer

**Amaç:** `SnifferModel`/`SniffletModel` (koklama, kazma, yürüme, ayağa kalkma, mutlu ve bebek animasyonları), `SnifferRenderer` ve `Sniffer` istemci durumu (`SNIFFER_STATE` verisi, animasyon durumları, yavru dönüşümü).
**Bağımlılık:** T37f. **Boy:** M.

## Oku
- Java: `world/entity/animal/sniffer/Sniffer.java`, `client/model/animal/sniffer/*.java`, `client/renderer/entity/{SnifferRenderer,state/SnifferRenderState}.java`.
- Rust: kalıp olarak `crates/voxel-world/src/armadillo.rs`, `crates/voxel/src/armadillos.rs`, `setup_armadillo` (`crates/voxel-render/src/mob_model.rs`). Animasyon tabloları (`Sniffer.*`) ve modeller (`sniffer#main`, `sniffer_baby#main`) üretilmiş durumda.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
