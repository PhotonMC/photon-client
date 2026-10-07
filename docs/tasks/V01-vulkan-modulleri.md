# V01 — `vulkan.rs` modüllere bölme ve RAII

**Amaç:** 5.4k satırlık arka ucu sorumluluklara bölmek ve Vulkan nesnelerini sahiplik tiplerine almak; davranış değişmez.
**Bağımlılık:** V00. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §5
- `crates/voxel-render/src/vulkan/` (V01 sonrası; öncesi tek dosya `vulkan.rs`)

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
- `vulkan.rs` (5780 satır) `vulkan/` altında 12 dosyaya bölündü (`tools` betiği: üretici geçici, depoya girmedi): `mod.rs` (türler, `VulkanBackend` alanları, sabitler), `shaders.rs` (GLSL sabitleri + shader derleme testi), `bytes.rs` (`Plain`, `as_bytes`), `device.rs` (`create`, `run_once`, `Drop`, aygıt seçimi), `swapchain.rs`, `memory.rs`, `sections.rs`, `pipeline.rs`, `scene.rs`, `texture.rs`, `draws.rs` (`set_*`), `frame.rs`. Taşıma mekanik: her kod satırı yerinde (çok kümeli karşılaştırma; fark yalnız `pub(super)` ve iki imzanın biçimi). Birden çok `impl VulkanBackend` bloğu alt modüllerde; alan görünürlüğü değişmedi, genel API değişmedi.
- RAII: `SkyBuffer` → `HostBuffer` (kendini bırakır, `Drop`), `CloudBuffer` onu içerir; dört tekrarlı "büyüt, eşle, kopyala" gövdesi (`fill_bytes`, hava, bulut, gökyüzü) `HostBuffer::new/write` ve `grow_host_buffer`e indi; `VulkanBackend::drop` host tamponlarını aygıt yok edilmeden önce bırakır.
- Bench (V00 ile aynı komut, 2 koşu): mesh p50 10.41 (temel 10.43), backend 4.78 (4.63, +%3.2), fence 7.89 (7.92), GPU sahne 7.50 (7.49), kare p95 18.97 (19.67). Fark gürültü içinde. Bir `shot` karesi okundu: gökyüzü, bulut, arazi, bayrak, HUD, el doğru.
- Sapma: iki kare piksel piksel karşılaştırılmadı (canlı dünya her karede değişiyor: entity, bulut); doğrulama kare okuma ve bench ile.
- **Kalan → V01b:** `Image`/`Pipeline` sahip tipleri, `ArenaBlock`/`StagingChunk` null işaretçileri, `create`'in aşamalara bölünmesi, `set_*` → `DrawList`, `record_scene`'in bölünmesi.
