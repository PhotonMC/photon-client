# P01.b — Vec3 lerp sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.lerp(Vec3, double)` metodunun üç bileşeni için `Mth.lerp(double, double, double)` çağrılarını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:228-230` içindeki metod ve `util/Mth.java:554-560` içindeki Vec3/scalar lerp overload'ları incelendi. Oracle doğrudan Java çağrılarını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Her bileşen f64 hassasiyetinde `p0 + alpha * (p1 - p0)` sırasıyla hesaplanır.
- `Vec3.lerp` x, y ve z bileşenleri için aynı alpha'yı sırayla uygular; alpha 0–1 aralığına clamp edilmez.
- Negatif ve 1.0'dan büyük alpha değerleri extrapolation üretir; signed zero, subnormal, Infinity ve NaN ara sonuçları değiştirilmeden taşınır.
- Başlangıç, hedef, alpha ve sonuç Vec3 snapshot'ları toleranssız ham f64 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu Vec3'ler, alpha `0.25` | Üç bileşen ve temel lerp sırası |
| `negative-extrapolation` | Sonlu Vec3'ler, alpha `-0.5` | Clamp edilmemiş negatif extrapolation |
| `over-one` | Büyük/küçük sonlu bileşenler, alpha `1.5` | 1.0 üstü extrapolation ve double aritmetiği |
| `signed-zero-and-subnormal` | Signed-zero/subnormal bileşenler, alpha `-0.0` | Sıfır işaretleri ve en küçük f64 değerleri |
| `special-values` | Infinity/NaN bileşenleri, alpha `0.5` | Özel değer yayılımı |

Girdiler `family: "math"`, `operation: "vec3_lerp"`, `first`, `second` ve on altı haneli f64 bit string'i `alpha` alanlarını taşır. Sonuç `value` alanında Vec3 snapshot'ıdır; bileşenler, uzunluk ve uzunluk karesi on altı haneli küçük harfli f64 bit string'leridir.

## Rust karşılığı

`crates/voxel-core/src/math.rs` içindeki `Vec3::lerp`, üç bileşende Java'nın çıkarma, çarpma ve toplama sırasını korur. `examples/behavior_adapter.rs` f64 bit girdilerini çözüp snapshot döndürür; `tests/behavior/java/MathOracle.java` aynı `Vec3.lerp` çağrısını doğrudan oracle çıktısına bağlar.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Vec3.lerp(Vec3, double)` metodunu karşılar. Rust adaptörü her eksende `p0 + alpha * (p1 - p0)` hesabını yapar; Java oracle doğrudan `Vec3.lerp` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda temel lerp, negatif ve 1.0 üstü extrapolation, signed zero, subnormal ve özel değer yayılımı karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.lerp` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth` scalar/f32 lerp overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
