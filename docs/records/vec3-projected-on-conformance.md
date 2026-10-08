# P01.b — Vec3 projectedOn sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.projectedOn(Vec3)` metodunun bir vektörü `onto` vektörüne izdüşürme ve sıfır uzunluk karesi kısa yolunu içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:323-325` içindeki `projectedOn` metodu ile `lengthSqr`, `dot` ve `scale` çağrılarının kaynak sırası incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Önce `onto.lengthSqr() == 0.0` karşılaştırılır; sonuç doğruysa `onto` vektörü yeni işlem yapılmadan aynen döndürülür.
- Sıfır olmayan durumda işlem sırası `onto.scale(this.dot(onto)).scale(1.0 / onto.lengthSqr())` biçimindedir.
- `lengthSqr()` iki kez çağrılır; her iki çağrı da aynı f64 bileşen karesi toplamını kullanır.
- Signed zero, subnormal underflow, overflow, Infinity ve NaN bitleri toleranssız f64 snapshot'larıyla karşılaştırılır.
- Sonuç yeni bir Vec3'tür; sıfır kısa yolunda dönen `onto` bileşenleri de değiştirilmez.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu vektörün x eksenine izdüşümü | Dot ve iki scale yolunun temel sonucu |
| `orthogonal` | Birbirine dik sonlu vektörler | Sıfır izdüşüm ve signed zero sonucu |
| `zero-onto-preserves` | Signed zero bileşenli sıfır `onto` vektörü | Sıfır uzunluk karesi kısa yolu |
| `negative-fraction` | Negatif ve kesirli sonlu bileşenler | İşaret ve işlem sırası |
| `special-values` | Infinity bileşenleri ve sonlu `onto` | NaN yayılımı |
| `subnormal-onto` | En küçük subnormal `onto` bileşeni | Uzunluk karesi underflow'u ve kısa yol |

Girdiler `family: "math"`, `operation: "vec3_projected_on"`, `first` ve `onto` Vec3 nesnelerinden oluşur. Sonuç `value` içinde Vec3 snapshot'ı olarak döner.

## Rust karşılığı

`crates/photon-core/src/math.rs` içindeki `Vec3::projected_on`, önce `onto.length_squared()` değerini Java koşuluyla sınar; devam eden yolda `dot`, iki `scale` ve ters uzunluk karesi sırasını korur. `examples/behavior_adapter.rs` iki Vec3 girdisini ham f64 bitlerinden okur; `tests/behavior/java/MathOracle.java` aynı `Vec3.projectedOn` çağrısını oracle çıktısına bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec3.projectedOn(Vec3)` metodunu karşılar. Rust adaptörü sıfır uzunluk karesi kısa yolunu ve Java `dot → scale → scale` sırasını uygular; Java oracle doğrudan `Vec3.projectedOn` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Altı yeni senaryoda normal projection, orthogonal zero, signed-zero-preserving zero onto, negatif/fractional operands, Infinity/NaN propagation ve subnormal length-squared underflow karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.projectedOn(Vec3)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `toVector3f`, local coordinate helpers ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
