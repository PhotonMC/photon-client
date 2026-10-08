# T32e — Tilki

**Amaç:** Tilki (yetişkin ve bebek): `FoxModel.setupAnim` (yürüme, çömelme, uyku, oturma, atlayış, yüz üstü), kırmızı/kar dokusu ve uyku dokusu, tutulan eşya katmanı.
**Bağımlılık:** T32c. **Boy:** M.

## Oku
- Java: `client/model/animal/fox/{FoxModel,AdultFoxModel,BabyFoxModel}.java`, `client/renderer/entity/FoxRenderer.java`, `client/renderer/entity/layers/FoxHeldItemLayer.java`, `client/renderer/entity/state/FoxRenderState.java`, `world/entity/animal/fox/Fox.java` (istemci: `getHeadRollAngle`, `getCrouchAmount`, `isSitting`, `isFaceplanted`, `isPouncing`, bayraklar, `tick`).
- Rust: `crates/photon/src/mobs/animals/felines.rs` ve `bears.rs` (kalıp), `photon-render/src/mob_model.rs` (`setup_feline`, `setup_polar_bear`), `photon-world/src/{cat,polar_bear}.rs`, `docs/records/client-polar-bear-parity-conformance.md`, `crates/photon/src/equipment/held_items.rs`.

## Yapılacaklar
1. `setup_fox` (yetişkin + bebek poz farkları; `legMotionPos` modelin kendi durumudur: yüz üstüyken her çizimde artar).
2. `Fox` istemci durumu: baş yuvarlanması, çömelme miktarı (`tick` + `aiStep`), bayraklar (`DATA_FLAGS_ID`), çeşit (`DATA_TYPE_ID`).
3. `setupRotations`: atlarken/yüz üstüyken `-xRot` ile XP dönüşü (`MobPlacement` alanı gerekebilir).
4. `FoxHeldItemLayer` (ana el eşyası; `held_items.rs` kalıbı).

## Oracle
`feline_model` / `polar_bear_model` kalıbı + `living-entity-animations` gerçek `Fox` ile (gerçek bayrak verisi).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı (`shot`): kırmızı/kar, uyuyan, oturan, çömelen tilki `summon` ile.

## Kapsam dışı
Tilki davranışı/yapay zekâsı (sunucu), güven listesi, sesler.

## Belgeler
`docs/index.md` satırı, yeni `docs/records/client-fox-parity-conformance.md`.

## Devir notu
- Yapıldı: `setup_fox` (yetişkin + bebek: yürüme/anahtar kare, çömelme, uyku, oturma, zıplama, yüz üstü), `fox_item_pose` (ağızdaki eşya), `MobPlacement.pitch`, `photon-world/src/fox.rs` (bayraklar, çeşit, ilgi açısı, çömelme miktarı), `photon_data::entity_type::fox_texture`, `crates/photon/src/mobs/animals/foxes.rs`. Kayıt: `docs/records/client-fox-parity-conformance.md`.
- Oracle: `fox-model` 26 (eşya matrisiyle), `fox-textures` 8, `living-entity-animations` 5, `variant-fields` +2.
- Sapma: `legMotionPos` canlıda karedir (oracle ilk pozu); parçacık ve ses yok; güvenilen varlıklar okunmuyor.
- Açık: T32f panda, T32g ravager, T32h strider, T32i titreme.
