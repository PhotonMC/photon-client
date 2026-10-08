# P01.b — Vec3 koordinat yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `Vec3` sınıfının `Vec3i`/`BlockPos` konumlarından alt köşe, merkez, taban merkezi ve y kaydırılmış taban merkezi üretmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:45-63` içindeki beş statik yardımcı ve `core/Vec3i.java:65-75` içindeki x/y/z getter'ları incelendi. Oracle doğrudan Java `Vec3` çağrılarını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `atLowerCornerOf`, konumun üç tamsayı bileşenini double olarak Vec3 x/y/z bileşenlerine taşır.
- `atLowerCornerWithOffset`, aynı alt köşe bileşenlerine verilen x/y/z double offset'lerini kaynak sırasıyla ekler.
- `atCenterOf`, üç eksene `0.5` ekleyen offset yolunu kullanır.
- `atBottomCenterOf`, x ve z'ye `0.5`, y'ye `0.0` ekleyen offset yolunu kullanır.
- `upFromBottomCenterOf`, x ve z'ye `0.5` ekler ve y offset'ini çağrıdan alır.
- Sonuç Vec3 x, y, z, length ve length squared alanlarının f64 ham bitleriyle karşılaştırılır; tolerans uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `lower-corner` | i32 sınırlarına yakın BlockPos bileşenleri | Alt köşe ve int-to-double dönüşümü |
| `lower-corner-offset` | Negatif konum ve üç sonlu f64 offset | Offset bileşenlerinin eklenmesi |
| `center` | Negatif ve farklı eksenlerde konum | Üç sabit yarım blok kayması |
| `bottom-center` | Sıfır konum | Taban merkezi ve y pozitif sıfırı |
| `up-from-bottom-center` | Negatif y konumu ve negatif y offset'i | Y kaydırmasının iletilmesi |
| `offset-special-values` | Signed zero, subnormal ve NaN offset'leri | Özel f64 değerlerinin toplama sonucuna taşınması |

Girdiler `family: "math"`, `operation: "vec3_coordinates"`, decimal string olarak i32 `x`, `y`, `z` ve `kind` alanlarından oluşur. Offset alanları 16 haneli küçük harfli f64 bit dizgeleridir.

## Rust karşılığı

`crates/photon-core/src/math.rs` içindeki `Vec3` statik yardımcıları `BlockPos` bileşenlerini f64'e çevirerek Java'daki üretim yollarını korur. `examples/behavior_adapter.rs` sonucu Vec3 f64 ham bitleriyle taşır; `tests/behavior/java/MathOracle.java` aynı Java yardımcılarını çağırır.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec3` sınıfının BlockPos tabanlı koordinat yardımcılarını karşılar. Java oracle, `atLowerCornerOf`, `atLowerCornerWithOffset`, `atCenterOf`, `atBottomCenterOf` ve `upFromBottomCenterOf` çağrılarını doğrudan çalıştırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Altı yeni senaryoda alt köşe, üç bileşenli offset, merkez, taban merkezi, y offset'i ve signed zero/subnormal/NaN offset sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik BlockPos girdisinde beş Java üreticisinin Vec3 x, y, z, length ve length squared f64 ham bit snapshot'larıyla sınırlıdır. `Vec3` constructor overload'ları ve daha geniş BlockPos/SectionPos davranışları ayrı dilimlerdir.
