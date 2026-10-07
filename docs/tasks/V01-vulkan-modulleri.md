# V01 — `vulkan.rs` modüllere bölme ve RAII

**Amaç:** 5.4k satırlık arka ucu sorumluluklara bölmek ve Vulkan nesnelerini sahiplik tiplerine almak; davranış değişmez.
**Bağımlılık:** V00. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §5
- `crates/voxel-render/src/vulkan.rs` (yapı `:1285`, `create` `:1411`, `Drop` `:5145`)

## Yapılacaklar
- `vk/{instance,device,swapchain,memory,pipeline,texture,frame,shaders}.rs`; `VulkanBackend` yalnız bileşim ve genel API.
- `Buffer`, `Image`, `Pipeline` gibi sahibi olan küçük tipler; yok etme sırası tipte belgeli.
- `set_entities`/`set_shadows`/… gibi tekrar eden fonksiyonlar tek `set_draw(kind, vertices)` üzerinden (API değişikliği çağıranlarda mekanik).

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit. Bench sayıları V00 ile aynı aralıkta.

## Kapsam dışı
Senkronizasyon değişikliği (V03).

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
