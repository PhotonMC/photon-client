# T35f — Aksolotl

**Amaç:** `AxolotlModel` (yetişkin ve bebek), `AxolotlRenderer` (renk çeşidi dokuları), anahtar kare animasyonları (yüzme, yürüme, suda durma, ölü taklidi) ve ilgili istemci durumları (`Axolotl.DATA_VARIANT`, `DATA_PLAYING_DEAD`, `FROM_BUCKET`).
**Bağımlılık:** T35d, T43b. **Boy:** M.

## Oku
- Java: `client/model/animal/axolotl/*.java`, `client/renderer/entity/AxolotlRenderer.java`, `client/renderer/entity/state/AxolotlRenderState.java`, `world/entity/animal/axolotl/Axolotl.java` (animasyon durumları, `DATA_*`, `AxolotlAnimation`).
- Rust: `crates/photon-render/src/entity/mob_model.rs` (`setup_turtle`, `setup_rabbit`: animasyon uygulama kalıbı), `crates/photon-world/src/{rabbit,fish}.rs`, `crates/photon/src/{turtles,fish}.rs`.

## Oracle
`sea_animal_model` ve `rabbit_model` kalıbı (gerçek model + animasyon durumları), gerçek varlık tiki için `living-entity-animations` (`axolotl` ekstrası).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Kova ile tutma, sesler, parçacıklar, davranışlar; kurbağa ve iribaş (T35g).

## Devir notu
- Yapıldı: `photon-world/axolotl.rs` (`BinaryAnimator` + `in_out_sine`, yedi animasyon durumu, bebek/yetişkin tiki), tür 18 ve ölü taklidi 19, `setup_axolotl_adult` (beş poz + ayna) ve `setup_axolotl_baby` (keyframe), `crates/photon/src/mobs/aquatic/axolotls.rs`. Kayıt: `docs/records/client-axolotl-parity-conformance.md`.
- Bulgular: bebek mob yürüyüş konumu ×3 (`follow(.., baby)`) tüm bebekler için düzeltildi; bebek yunus `DATA_BABY_ID`'yi okumuyordu, düzeltildi.
- Sapma: doku yolu oracle'sız; ses, parçacık, gölge yok. Canlı gözlem yapılmadı.
