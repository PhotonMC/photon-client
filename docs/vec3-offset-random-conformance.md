# P01.b — Vec3 offsetRandom sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.offsetRandom(RandomSource, float)` ve `Vec3.offsetRandomXZ(RandomSource, float)` overload'larını aynı seed ve offset ile birlikte içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:172-178` içindeki iki gövde ve `util/RandomSource.java:53` içindeki `nextFloat()` sözleşmesi incelendi. Oracle doğrudan Java çağrılarını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Her random bileşen `(random.nextFloat() - 0.5F) * offset` ifadesiyle f32 hassasiyetinde hesaplanır ve `Vec3.add(double, double, double)` çağrısına f64 olarak genişletilir.
- `offsetRandom` x, y ve z için sırayla üç `nextFloat()` çekişi yapar.
- `offsetRandomXZ` x ve z için iki çekiş yapar; y argümanı Java `0.0` double literal'ıdır ve pozitif f64 sıfırı olarak eklenir.
- İki overload aynı seed ile ayrı `RandomSource` örnekleri üzerinde çalıştırılır; böylece karşılaştırma aynı ilk random çekişleri kullanır.
- Signed zero, subnormal offset, büyük sonlu offset, Infinity ve NaN bileşenleri toleranssız ham f64 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu Vec3, seed `42`, offset `1.0f` | Üç çekiş ve XZ iki çekiş sırası |
| `negative-offset` | Negatif offset ve negatif seed | f32 çıkarma/çarpma işareti |
| `signed-zero` | Signed-zero Vec3 ve `-0.0f` offset | Sıfır işaretleri ve XZ y sıfırı |
| `subnormal-offset` | En küçük f32 subnormal offset | f32 underflow ve f64 bileşenlerin korunması |
| `special-values` | Infinity/NaN Vec3 ve en büyük sonlu f32 offset | Özel değer yayılımı ve f32 offset genişlemesi |

Girdiler `family: "math"`, `operation: "vec3_offset_random"`, `first`, decimal `seed` ve sekiz haneli f32 bit string'i `offset` alanlarını taşır. Sonuç `offset_random` ve `offset_random_xz` alanlarında Vec3 snapshot'ları içerir; bileşenler, uzunluk ve uzunluk karesi 16 haneli küçük harfli f64 bit string'leridir.

## Rust karşılığı

`crates/voxel-core/src/math.rs` içindeki `Vec3::offset_random` ve `Vec3::offset_random_xz`, random çekiş sırasını ve f32 ara hesabını taşır. `examples/behavior_adapter.rs` seed'i yeniden kurarak iki overload'u çağırır; `tests/behavior/java/MathOracle.java` aynı iki Java çağrısını doğrudan oracle çıktısına bağlar.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Vec3.offsetRandom(RandomSource, float)` ve `Vec3.offsetRandomXZ(RandomSource, float)` overload'larını karşılar. Rust adaptörü aynı seed ile iki Rust `RandomSource` örneği kurar; Java oracle iki overload'u doğrudan çağırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda üç ve iki `nextFloat()` çekişi, f32 offset ara hesabı, negatif offset, signed zero, subnormal offset ve özel Vec3 değerleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde iki random offset overload'unun Vec3 snapshot'larıyla sınırlıdır. `RandomSource`'un diğer üreticileri ve `Vec3`'ün kalan yardımcıları ayrı dilimlerdir.
