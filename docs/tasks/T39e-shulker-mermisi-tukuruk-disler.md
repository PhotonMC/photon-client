# T39e — Shulker mermisi, lama tükürüğü ve evoker dişleri

**Amaç:** `ShulkerBulletRenderer`, `LlamaSpitRenderer`, `EvokerFangsRenderer` (modeller, dönüş ve ısırma animasyonu).
**Bağımlılık:** T39. **Boy:** M.

## Oku
- Java: `client/renderer/entity/{ShulkerBulletRenderer,LlamaSpitRenderer,EvokerFangsRenderer}.java` ve modelleri.
- Rust: `crates/voxel-render/src/mob_model.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: üç model + pose'lar, `evoker_fangs.rs` (olay 4, sayaç), `bullets.rs`; kabuk için entity gölgelendiricisinde `shade ≥ 64` = harmanlı alfa. Kayıt: `docs/records/client-small-projectiles-parity-conformance.md`.
- Sapma: fangs durumu gerçek tike karşı karşılaştırılmadı (birim test); kabuk alfası/sırası oracle'sız; istemci hareket tahmini yok → T39g.
