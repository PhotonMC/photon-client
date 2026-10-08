# A03 — Extract → render state → tek geçişli dispatcher

**Amaç:** Mob başına modülleri uygulama crate'inden çıkarmak, kare başına O(tür×entity) taramayı tek geçişe indirmek.
**Bağımlılık:** A02. **Boy:** L.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §4.1, §4.3
- `crates/photon/src/mobs.rs:600-660`, `foxes.rs`, `play.rs:640-700`, `crates/photon-render/src/entity/mob_model.rs`

## Yapılacaklar
- `photon-render::entity`: `RenderContext` (görünüm, partial, dokular, modeller) ve `EntityRenderState`.
- Extract: dünya bir kez gezilir; tür → çizici `match`'i (Java `EntityRenderers` karşılığı).
- Çıktı kare başına yeniden kullanılan `RenderType` kovalarına (V03 ile uyumlu); `Vec` ayırması kare başına sabit.
- Uygulama crate'indeki mob modülleri aile aile taşınır; tuple parametreler bağlam yapısıyla değişir; dize ile tür araması kalkar.

## Kabul
Davranış değişmez: `python3 tools/photon.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit. Bench: entity extract+mesh süresi V00'a göre düşer.

## Kapsam dışı
Çizim çıktısının değişmesi.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
