# T32d — Hoglin, zoglin, ravager, strider

**Amaç:** Hoglin ve zoglin (`HoglinModel`), ravager (`RavagerModel`, saldırı/sersemleme/kükreme sayaçları) ve strider (`StriderModel`, eyer, soğuk doku).
**Bağımlılık:** T32. **Boy:** M.

## Oku
- Java: `client/model/monster/hoglin/HoglinModel.java`, `client/model/monster/ravager/RavagerModel.java`, `client/model/animal/strider/StriderModel.java`, ilgili `*Renderer.java`, `world/entity/monster/{hoglin/Hoglin,Ravager}.java`, `world/entity/monster/Strider.java`.
- Rust: `crates/voxel/src/wolves.rs` (kalıp: çeşit dokusu, tasma katmanı), `voxel-render/src/mob_model.rs` (`setup_wolf`), `voxel-world/src/wolf.rs`, `docs/client-wolf-parity-conformance.md`.

## Yapılacaklar
1. Üç ailenin `setupAnim` dalları; `attackAnimationRemainingTicks`, `stunnedTick`, `roarTick` istemci durumu.
2. Strider soğuk dokusu ve baş/saç salınımı.

## Oracle
`wolf_model` / `living-entity-animations` kalıbı: gerçek model + `setupAnim`; tik durumu gerçek varlıkla.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır.

## Kapsam dışı
Evcil hayvan etkileşimi (sunucu), zırh/ekipman katmanları.

## Belgeler
`docs/index.md` satırı, yeni `docs/client-*-parity-conformance.md`.

## Devir notu
(boş)
