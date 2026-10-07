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
- **Ölçüldü, uygulanmadı.** Kare başına CPU komut kaydı (`record`) p50 0.28–0.30 ms, bunun bölüm çizimi payı daha da küçük (Apple M1, görüş 12 = istemcinin üst sınırı, 1345 dolu bölüm). `bind_section` başına push constant + draw çağrısı bu küçük süre içinde; indirect/multi-draw'ın kazanabileceği süre 0.3 ms'yi geçemez. Kare süresini belirleyenler GPU sahnesi (~5.7 ms) ve `submit_present` (~4 ms, MoltenVK).
- Kabul ("yüksek görüş mesafesinde CPU kayıt süresi düşer") için yüksek görüş mesafesi bu istemcide yok (`view_distance().clamp(2, 12)`); görüş sınırı kalkarsa ve kayıt süresi ölçülebilir hâle gelirse yeniden açılır. SSBO'ya taşınan bölüm ofsetleri ayrıca `Vulkan 1.2` özellik ve MoltenVK uyumu gerektirir.
- Yapılan tek şey ölçüm altyapısı: `bench` raporunda `cpu_ms.record` ve `section_memory`.
