# P15.a.1 — Kontrollü Java istemci koşusu — [altyapı, parity iddiası yok]

**Tarih:** 2026-09-15 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Bu kayıt eşitlik kanıtı **değildir**. Render bölümünün referans yarısının ilk
adımıdır: pinned Java istemcisini sabit bir yapılandırmayla, seçilen grafik
backend'inde, tekrarlanabilir biçimde çalıştırıp penceresini yakalamak.
Karşılaştırma yapılmaz.

## Çalıştırıcı

```text
python3 tools/render/reference.py launch --backend opengl|vulkan \
    [--width 854] [--height 480] [--captures N] [--interval S] [--settle S]
```

Her koşu `options.txt`'i baştan yazar, böylece istemci önceki oturumun
ayarlarını devralmaz. Sabitlenen 29 ayar iki gruba ayrılır: çizimi etkileyenler
(GUI ölçeği, FOV, render/simulation distance, graphics mode, ambient occlusion,
partiküller, mipmap, gamma, view bob, ekran efekt ölçekleri, glint hızı/gücü,
vsync, maxFps, dil) ve yalnızca ortamı sessizleştirenler (narrator, ses, altyazı,
odak kaybında duraklatma, öğretici adımı). İkinci grup çizilene karışmaz ama
koşunun kendiliğinden ilerlemesini engeller.

Çıktılar `.data/render/runs/<backend>-<zaman>/` altında kalır: istemci logu ve
yakalanan kareler. İzlenen dosya yalnızca çalıştırıcının kendisidir.

## Doğrulanan

Her iki backend de yerelde ayağa kalkıyor ve istemci kendi bildirdiği sürücüyü
logluyor:

| Backend | İstemcinin bildirdiği |
| --- | --- |
| `opengl` | `OpenGL`, `4.6 (Core Profile) Mesa 26.2.2-arch3.2` |
| `vulkan` | `Vulkan`, `1.4.354 radv Mesa 26.2.2-arch3.2` |

`--graphicsBackend` argümanı oyun içi tercihi geçersiz kılıyor ve bunu logda
açıkça bildiriyor; P15.a.3'teki OpenGL↔Vulkan karşılaştırması bu tutamağa
dayanacak.

Yakalama yolu **taze**: aynı koşuda dört saniye arayla alınan kareler
birbirinden %61–77 oranında farklı çıkıyor, yani `import -window` bayat bir
tampon döndürmüyor. Bu ölçüm aynı zamanda başlık ekranının **sabit sahne
olmadığını** gösteriyor; panorama sürekli döndüğü için referans karesi olarak
kullanılamaz.

Yerleşme süresi kısa tutulduğunda (5 saniye) ardışık iki kare bit-bit aynı
çıktı: istemci o anda henüz panoramayı sürmüyordu. Yakalamadan önce yeterli
yerleşme beklenmezse ölçüm yanıltıcı olur; çalıştırıcıda `--settle` bu yüzden
ayrı bir parametredir.

## Ortam bağımlılığı

İstemci burada XWayland altında çalışıyor; bu yüzden penceresi X11'e görünür ve
hem tek pencere yakalaması hem de tuş enjeksiyonu mümkün. Native Wayland
oturumunda bu yol geçerli değildir ve farklı bir yakalama mekanizması gerekir.
Çalıştırıcı `xdotool` veya ImageMagick `import` yoksa `blocked` döner; eksik
ortam başarı sayılmaz.

Narrator native'i (`libflite.so`) sistemde yok. İstemci bunu hata olarak
loglayıp devam ediyor, koşuyu düşürmüyor; çizime etkisi yok.

## Açık kalan — P15.a'nın asıl ölçümü

Bu dilim yalnızca koşuyu tekrarlanabilir hale getirir. Ölçüm hâlâ yapılmadı:

1. **Sabit sahne.** Sabit seed'li önceden üretilmiş bir dünya, sabit oyuncu
   konumu ve bakışı, sabitlenmiş saat ve hava, hareketsiz entity'ler. Başlık
   ekranı bunu karşılamıyor. `--quickPlaySingleplayer` bu sahneye menüsüz
   girmeyi sağlayacak tutamaktır.
2. **Java'nın kendi değişkenliği.** Aynı sabit sahnede tekrarlanan koşular
   arasındaki fark. Bu sayı, her render karşılaştırmasının tolerans tabanıdır.
3. **OpenGL↔Vulkan farkı.** Aynı sabit sahnenin iki backend'deki çıktısı.
   Sıfırsa [backend seçimimiz](render-backend-decision.md) gözlemsel olarak
   bedelsizdir; değilse hangi backend'e karşı eşitlik iddia edildiği her render
   kaydında yazılır.

Bu üçü tamamlanmadan hiçbir render dilimi `client-parity` sayılamaz.
