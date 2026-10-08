# P16.k — Model çözümü ve gerçek blok çizimi — [karma: pinlenmiş parçalar, pinlenmemiş birleştirme]

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Bu kayıt tek bir kanıt sınıfı taşımaz ve bu bilerek böyledir. Birleştirdiği
parçaların her biri ayrı ayrı `client-parity` olarak pinlenmiştir; onları bir
araya getiren zincir yürüme ise pinlenmemiştir.

## Pinlenmiş olan

| Adım | Kayıt |
| --- | --- |
| Model dosyasını ayrıştırma | [P16.j](model-file-parse-conformance.md) |
| JSON ve sayı dönüşümü | [P16.d](json-parse-conformance.md) |
| Texture slot çözümü | [P16.c](model-texture-slots-conformance.md) |
| Atlas paketleme | [P16.a](atlas-stitcher-conformance.md) |
| PNG çözme | [P16.b](png-decode-conformance.md) |
| Yüz köşeleri ve varsayılan UV | [P16.e](face-geometry-conformance.md) |
| Element döndürmesi | [P16.g](element-rotation-conformance.md) |
| Yüz UV döndürmesi | [P16.i](face-uv-rotation-conformance.md) |
| Varyant döndürmesi | [P16.l](model-state-conformance.md) |

## Pinlenmemiş olan

Zincir yürüme üç kuraldan ibarettir ve üçü de kaynaktan okunmuştur:
geometri, ambient occlusion ve GUI ışığı için çocuktan köke doğru ilk ayarlı
değer alınır; bulunamazsa sırasıyla boş geometri, `true` ve `SIDE` kullanılır.
Texture haritaları çocuktan köke toplanıp pinlenmiş çözücüye verilir.

Bu kurallar ayrıca pinlenmedi çünkü referansta `ResolvedModel` örneği üretmek
kaynak yöneticisi altyapısını gerektirir. Kurallar tek satırlık ve okunarak
denetlenebilir; yine de **doğrulanmamıştır** ve öyle işaretlenir.

Çizimin kendisi de pinlenmemiştir: shader'lar, kamera ve yüz başına sabit
gölge bu projenin kendi uydurmasıdır. Ayrıntılar
[dokulu küp kaydındadır](textured-cube-record.md).

## Çalıştırma

```text
PHOTON_ASSETS=/kullanıcının/asset/klasörü \
PHOTON_MODEL=minecraft:block/oak_stairs \
PHOTON_ROTATE_Y=90 \
cargo run -- window 854 480
```

Asset kökü kullanıcıdan gelir; kurulum hiçbir orijinal içerik taşımaz. Kaynak
soyutlaması yol kurmayı kendi üstlenir: çözüm bir `parent` alanında ne yazıyorsa
onu ister, dosya yolunu kaynak bilir.

Yerel koşuda gözlenenler:

| Model | Element | Sprite | Atlas |
| --- | ---: | ---: | --- |
| `block/oak_stairs` | `2` | `1` | `32x32` |
| `block/oak_log` | `1` | `2` | `64x32` |
| `block/crafting_table` | `1` | `4` | `64x64` |
| `block/lantern` | `4` | `1` | `32x64` |

`oak_stairs` parent zincirini `block/stairs`'e kadar yürür; `crafting_table`
dört ayrı dokuyu tek atlasa paketler; `lantern` döndürülmüş elementler taşır.

## Bilinen eksikler

- **`cullface` uygulanmıyor.** Ayrıştırılıyor ve saklanıyor, fakat çizimde
  kullanılmıyor, yani komşusu olmayan bir blokta bile iç yüzler çiziliyor.
- **Blockstate varyant seçimi yok.** Model adı ve döndürme doğrudan veriliyor;
  gerçek istemci `blockstates/` dosyasından blok durumuna göre varyant seçer.
  Seçilen varyantın döndürmesi [pinlendi](model-state-conformance.md) ve
  uygulanıyor; `PHOTON_ROTATE_X` ve `PHOTON_ROTATE_Y` ile sürülebilir.
- **UV kilidi yok.**
- **`display` dönüşümleri ayrıştırılmıyor.**
- **Tint uygulanmıyor.** `tintindex` okunuyor ama renk çarpanı yok, bu yüzden
  çim ve yaprak gibi bloklar gri çizilir.
- Zincir derinliği `64` ile sınırlandırılmıştır; referansın kendi sınırı ayrıca
  incelenmelidir.
