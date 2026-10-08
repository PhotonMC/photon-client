# P01.b — Vec3 toVector3f sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.toVector3f()` metodunun f64 bileşenleri JOML `Vector3f` içindeki f32 bileşenlere dönüştürmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:319-321` içindeki üç ayrı `(float)` dönüşümü ve yeni `Vector3f` kurulumu incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- x, y ve z bileşenleri birbirinden bağımsız olarak Java f64 → f32 daraltmasından geçirilir.
- Sonuç üç f32 bileşenli yeni bir `Vector3f` değeridir; Vec3'ün f64 bileşenleri değişmez.
- IEEE-754 yuvarlama, signed zero, subnormal değerlerin sıfıra veya f32 subnormal'a dönüşmesi, overflow, Infinity ve NaN ham f32 bitleriyle karşılaştırılır.
- Sonuç `Vector3f` olduğundan Vec3'e özgü length ve length squared alanları bu operasyona dahil değildir.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu, tam temsil edilebilir bileşenler | Üç bağımsız f32 dönüşümü |
| `rounding` | f32 aralığında yuvarlama gerektiren f64 değerler | Java daraltma yuvarlaması |
| `signed-zero` | Pozitif/negatif sıfır ve en küçük f64 subnormal | İşaret bitleri ve sıfıra underflow |
| `subnormal` | f64 subnormal'ları ve f32 en küçük subnormal sınırı | f32 subnormal üretimi |
| `overflow` | Çok büyük pozitif/negatif sonlu değerler | f32 Infinity taşması |
| `special-values` | Infinity ve özel NaN payload'ı | Özel değer işaret/payload sonucu |

Girdiler `family: "math"`, `operation: "vec3_to_vector3f"` ve f64 bit dizgeli `first` Vec3 nesnesinden oluşur. Sonuç `value` içinde f32 `x`, `y`, `z` ham bit snapshot'ı olarak döner.

## Rust karşılığı

`crates/photon-core/src/math.rs` içindeki `Vec3::to_vector3f`, üç bileşeni ayrı ayrı `f32`'ye daraltıp `Vec3f` değeri üretir. `examples/behavior_adapter.rs` sonucu f32 ham bitleriyle taşır; `tests/behavior/java/MathOracle.java` aynı `Vec3.toVector3f()` çağrısını oracle çıktısına bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec3.toVector3f()` metodunu karşılar. Rust `Vec3f` değeri üç bileşeni Java'nın bağımsız f64 → f32 daraltmalarıyla üretir; Java oracle doğrudan `Vec3.toVector3f()` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Altı yeni senaryoda temel f32 dönüşüm, f64 → f32 yuvarlama, signed zero, f64/f32 subnormal sınırları, sonlu değer taşması ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.toVector3f()` sonucunun üç f32 bileşen snapshot'ıyla sınırlıdır. `Vec3`'ün f64 işlemleri, `Vector3f` mutasyon yardımcıları ve diğer dönüşümler ayrı dilimlerdir.
