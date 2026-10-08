# P16.f — Dokulu küp çizimi — [altyapı, parity iddiası yok]

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Bu kayıt eşitlik kanıtı **değildir**. Doğrulanmış veri dilimlerinin ekrana
bağlandığını gösterir ve hangi parçanın kanıtlı, hangisinin uydurma olduğunu
ayırır.

## Kanıtlı olan: çizilen veri

Küpün geometrisi ve doku koordinatları doğrudan pinlenmiş dilimlerden gelir:

| Parça | Kaynak |
| --- | --- |
| Dört köşenin sırası ve konumu | [P16.e yüz geometrisi](face-geometry-conformance.md) |
| Varsayılan UV dikdörtgeni | [P16.e yüz geometrisi](face-geometry-conformance.md) |
| Doku pikselleri | [P16.b PNG çözme](png-decode-conformance.md) |

`full_cube`, element kutusunu tam blok alıp altı yüzün her biri için
`face_vertices` ve `default_face_uv` çağırır. Yani ekrandaki köşe konumları ve
doku eşlemesi Java ile birebir karşılaştırılmış değerlerdir.

## Kanıtlı olmayan: nasıl çizildiği

Aşağıdakilerin hiçbiri referansla karşılaştırılmadı ve bir kısmı referansta
başka türlü yapılıyor:

- **Shader'lar bu projenin kendi GLSL'idir.** Referansın shader'ları değil,
  yalnızca hattın çalıştığını göstermek için yazıldı. Derleme `shaderc` ile
  yapılır; bu, [backend kararında](../render-backend-decision.md) saptandığı gibi
  referansın da kullandığı kütüphanedir.
- **Kamera uydurmadır.** Blok etrafında dönen bir yörünge; istemcinin gerçek
  kamerası P14'tedir.
- **Gölgelendirme uydurmadır.** Yüz başına sabit bir çarpan; referansın ışık,
  ambient occlusion ve biome renklendirmesi yoktur.
- **Atlas kullanılmıyor.** Tek doku tek başına yükleniyor; paketleyici
  [P16.a'da pinlendi](atlas-stitcher-conformance.md) ama bu yola henüz
  bağlanmadı.
- **Model dosyası okunmuyor.** Küp kodda kuruluyor; `blockstates` ve `models`
  dosyalarından zincir kurma ayrı bir dilimdir.
- **Sprite UV paketlemesi yok.** Referans UV'yi sprite'ın atlas konumuna göre
  paketler; burada doku birim kareye eşlenir.

## Çalıştırma

```text
PHOTON_TEXTURE=/yol/oak_planks.png cargo run -- window 854 480
```

Asset yolu kullanıcıdan gelir; kurulum hiçbir orijinal doku içermez. Değişken
verilmezse probe kendi ürettiği damalı dokuyu kullanır ve hangisini seçtiğini
bildirir. `PHOTON_CAPTURE_PPM` verildiğinde sunulan kare geri okunup yazılır.

Yerel koşuda gözlenen: 24 köşe, 36 indeks, `16x16` doku, `854x480` kare.
Gerçek bir blok dokusuyla çizilen küpte on beş benzersiz renk sayıldı — dokunun
paleti çarpı üç yüz gölgesi.

## Vulkan tarafında eklenenler

Derinlik hedefi, doku yükleme ve düzen geçişleri, örnekleyici, descriptor set,
push constant ile view-projection, ve dinamik render ile grafik hattı. Dinamik
render Vulkan 1.2'de çekirdek olmadığı için giriş noktaları KHR yükleyicisinden
alınır; instance 1.2 hedeflediğinden çekirdek adları yüklenmez.

Örnekleyici `NEAREST` filtreler: blok dokuları piksel sanatıdır ve yumuşatma
referanstan ilk görünür sapma olurdu.

## Açık kapsam

Swapchain yeniden oluşturma yok; pencere yeniden boyutlandığında derinlik
hedefi ve hat eski boyutta kalır. Kesici bir eşikle alfa reddi yapılır, gerçek
saydamlık sıralaması yoktur. Bu kaydın hiçbir pikseli
[P15.a.2'deki tolerans engeli](../render-variability-measurement.md) çözülmeden
`client-parity` sayılamaz.
