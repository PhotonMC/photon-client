# P01.b — Vec3 align sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.align(EnumSet<Direction.Axis>)` metodunun seçilen eksenleri aşağı yuvarlamasını ve diğer bileşenleri korumasını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:281-285` içindeki `align` metodu ile `util/Mth.java:66-68` içindeki double floor yolu incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `axes` EnumSet üyeliği olarak değerlendirilir; eksen sırası ve tekrarları sonucu değiştirmez.
- X, Y veya Z kümede bulunuyorsa ilgili bileşen `Mth.floor(double)` ile i32'e çevrilir ve Vec3 constructor'ına double olarak aktarılır.
- Seçilmeyen bileşenlerin ham f64 bitleri korunur.
- Negatif kesirler, signed zero, subnormal, Infinity ve NaN için Java floor ve narrowing-cast sonucu toleranssız f64 bitleriyle karşılaştırılır.
- Sonuç yeni bir Vec3'tür; başlangıç vektörü değiştirilmez.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `none-preserves` | Boş eksen kümesi ve sonlu bileşenler | Hiçbir bileşeni yuvarlamama |
| `all-floors` | Üç eksen, pozitif ve negatif kesirler | Tüm eksenlerde aşağı yuvarlama |
| `xz-negative` | X/Z eksenleri ve negatif kesirler | Kısmi eksen seçimi |
| `y-signed-zero` | Y seçimi ve signed zero bileşenleri | Floor sonrası sıfır işareti |
| `selected-special` | Üç eksende Infinity ve NaN | Java double-to-int sınır/NaN dönüşümü |
| `reordered-duplicate` | Tekrarlı ve farklı sıradaki X/Z eksenleri | EnumSet üyelik semantiği ve sınır değerleri |

Girdiler `family: "math"`, `operation: "vec3_align"`, `first` Vec3 nesnesi ve `axes` eksen adları dizisinden oluşur. Sonuç `value` içinde Vec3 snapshot'ı olarak döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::align`, eksen slice'ında üyelik arayıp seçilen bileşenlere `floor_f64` uygular; `examples/behavior_adapter.rs` JSON eksen listesini Axis değerlerine çözer. `tests/behavior/java/MathOracle.java` aynı `Vec3.align` çağrısını doğrudan oracle çıktısına bağlar.
