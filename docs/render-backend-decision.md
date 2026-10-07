# P15 — Render yığını kararı: GLFW + Vulkan 1.2 (ash), wgpu değil

**Tarih:** 2026-09-15 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Bu kayıt bir davranış dilimi değildir; [AGENTS.md](../AGENTS.md)'nin "teknolojiyi
ihtiyaç ve kaynak incelemesi ortaya çıkmadan seçme" kuralı gereği, render
bölümüne girmeden önce yapılan kaynak incelemesinin sonucudur.

## Referansın gerçekte kullandığı yığın

Pinned 26.2 istemcisi tek bir grafik yolu taşımıyor. `PreferredGraphicsApi`
enum'u `DEFAULT`, `OPENGL` ve `VULKAN` değerlerini tanımlar ve
`getBackendsToTry()` denenecek backend dizisini döndürür: `VULKAN` seçiliyse
Vulkan önce, aksi halde OpenGL önce. `Minecraft` açılışta `DEFAULT` değerini
`OPENGL`'e normalize eder. Yani **varsayılan OpenGL, Vulkan ise üretim
alternatifidir** — deneysel işaret, uyarı veya kapalı bayrak yoktur.

Vulkan yolunun bileşenleri:

| Katman | Referansta |
| --- | --- |
| Pencere/olay | GLFW (LWJGL `lwjgl-glfw`) |
| Grafik API | Vulkan **1.2** (`VK_API_VERSION_1_2`; cihaz 1.1 altındaysa eleniyor) |
| Bellek | Vulkan Memory Allocator (`lwjgl-vma`) |
| Shader derleme | **shaderc** ile GLSL → SPIR-V; hedef ortam Vulkan 1.2, `auto_bind_uniforms` ve `auto_map_locations` açık |
| Shader yansıması | SPIRV-Cross (`lwjgl-spvc`) |
| macOS | MoltenVK; `VK_KHR_portability_enumeration` ve `VK_KHR_portability_subset`, ayrıca integrated Intel MoltenVK için ayrı tampon yolu |

İstemcinin dağıttığı native'ler `glfw, vulkan, vma, shaderc, spvc, opengl,
openal, freetype, stb, tinyfd, jemalloc` modüllerini linux, macOS (x64 + arm64),
Windows (x64 + arm64 + x86) için içerir. Vulkan yolu referansta gerçekten
cross-platform'dur.

## Karar

Rust tarafında **GLFW + ham Vulkan 1.2 (`ash`) + `shaderc` + SPIRV-Cross**
hedeflenir. Yani referansın Vulkan backend'inin bire bir karşılığı kurulur:
aynı pencere kütüphanesi, aynı API sürümü, aynı shader derleyicisi ve aynı
derleme seçenekleri.

### wgpu neden seçilmedi

wgpu "en güncel ve cross-platform" tanımına uyar, ancak bu projede eşitliği
bozar:

- **Shader yolu değişir.** Referans GLSL'i shaderc ile SPIR-V'ye derler. wgpu
  kendi shader hattını (naga) kullanır; aynı GLSL'den farklı SPIR-V, farklı
  optimizasyon ve farklı kayan nokta davranışı çıkabilir. Karşılaştırdığımız şey
  referansın çıktısı değil, naga'nın yorumu olur.
- **Pipeline ve state soyutlanır.** wgpu, backend'ler arası davranışı normalize
  eder; blaze3d'nin kurduğu pipeline/state'i birebir kuramayız, sadece benzerini
  kurabiliriz.
- **Fark kaynağı gizlenir.** Bir piksel farkı çıktığında "referans mı, bizim
  kodumuz mu, wgpu katmanı mı" ayrımı yapılamaz. `ash` ile Vulkan çağrıları
  referanstaki çağrılarla karşılaştırılabilir kalır.

Aynı gerekçe pencere katmanı için de geçerlidir: `winit` yerine `glfw` seçilir,
çünkü referans GLFW'nin kendisini kullanır ve odak, resize, tuş tekrarı ve olay
sırası semantiği kütüphaneye özgüdür.

### OpenGL yolu kapsam dışı değildir

Referansın varsayılanı OpenGL olduğundan, pinned referans yapılandırması
açıkça yazılmalıdır: karşılaştırma `preferredGraphicsBackend=vulkan` ile
çalışan Java istemcisine karşı yapılır. Bu meşru bir referans yapılandırmasıdır,
ancak varsayılan değildir ve [referans envanterine](reference-baseline.md)
kaydedilmeden eşitlik iddiası kurulamaz. Ölçülen `22` piksellik fark bu şartın
gerekçesidir: yapılandırma yazılmazsa o pikseller sessiz bir eşitlik açığı
olarak kalır.

## Karar öncesi ölçülmesi gereken

Bu kayıt backend seçimini sabitler; **eşitlik toleransını sabitlemez**. P15.a
ilk dilimdir ve iki şeyi ölçer:

1. Aynı Java yapılandırmasının kendini tekrarındaki frame değişkenliği (aynı
   makine, aynı sürücü, sabit sahne).
2. Java-OpenGL ile Java-Vulkan arasındaki fark.

(2) sıfırsa backend seçimi gözlemsel olarak bedelsizdir. Sıfır değilse hangi
backend'e karşı eşitlik iddia ettiğimiz her render kaydında açıkça yazılır.
Bu ölçüm yapılmadan hiçbir render dilimi `client-parity` sayılmaz.

**Ölçümün ilk sonucu geldi.** [P15.a.2 kaydı](render-variability-measurement.md)
sabit bir sahnede iki backend'i karşılaştırdı: ortalama kanal farkı bir
seviyenin altında, ancak `delta > 8` olan **22 piksel** var ve bunlar backend
başına deterministik (aynı backend'in iki koşusunda `0`). Sıyırma açısında
texel sınırı yuvarlamasından geliyorlar. Yani (2) sıfır değildir: Vulkan
hedeflemek, bu sahnede Java'nın varsayılan OpenGL yapılandırmasından kalıcı
olarak `22` pikselde ayrılmak demektir. Aşağıdaki "pinned referans
yapılandırması" şartı bu yüzden zorunludur.

## Bağımlılık eklenmesi

Hiçbir GPU veya pencere bağımlılığı bu kayıtla birlikte eklenmez. İlk render
dilimi ([P05.d.1 window ölçek matematiği](records/window-gui-scale-conformance.md))
GPU gerektirmez ve saf tamsayı/çift duyarlık davranışıdır. `glfw`, `ash`,
`shaderc` bağımlılıkları, onları gerçekten kullanan ilk dilimle birlikte ve
kendi gerekçesiyle eklenir.
