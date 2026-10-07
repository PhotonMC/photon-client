# V01b — Vulkan nesnelerinde sahiplik ve `create`'in bölünmesi

**Amaç:** V01'in bölünen kalanı: kalan Vulkan nesnelerini sahibi olan tiplere almak, `create`'i aşamalara ayırmak, tekrar eden `set_*` fonksiyonlarını tek yapıya indirmek.
**Bağımlılık:** V01. **Boy:** M.

## Oku
- [V01 devir notu](V01-vulkan-modulleri.md)
- `crates/voxel-render/src/vulkan/{mod,device,scene,texture,draws}.rs`

## Yapılacaklar
- `Image` (görüntü + bellek + görünüm) ve `Pipeline` (boru hattı + yerleşim) sahip tipleri; `SceneResources`'taki elle `destroy_*` çağrıları ve `destroy_scene` bunlara iner. Yok etme sırası tipte belgeli; `VulkanBackend::drop` aygıttan önce hepsini bırakır (`HostBuffer` örneği).
- `ArenaBlock`/`StagingChunk` için `null` işaretçili boşluk yerine `Option`; `upload_texture`'ın hazırlama tamponu bir sahip tip (hata yolunda sızıntı olmasın).
- `create` (≈325 satır) `instance`, `device`, `swapchain`, `sync` aşama fonksiyonlarına bölünür; `device.rs` yalnız bunları çağırır.
- `set_entities`/`set_shadows`/… çiftleri (`*_buffer` + `*_count`) tek `DrawList` değerine ve `set_draw(kind, vertices)`ye iner. V03'ün `RenderType` tablosuyla çakışmamak için önce V03'ün tasarımına bakılır; çakışıyorsa bu madde V03'e taşınır.
- `frame.rs` (≈1200 satır; `record_scene` 550) çizim türü başına işlevlere ayrılır (V03 ile birlikte yapılabilir).

## Kabul
`python3 tools/voxel.py check` → `CHECK OK`; `bench` sayıları V01 devir notundaki aralıkta; bir `shot` karesi doğru.

## Kapsam dışı
Senkronizasyon değişikliği (V03).

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
(boş)
