# T32e — Tilki

**Amaç:** Tilki (yetişkin ve bebek): `FoxModel.setupAnim` (yürüme, çömelme, uyku, oturma, atlayış, yüz üstü), kırmızı/kar dokusu ve uyku dokusu, tutulan eşya katmanı.
**Bağımlılık:** T32c. **Boy:** M.

## Oku
- Java: `client/model/animal/fox/{FoxModel,AdultFoxModel,BabyFoxModel}.java`, `client/renderer/entity/FoxRenderer.java`, `client/renderer/entity/layers/FoxHeldItemLayer.java`, `client/renderer/entity/state/FoxRenderState.java`, `world/entity/animal/fox/Fox.java` (istemci: `getHeadRollAngle`, `getCrouchAmount`, `isSitting`, `isFaceplanted`, `isPouncing`, bayraklar, `tick`).
- Rust: `crates/voxel/src/felines.rs` ve `bears.rs` (kalıp), `voxel-render/src/mob_model.rs` (`setup_feline`, `setup_polar_bear`), `voxel-world/src/{cat,polar_bear}.rs`, `docs/client-polar-bear-parity-conformance.md`, `crates/voxel/src/held_items.rs`.

## Yapılacaklar
1. `setup_fox` (yetişkin + bebek poz farkları; `legMotionPos` modelin kendi durumudur: yüz üstüyken her çizimde artar).
2. `Fox` istemci durumu: baş yuvarlanması, çömelme miktarı (`tick` + `aiStep`), bayraklar (`DATA_FLAGS_ID`), çeşit (`DATA_TYPE_ID`).
3. `setupRotations`: atlarken/yüz üstüyken `-xRot` ile XP dönüşü (`MobPlacement` alanı gerekebilir).
4. `FoxHeldItemLayer` (ana el eşyası; `held_items.rs` kalıbı).

## Oracle
`feline_model` / `polar_bear_model` kalıbı + `living-entity-animations` gerçek `Fox` ile (gerçek bayrak verisi).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): kırmızı/kar, uyuyan, oturan, çömelen tilki `summon` ile.

## Kapsam dışı
Tilki davranışı/yapay zekâsı (sunucu), güven listesi, sesler.

## Belgeler
`docs/index.md` satırı, yeni `docs/client-fox-parity-conformance.md`.

## Devir notu
(boş)
