# T35c — Morina, somon, tropikal balık, kirpi balığı

**Amaç:** `CodModel`, `SalmonModel`, `TropicalFishSmallModel`/`LargeModel` (desen ve renk katmanları), `PufferfishSmall/Mid/BigModel` (puf durumu), `AbstractFish`'in `setupRotations`'ı (karada çırpınma, suda kuyruk salınımı), `tadpole`.
**Bağımlılık:** T35b (suda olma durumu). **Boy:** M–L.

## Oku
- Java: `client/model/animal/fish/*.java`, `client/renderer/entity/{Cod,Salmon,TropicalFish,Pufferfish}Renderer.java`, `client/renderer/entity/layers/TropicalFishPatternLayer.java`, `world/entity/animal/fish/*.java` (`AbstractFish`, `TropicalFish.DATA_ID_TYPE_VARIANT`, `Pufferfish.PUFF_STATE`, `Salmon` boyutları).
- Rust: `crates/voxel-world/src/{squid,entity}.rs` (`tracks_water` genişler), `crates/voxel/src/squids.rs` (kalıp).

## Oracle
`squid_model` ve `setup_rotations` kalıbı; gerçek renderer `setupRotations`'ı `isInWater` iki durumunda; desen/renk verisi için `variant-fields` satırları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Kova ile tutma, sesler, parçacıklar; yunus, aksolotl, kaplumbağa, kurbağa (T35d); nautilus ve gardiyan (T35e).

## Devir notu
(boş)
