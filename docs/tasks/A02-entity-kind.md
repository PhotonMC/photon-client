# A02 — `ClientEntity` → `EntityBase` + `EntityKind` enum

**Amaç:** Türe özgü ~30 `Option` alanını toplam tipe çevirmek; geçersiz durumları temsil edilemez yapmak.
**Bağımlılık:** A01. **Boy:** L.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §3.1
- `crates/voxel-world/src/entity.rs:661-818`, tür modülleri (`rabbit.rs`, `warden.rs`…)

## Yapılacaklar
- `EntityBase` (ortak), `LivingState` (canlılar), `EntityKind` enum'u (büyük durumlar `Box`).
- Erişimciler `match`/`if let EntityKind::Rabbit(..)` ile; türü değişmeyen entity için kurulumda bir kez seçilir.
- Sığmazsa aile aile (canlılar, araçlar, mermiler) bölünür.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Render tarafı (A03).

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
