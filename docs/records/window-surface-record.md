# P05.d.2 — GLFW pencere yüzeyi — [altyapı, parity iddiası yok]

**Tarih:** 2026-09-15 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Bu kayıt bir eşitlik kanıtı **değildir**. Pencere yüzeyinin kurulmasını, hangi
kararların kaynaktan türetildiğini ve hangilerinin henüz doğrulanmadığını
kaydeder. Pinlenen saf davranış
[P05.d.1 kaydındadır](window-gui-scale-conformance.md).

## Kaynaktan türetilen kurulum sırası

`Window` kurucusu şu sırayı izler: `DisplayData` boyutu `allowedWindowMinSize`
ile kelepçelenir, pencere **gizli** oluşturulur (`GLFW_VISIBLE=false`), birincil
monitörün tercih edilen video moduna göre ortalanır, `setMode` çağrılır,
ardından framebuffer boyutu okunup GUI ölçeği ondan türetilir. Geri çağrılar
(framebuffer resize, move, resize, focus, cursor enter, iconify) en son
bağlanır.

Backend pencere hint'leri `GpuBackend.setWindowHints()` ile verilir. Vulkan
yolunda tek hint `GLFW_CLIENT_API = GLFW_NO_API`'dir: yüzey GLFW tarafından
değil Vulkan tarafından oluşturulur. OpenGL yolu ayrıca 3.3 core profile,
forward-compat ve native context API hint'lerini verir.

Referans varsayılan pencere boyutu `854x480`'dir (`Main` argüman çözümleyicisi).

## Bu dilimde kurulan

`voxel_client::window::surface` GLFW'yi başlatır, Vulkan hint'ini ve gizli
oluşturmayı uygular, boyutu kelepçeler, birincil monitöre referans formülüyle
ortalar, framebuffer boyutunu okuyup [P05.d.1'de pinlenen](window-gui-scale-conformance.md)
ölçek kurallarını uygular ve pencereyi gösterir. Framebuffer resize olayında
ölçek yeniden türetilir.

Çalıştırma:

```text
cargo run -- window [genişlik] [yükseklik] [olay-turu]
```

`854x480` girdisinde gözlenen çıktı: `framebuffer_size=854x480`,
`gui_scale=2`, `gui_scaled_size=427x240`. Bu değerler P05.d.1'de Java ile
birebir eşleşen kurallardan üretilmiştir.

## Doğrulanmamış olanlar

- **Pencere hâlâ piksel göstermiyor.** `GLFW_NO_API` ile oluşturulan yüzeye
  hiçbir kare sunulmadığı için Wayland compositor'ü yüzeyi map etmez; GLFW
  kendi `visible` bayrağını `true` bildirse de ekranda bir şey görünmez. Görünür
  çıktı Vulkan swapchain dilimini (P15.b) gerektirir ve o dilimden önce
  "pencere çalışıyor" görsel olarak gösterilemez. **Bu madde
  [P15.b kaydıyla](vulkan-backend-record.md) kapandı**: ilk kare sunulduğunda
  pencere göründü.
- **Monitör ortalama, `setMode` ve fullscreen geçişleri** yalnızca kaynaktan
  türetildi; çalışan Java penceresiyle karşılaştırılmadı. `refreshFramebufferSize`
  kelepçesi de bu durumdadır.
- **Olay semantiği** (odak sırası, tuş tekrarı, resize/move sıralaması)
  karşılaştırılmadı.

## Bilinen sürüm farkı — açık risk

Referans LWJGL 3.4.1 ile **GLFW 3.5.0** dağıtır. Rust tarafında kullanılan
`glfw` crate'i `glfw-sys` üzerinden **GLFW 3.4.0** kaynağını derler. Sistemdeki
GLFW ise 3.5.1'dir. Vendor'lı 3.4.0 makineler arası deterministik olduğu için
sistem kütüphanesine linklemeye tercih edildi, ancak **sürüm farkı kapanmadan
hiçbir pencere/olay davranışı `client-parity` sayılamaz**. Boyut ve framebuffer
davranışı 3.4 ile 3.5 arasında kararlıdır; odak, tuş tekrarı ve resize sıralaması
değildir. P05.d girdi/olay dilimlerinden önce GLFW 3.5.0 pinlenmelidir.

## Bağımlılık

Bu dilimle `glfw` eklendi; gerekçesi
[render backend kararındadır](../render-backend-decision.md). Özellik seti
`raw-window-handle-v0-6, vulkan, wayland, x11, src-build` ile sınırlandı;
`image` ve prebuilt indirme dışarıda bırakıldı, kaynak derlemesi
tekrarlanabilirlik için seçildi.
