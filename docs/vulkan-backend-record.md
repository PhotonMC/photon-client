# P15.b — Vulkan backend ve swapchain — [altyapı, parity iddiası yok]

**Tarih:** 2026-09-15 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Bu kayıt eşitlik kanıtı **değildir**. Backend'in ayağa kalkmasını, hangi
seçimlerin referanstan alındığını ve hangilerinin henüz karşılaştırılmadığını
kaydeder. Backend tercihinin gerekçesi
[ayrı karar kaydındadır](render-backend-decision.md).

## Referanstan alınan seçimler

Bunlar hangi GPU'nun ve hangi görüntünün seçildiğini belirlediği için kaynaktan
birebir alındı:

| Seçim | Referansta | Bizde |
| --- | --- | --- |
| Instance API sürümü | `VK_API_VERSION_1_2` | aynı |
| Uygulama/engine adı | `Minecraft Java Edition` / `MinecraftJE`, engine sürümü `0` | aynı |
| Instance uzantıları | GLFW'nin bildirdiği yüzey uzantıları | aynı |
| Minimum cihaz API | feature sorgusu `1.1` altını eler | aynı |
| Zorunlu cihaz uzantıları | `dynamic_rendering`, `push_descriptor`, `synchronization2`, `vertex_attribute_divisor`, `swapchain` | aynı |
| Cihaz seçimi | ilk uygun cihaz; sonradan discrete çıkarsa ona yükselt | aynı |
| Yüzey formatı | `colorSpace == SRGB_NONLINEAR` ve format `R8G8B8A8_UNORM` ya da `B8G8R8A8_UNORM` | aynı |
| Swapchain görüntü sayısı | `max(3, capabilities.minImageCount)` | aynı |
| Swapchain | array layers `1`, sharing `EXCLUSIVE`, preTransform `currentTransform`, compositeAlpha `OPAQUE`, clipped | aynı |

Etkinleştirilen cihaz özellikleri referansın `REQUIRED_DEVICE_FEATURES`
listesini izler: `multiDrawIndirect`, `fillModeNonSolid`, `samplerAnisotropy`,
`shaderDrawParameters`, `timelineSemaphore`, `hostQueryReset`,
`synchronization2`, `dynamicRendering`, `vertexAttributeInstanceRateDivisor`.

## Bu dilimde kurulan

Instance → yüzey → fiziksel cihaz → mantıksal cihaz ve kuyruk → swapchain →
komut havuzu → senkronizasyon. Her kare bir görüntü alır, tek renge temizler,
sunar; isteğe bağlı olarak sunulan görüntüyü host'a kopyalayıp RGBA8 olarak geri
okur.

```text
cargo run -- window 854 480
VOXEL_CAPTURE_PPM=/yol/frame.ppm cargo run -- window 854 480 1
```

Yerel koşuda gözlenen: `AMD Radeon RX 9060 XT (RADV GFX1200)`, discrete `true`,
cihaz API `1.4.354`, format `B8G8R8A8_UNORM`, `3` swapchain görüntüsü, extent
`854x480`. Geri okunan karenin her pikseli istenen temizleme rengini taşıyor.

Pencere ancak bir kare sunulduktan sonra ekranda görünür hale geldi; Wayland
compositor'ü buffer commit edilmeden yüzeyi map etmiyor. Bu, P05.d.2 kaydında
açık bırakılan maddeyi kapatır.

## Doğrulanmamış olanlar

- **Temizleme rengi referans değeri değildir.** Hiçbir Java karesi yakalanmadı
  veya karşılaştırılmadı; renk yer tutucudur ve piksel hakkında hiçbir eşitlik
  iddiası taşımaz.
- **Sunum modu** `FIFO` sabitlendi. Referans modu ayarlardan gelen
  `GpuSurface.PresentMode` ile seçer; ayar bağlantısı kurulmadı.
- **VMA yok.** Referans belleği Vulkan Memory Allocator ile yönetir; buradaki
  tek geri okuma tamponu doğrudan ayrılıyor.
- **Bariyerler kabadır.** Geçişler `ALL_COMMANDS` ile tüm yazmaları görünür
  kılar. Gerçek pipeline geldiğinde her geçiş kendi stage/access çiftini alır.
- **Validation layer, debug etiketleri, checkpoint uzantıları ve shader hattı**
  (shaderc + SPIRV-Cross) kurulmadı.
- **Swapchain yeniden oluşturma** yok; pencere yeniden boyutlandığında extent
  sabit kalır.

## Kare geri okuma ve P15.a

Geri okuma, sunulan görüntüyü kopyalar; yeniden çizim yapmaz. Bu yol
[P15.a ölçümünün](render-backend-decision.md) ihtiyaç duyduğu yoldur: Java'nın
kendi tekrarları arasındaki değişkenlik ve OpenGL ile Vulkan arasındaki fark
ölçülene kadar hiçbir render dilimi `client-parity` sayılmaz. Çıktı şimdilik
ikili PPM; kalıcı yakalama biçimi P15.a ile sabitlenecek.
