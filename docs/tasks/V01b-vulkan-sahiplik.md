# V01b — Vulkan nesnelerinde sahiplik ve `create`'in bölünmesi

**Amaç:** V01'in bölünen kalanı: kalan Vulkan nesnelerini sahibi olan tiplere almak, `create`'i aşamalara ayırmak, tekrar eden `set_*` fonksiyonlarını tek yapıya indirmek.
**Bağımlılık:** V01. **Boy:** M.

## Oku
- [V01 devir notu](V01-vulkan-modulleri.md)
- `crates/photon-render/src/vulkan/{mod,device,scene,texture,draws}.rs`

## Yapılacaklar
- `Image` (görüntü + bellek + görünüm) ve `Pipeline` (boru hattı + yerleşim) sahip tipleri; `SceneResources`'taki elle `destroy_*` çağrıları ve `destroy_scene` bunlara iner. Yok etme sırası tipte belgeli; `VulkanBackend::drop` aygıttan önce hepsini bırakır (`HostBuffer` örneği).
- `ArenaBlock`/`StagingChunk` için `null` işaretçili boşluk yerine `Option`; `upload_texture`'ın hazırlama tamponu bir sahip tip (hata yolunda sızıntı olmasın).
- `create` (≈325 satır) `instance`, `device`, `swapchain`, `sync` aşama fonksiyonlarına bölünür; `device.rs` yalnız bunları çağırır.
- (`set_*` → `DrawList` maddesi V03b'ye taşındı.)
- `frame.rs` (≈1200 satır; `record_scene` 550) çizim türü başına işlevlere ayrılır (V03 ile birlikte yapılabilir).

## Kabul
`python3 tools/photon.py check` → `CHECK OK`; `bench` sayıları V01 devir notundaki aralıkta; bir `shot` karesi doğru.

## Kapsam dışı
Senkronizasyon değişikliği (V03).

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
- Yapıldı: `vulkan/owned.rs` — `Image` (görüntü+bellek+görünüm), `Sampler`, `Texture` (görüntü+örnekleyici), `Pipeline` (boru hattı+yerleşim), `Descriptors` (havuz+yerleşim), `BoundBuffer` (tampon+bellek); hepsi `Drop` ile kendini boşaltır. `SceneResources` 20 elle yok edilen boru hattı çiftini ve görüntü üçlülerini bunlara indirdi; `destroy_scene` aygıtı bekler ve `scene = None` yapar.
- `ArenaBlock`/`StagingChunk` null işaretçi yerine `Vec<Option<_>>` yuvası; hazırlama ve okuma tamponları (`upload_buffer`, `readback`) `BoundBuffer`: hata yolunda sızıntı yok (`bound_buffer` bir adım başarısız olursa kendini boşaltır; `create_pipeline`, `upload_texture`, `Descriptors::create` da öyle).
- `create` dört aşamaya bölündü: `device/{instance,logical,presentation,sync}.rs` (`InstanceParts`, `DeviceParts`, `PresentationParts`, `SyncParts`); `create.rs` yalnız bunları çağırıp `VulkanBackend`'i kurar. Sınır: aşama yapıları `Drop`suz; ikinci aşamadan sonra başarısız olan bir başlatma (aygıt/swapchain) örnek/aygıtı serbest bırakmaz, süreç zaten çıkar.
- Doğrulama: `bench` (PHOTON_VALIDATION=1) 0 hata 0 uyarı; kare p50 8.68 ms (V03: 8.37, fark %3.7, aynı aralık); `shot` kare doğru (arazi, bayraklar, bulutlar, envanter).
- Açık: `frame/scene.rs` `record_scene` çizim türü başına işlevlere ayrılması V03c'de (tablo ile birlikte).
