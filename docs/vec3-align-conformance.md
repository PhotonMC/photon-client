# P01.b — Vec3 align sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.align(EnumSet<Direction.Axis>)` metodunu karşılar. Rust adaptörü JSON eksen listesini Axis değerlerine çözer ve seçilen bileşenlerde Java floor/cast yolunu kullanır; Java oracle doğrudan `Vec3.align` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Altı yeni senaryoda boş, kısmi, tüm, tekrarlı ve sıralı eksen kümeleri; negatif kesir, signed zero, subnormal, Infinity ve NaN double-to-int sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.align(EnumSet<Direction.Axis>)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth.floor` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
