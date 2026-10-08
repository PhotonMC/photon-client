# P16.b — PNG çözme ve RGBA8'e indirgeme — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik PNG baytları pinned
JAR'daki `NativeImage.read` yoluna ve üretim kütüphanesindeki
`photon::render::png` modülüne verilir, çözülen RGBA8 pikselleri birebir
karşılaştırılır. GPU gerektirmez.

Bu, render veri katmanının ikinci dilimidir; birincisi
[atlas paketleyicidir](atlas-stitcher-conformance.md). Piksel toleransı sorunu
bu dilimi etkilemez: karşılaştırılan şey rasterize edilmiş görüntü değil,
dosyadan çözülen piksel verisidir.

## Java kaynak incelemesi

`NativeImage.read`, baytları `STBImage.stbi_load_from_memory` çağrısına dört
bileşen isteyerek verir. Bunun sonucu, PNG'nin renk tipi ne olursa olsun
RGBA8'dir:

- Gri tonlama renk kanallarına kopyalanır, alfa `255` olur.
- Gri+alfa aynı şekilde kopyalanır, alfa dosyadan gelir.
- RGB'ye opak alfa eklenir.
- Palet indeksleri `PLTE` üzerinden açılır; alfa varsa `tRNS`'ten gelir.
- 16 bitlik örnekler **yüksek baytına** indirilir.
- Bir bayttan küçük örnekler tam aralığa ölçeklenir: 1 bit `×0xFF`, 2 bit
  `×0x55`, 4 bit `×0x11`. Palet indeksi parlaklık değil sayı olduğu için
  ölçeklenmez.

`tRNS` yalnızca palet için değildir: gri tonlamada tek bir örnek değeri, RGB'de
tek bir renk tamamen saydam ilan edilebilir. Anahtar, görüntünün bit derinliği
ne olursa olsun 16 bit big-endian yazılır, bu yüzden karşılaştırma **ölçekleme
öncesi ham örnek** üzerinde yapılmalıdır.

## Bağımlılık eklenmedi

PNG çözücü depoda yazıldı; `flate2` zaten üretim bağımlılığı olduğu için
zlib dışında bir şey gerekmedi. Çözücü genel amaçlı bir PNG okuyucu değil,
stb'nin dört bileşenli sonucunu üreten bir okuyucudur.

## Girdiler

`image/png-decode` 29 parametrik satır taşır. Sentetik PNG'ler test
üreticisinde yazılır, yani girdi izlenebilir ve orijinal içerik taşımaz.
Kapsam: beş satır filtresinin her biri ve karışık filtreli görüntü; renk
tipleri gri, gri+alfa, RGB, RGBA, palet; bit derinlikleri 1, 2, 4, 8, 16;
satır dolgusunun önem kazandığı tek sayılı genişlikler; palet, gri ve RGB için
`tRNS`; tek piksel, dar ve geniş görüntüler.

## Sonuçlar

Java ve Rust her satırda aynı genişlik, yükseklik ve RGBA baytlarını verdi.

Koşu: `python3 tools/behavior/run.py capture --family image` ardından
`compare --family image --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

### Gerçek texture'larla duman testi

Sentetik kapsam doğrulandıktan sonra çözücü, pinned JAR içindeki 71 gerçek
texture'a karşı ImageMagick ile karşılaştırıldı (yerel, izlenmeyen kontrol):
**71 eşleşti, 0 farklı**. Bu kontrol ilk koşusunda bir boşluk yakaladı:
`gui/sprites/container/slot/emerald.png` gri tonlama, 2 bit ve `tRNS`
taşıyordu; çözücü o sırada `tRNS`'i yalnızca palet için uyguluyordu. JAR'daki
3.856 PNG'nin dağılımı bu yolun neden önemli olduğunu gösteriyor:

| Renk tipi | Bit derinliği | Dosya |
| --- | ---: | ---: |
| palet | 8 | 1.775 |
| palet | 4 | 710 |
| RGBA | 8 | 529 |
| gri+alfa | 8 | 396 |
| gri | 8 | 173 |
| RGB | 8 | 133 |
| palet | 2 | 112 |
| palet | 1 | 16 |
| gri | 1 / 2 / 4 | 12 |

Alt-bayt derinlikler gerçek asset'lerin yaklaşık beşte biridir; yalnızca 8 ve
16 bit destekleyen bir çözücü sessizce yetersiz kalırdı.

## Çalıştırıcıda düzeltilen çakışma

`math` ailesi tek bir LWJGL yardımcısını native yüklemeden çağırabilmek için
`org/lwjgl/system/MemoryUtil` stub'ı derliyordu. Derlenen sınıflar classpath'te
jar'lardan önce geldiği için bu stub, LWJGL'i gerçekten kullanan `image`
ailesinin de önüne geçiyordu: iki aile ayrı ayrı geçerken birlikte
`NoSuchMethodError` veriyordu. Çalıştırıcı artık stub'ı yalnızca seçili
ailelerin hiçbiri gerçek LWJGL jar'ını getirmediğinde derliyor.

## Açık kapsam

Adam7 interlace desteklenmiyor; pinned JAR'daki 3.856 PNG'nin hiçbiri
interlaced değil, ama harici resource pack'ler olabilir ve çözücü bu durumda
açıkça hata döndürür. Animasyonlu sprite `.mcmeta` verisi, mipmap üretimi,
`MissingTextureAtlasSprite` yolu ve atlas'a yerleştirme bu kaydın dışındadır.
Bozuk veya kötü niyetli PNG girdilerinin davranışı ayrıca sınanmadı.

`image` ailesi gerçek LWJGL native'lerini çalıştırdığı için platforma bağlıdır:
pinned kütüphane listesi `natives-linux` jar'larını adlandırır ve başka bir
platformda `blocked` döner. Diğer ailelerin böyle bir bağı yoktur.
