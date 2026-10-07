# T32f — Panda

**Amaç:** Panda (yetişkin ve bebek): `PandaModel.setupAnim` (oturma, yuvarlanma, sırtüstü, hapşırma, yeme), gen dokusu, tutulan eşya katmanı.
**Bağımlılık:** T32c. **Boy:** M.

## Oku
- Java: `client/model/animal/panda/{PandaModel,BabyPandaModel}.java`, `client/renderer/entity/PandaRenderer.java`, `client/renderer/entity/layers/PandaHoldsItemLayer.java`, `client/renderer/entity/state/PandaRenderState.java`, `world/entity/animal/panda/Panda.java` (istemci: sayaçlar `sitAmount`, `onBackAmount`, `rollAmount`, `unhappyCounter`, `sneezeCounter`, `eatCounter`, genler, `tick`).
- Rust: `crates/voxel/src/mobs/animals/bears.rs`, `felines.rs` (kalıp), `voxel-render/src/mob_model.rs` (`setup_polar_bear`), `voxel-world/src/{cat,polar_bear}.rs`, `docs/records/client-polar-bear-parity-conformance.md`.

## Yapılacaklar
1. `setup_panda` ve panda istemci durumu (miktarlar tikte ilerler, `Mth.lerp` ile çizimde ara değer).
2. Gen verisi (ana/gizli gen) ve doku seçimi (`Panda.Gene`).
3. `setupRotations`: yuvarlanma ve sırtüstü dönüşleri (`MobPlacement` alanları gerekebilir).
4. `PandaHoldsItemLayer`.

## Oracle
`feline_model` / `polar_bear_model` kalıbı + `living-entity-animations` gerçek `Panda` ile.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): her gen, oturan, yuvarlanan panda `summon` ile.

## Kapsam dışı
Panda davranışı (sunucu), sesler.

## Belgeler
`docs/index.md` satırı, yeni `docs/records/client-panda-parity-conformance.md`.

## Devir notu
- Yapıldı: `setup_panda` (yetişkin + bebek), `PandaRotation`/`panda_rotate` (takla, oturma, sırtüstü) ve `panda_item_pose`, `MobPlacement::cat_lying`/`fox_tipping` ve `placement_rotations` (kedi ve tilki dönüşleri artık oracle'da), `voxel-world/src/panda.rs` (sayaçlar, gen, miktarlar, yeme çekimi), dünyaya hava (`set_weather`, `is_thundering`), `crates/voxel/src/mobs/animals/pandas.rs`. Kayıt: `docs/records/client-panda-parity-conformance.md`.
- Oracle: `panda-model` 40, `panda-textures` 14, `setup-rotations` 84, `living-entity-animations` 9 panda, `variant-fields` +6; harness'a `weather`, `equipment`, `panda_data` olayları.
- Sapma: hapşırık sonu ve beşinci yeme tikinden sonrası parçacık yüzünden oracle'da koşulamaz (birim test); suda olma izlenmiyor; ses/parçacık yok.
- Açık: T32g ravager, T32h strider, T32i titreme.
