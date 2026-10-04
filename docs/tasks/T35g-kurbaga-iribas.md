# T35g — Kurbağa ve iribaş

**Amaç:** `FrogModel` (anahtar kare animasyonları: zıplama, dil, şişme, yüzme), `TadpoleModel`, renderer'ları (kurbağa çeşidi dokuları) ve ilgili istemci durumları (`Frog.DATA_VARIANT`, `DATA_TONGUE_TARGET`, animasyon durumları).
**Bağımlılık:** T35d, T43b. **Boy:** M–L.

## Oku
- Java: `client/model/animal/frog/*.java`, `client/renderer/entity/{Frog,Tadpole}Renderer.java`, `world/entity/animal/frog/{Frog,Tadpole}.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`setup_rabbit`), `crates/voxel-world/src/rabbit.rs`.

## Oracle
`rabbit_model` kalıbı; gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, davranışlar; aksolotl (T35f).

## Devir notu
- Yapıldı: `voxel-world/frog.rs` (poz → zıplama/vıraklama/dil durumları, suda boşta durma, ×25 adım hedefi), çeşit 18 (`frog` çeşit tablosu oracle'dan), `setup_frog` ve `setup_tadpole`, `crates/voxel/src/frogs.rs`. Kayıt: `docs/client-frog-parity-conformance.md`.
- Bulgu: kurbağanın çeşit verisi 18'dedir (17 değil); poz verisi her geldiğinde durum yeniden başlar.
- Sapma: dil hedefi, ses, parçacık, gölge yok; iribaş doku yolu oracle'sız. Canlı gözlem yapılmadı.
