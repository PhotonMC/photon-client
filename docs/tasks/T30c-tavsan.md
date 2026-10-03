# T30c — Tavşan

**Amaç:** Tavşan (`AdultRabbitModel`/`BabyRabbitModel`, zıplama ve boşta baş eğme `KeyframeAnimation`'ları, `jumpCompletion`, yedi çeşit, bebek, "Toast" adı).
**Bağımlılık:** T43b. **Boy:** M.

## Oku
- Java: `client/model/animal/rabbit/{RabbitModel,AdultRabbitModel,BabyRabbitModel}.java`, `client/animation/definitions/{RabbitAnimation,BabyRabbitAnimation}.java`, `client/renderer/entity/RabbitRenderer.java`, `world/entity/animal/rabbit/Rabbit.java` (`getJumpCompletion`, `hopAnimationState`, `idleHeadTiltAnimationState`, `getVariant`, `DATA_TYPE_ID`).
- Rust: T43b'nin oynatıcısı, `crates/voxel/src/farm_animals.rs` (kalıp), `voxel-world/src/animal.rs`.

## Yapılacaklar
1. `RabbitModel.setupAnim`: baş bakışı (boşta baş eğme yokken), iki animasyonun uygulanması.
2. Çeşit verisi ve dokular, `jumpCompletion`, animasyon durumlarını süren paket/olay kolları.

## Oracle
`goat_model` kalıbı: gerçek model + `setupAnim`; animasyon durumları `living-entity-animations` kalıbıyla gerçek `Rabbit` tikinden.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon rabbit`.

## Kapsam dışı
Killer Bunny davranışı, binme, sesler.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: `setup_rabbit`, `RabbitState` (zıplama ve boşta eğme durumları, olay 1, `jump_completion`), çeşit verisi ve dokular (`farm_animals.rs`). Kayıt: `docs/client-keyframe-animation-parity-conformance.md`.
- Oracle: `rabbit-model` 12, `living-entity-animations` 5 tavşan satırı (gerçek `Rabbit` tiki; bekleme ve tohum sabit). Canlı: dört çeşit `shot` ile görüldü.
- Sapma: tasma izlenmiyor, "Toast" dokusu yok, bekleme tohumsuz (referans gibi).
