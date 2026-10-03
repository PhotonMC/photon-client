# T31b — Deve, lama ve tüccar lamasi

**Amaç:** Deve ve deve kabuğu (`CamelModel`, `AdultCamelModel`/`BabyCamelModel`, oturma/ayağa kalkma/koşu anahtar kareleri: `CamelAnimation`, `CamelBabyAnimation`), lama ve tüccar lamasi (`LlamaModel`, `BabyLlamaModel`, halı katmanı `LlamaDecorLayer`, çeşit dokuları).
**Bağımlılık:** T31, T43b. **Boy:** M–L.

## Oku
- Java: `client/model/animal/camel/{CamelModel,AdultCamelModel,BabyCamelModel,CamelSaddleModel}.java`, `client/model/animal/llama/{LlamaModel,BabyLlamaModel}.java`, `client/animation/definitions/{CamelAnimation,CamelBabyAnimation}.java`, `client/renderer/entity/{CamelRenderer,CamelHuskRenderer,LlamaRenderer}.java`, `client/renderer/entity/layers/LlamaDecorLayer.java`, `world/entity/animal/camel/Camel.java` (animasyon durumları: `sitAnimationState`, `sitPoseAnimationState`, `sitUpAnimationState`, `idleAnimationState`, `dashAnimationState`), `world/entity/animal/equine/Llama.java`.
- Rust: `crates/voxel/src/farm_animals.rs` (kalıp), `voxel-render/src/{keyframe,mob_model}.rs`, `voxel-world/src/{horse,rabbit,animation_state}.rs`.

## Yapılacaklar
1. Deve: `setupAnim` (baş bakışı, yürüyüş), anahtar kare animasyonları (`Camel.setupAnimationStates`), bebek.
2. Lama: model, çeşit (`Llama.Variant`), halı rengi ve katmanı.

## Oracle
`equine_model` / `rabbit_model` kalıbı: gerçek model + `setupAnim`; animasyon durumları `living-entity-animations` kalıbıyla gerçek varlık tikinden.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon camel`, `llama`.

## Kapsam dışı
Eyer, binme (T44), tükürük mermisi (T39).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
