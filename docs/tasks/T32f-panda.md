# T32f — Panda

**Amaç:** Panda (yetişkin ve bebek): `PandaModel.setupAnim` (oturma, yuvarlanma, sırtüstü, hapşırma, yeme), gen dokusu, tutulan eşya katmanı.
**Bağımlılık:** T32c. **Boy:** M.

## Oku
- Java: `client/model/animal/panda/{PandaModel,BabyPandaModel}.java`, `client/renderer/entity/PandaRenderer.java`, `client/renderer/entity/layers/PandaHoldsItemLayer.java`, `client/renderer/entity/state/PandaRenderState.java`, `world/entity/animal/panda/Panda.java` (istemci: sayaçlar `sitAmount`, `onBackAmount`, `rollAmount`, `unhappyCounter`, `sneezeCounter`, `eatCounter`, genler, `tick`).
- Rust: `crates/voxel/src/bears.rs`, `felines.rs` (kalıp), `voxel-render/src/mob_model.rs` (`setup_polar_bear`), `voxel-world/src/{cat,polar_bear}.rs`, `docs/client-polar-bear-parity-conformance.md`.

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
`docs/index.md` satırı, yeni `docs/client-panda-parity-conformance.md`.

## Devir notu
(boş)
