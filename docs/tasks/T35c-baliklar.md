# T35c — Morina, somon, tropikal balık, kirpi balığı

**Amaç:** `CodModel`, `SalmonModel`, `TropicalFishSmallModel`/`LargeModel` (desen ve renk katmanları), `PufferfishSmall/Mid/BigModel` (puf durumu), `AbstractFish`'in `setupRotations`'ı (karada çırpınma, suda kuyruk salınımı), `tadpole`.
**Bağımlılık:** T35b (suda olma durumu). **Boy:** M–L.

## Oku
- Java: `client/model/animal/fish/*.java`, `client/renderer/entity/{Cod,Salmon,TropicalFish,Pufferfish}Renderer.java`, `client/renderer/entity/layers/TropicalFishPatternLayer.java`, `world/entity/animal/fish/*.java` (`AbstractFish`, `TropicalFish.DATA_ID_TYPE_VARIANT`, `Pufferfish.PUFF_STATE`, `Salmon` boyutları).
- Rust: `crates/voxel-world/src/{squid,entity}.rs` (`tracks_water` genişler), `crates/voxel/src/mobs/aquatic/squids.rs` (kalıp).

## Oracle
`squid_model` ve `setup_rotations` kalıbı; gerçek renderer `setupRotations`'ı `isInWater` iki durumunda; desen/renk verisi için `variant-fields` satırları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Kova ile tutma, sesler, parçacıklar; yunus, aksolotl, kaplumbağa, kurbağa (T35d); nautilus ve gardiyan (T35e).

## Devir notu
- Yapıldı: `voxel-world/fish.rs` (somon boyutu, kirpi balığı katmanı ve ölçeği, tropikal desen/boya çözümü, doku yolları), veri 17 (`fish_variant`), `tracks_water` dört balığa, balık kutusu ölçeği (`bounding_box`, suda olma taraması); `MobModel::{setup_tail_fish, setup_salmon, setup_pufferfish}`, `FishTurns` ve `{cod,salmon,tropical_fish,pufferfish}_turns`; `crates/voxel/src/mobs/aquatic/fish.rs` (tropikal balıkta taban rengi + desen katmanı). Kayıt: `docs/records/client-fish-parity-conformance.md`.
- Oracle: `entity/fish-model` 88, `fish-rotations` 24 (üretimdeki feet/scale önsözüyle; kirpi balığının dönüşsüz yolu sıfır işareti için ayrı), `fish-textures` 23, `variant-fields`'e 3, `living-entity-animations`'a 10 satır.
- Sapma: karada çırpınma hareketi, ses, gölge, akıntı yok. Iribaş (tadpole) T35d'ye kaldı (görev amacından çıkarıldı). Canlı gözlem yapılmadı.
