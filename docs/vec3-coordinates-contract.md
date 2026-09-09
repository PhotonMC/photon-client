# P01.b — Vec3 koordinat yardımcıları sözleşmesi

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

`src/math.rs` içindeki `Vec3` statik yardımcıları `BlockPos` bileşenlerini f64'e çevirerek Java'daki üretim yollarını korur. `examples/behavior_adapter.rs` sonucu Vec3 f64 ham bitleriyle taşır; `tests/behavior/java/MathOracle.java` aynı Java yardımcılarını çağırır.
