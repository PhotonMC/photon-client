# T35d — Yunus, aksolotl, kaplumbağa, iribaş ve kurbağa

**Amaç:** `DolphinModel`, `AxolotlModel` (anahtar kare animasyonları, ölü taklidi), `TurtleModel`, `TadpoleModel`, `FrogModel` (anahtar kare animasyonları: zıplama, dil, şişme), renderer dönüşleri ve ilgili istemci durumları.
**Bağımlılık:** T35b, T43b. **Boy:** L (sığmazsa böl: yunus+kaplumbağa, aksolotl, kurbağa+iribaş).

## Oku
- Java: `client/model/animal/{dolphin,axolotl,turtle,frog}/*.java`, ilgili `client/renderer/entity/*Renderer.java`, `world/entity/animal/{dolphin,axolotl,turtle,frog}/*.java` (animasyon durumları, `DATA_*`).
- Rust: `crates/voxel-world/src/{squid,rabbit,copper_golem}.rs` (durum ve tik kalıpları), `crates/voxel-render/src/entity/mob_model.rs` (`setup_rabbit`: animasyon uygulama kalıbı).

## Oracle
`rabbit_model` ve `copper_golem_model` kalıbı (gerçek model + animasyon durumları), gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Nautilus ve gardiyan (T35e), sesler, parçacıklar, davranışlar.

## Devir notu
- Yapıldı: yunus ve kaplumbağa (yetişkin ve bebek): `setup_dolphin`, `setup_turtle`, `dolphin_item_pose`, `crates/voxel/src/{dolphins,turtles}.rs`, kaplumbağa verisi 18/19 ve suda olma izlemesi, `is_moving_horizontally`. Kayıt: `docs/records/client-sea-animals-parity-conformance.md`.
- Bölündü: aksolotl [T35f](T35f-aksolotl.md), kurbağa ve iribaş [T35g](T35g-kurbaga-iribas.md).
- Sapma: bebek kaplumbağanın yüz atması (`entityCutoutCull`) ayırt edilmiyor; ses, parçacık, gölge yok.
