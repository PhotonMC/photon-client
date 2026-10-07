# V06 — Bölümler için indirect/multi-draw

**Amaç:** Bölüm başına draw çağrısını katman başına tek indirect çağrıya indirmek.
**Bağımlılık:** V05. **Boy:** L.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §5 (5.9)
- `vulkan.rs` `bind_section :1849`, `drawn_sections :3316`, arena `:3414`

## Yapılacaklar
- Bölüm ofsetleri SSBO'da (push constant yerine); görünür bölümler için `VkDrawIndexedIndirectCommand` dizisi.
- `vkCmdDrawIndexedIndirect` (multiDrawIndirect özelliği yoksa döngüye geri düşüş).
- Yarı saydam katmanın sıralaması korunur.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit. Bench: yüksek görüş mesafesinde CPU kayıt süresi düşer.

## Kapsam dışı
GPU tarafı ayıklama.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
