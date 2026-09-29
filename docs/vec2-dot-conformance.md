# P01.b — Vec2 dot sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2.dot(Vec2)` metodunun iki f32 bileşen çarpımını kaynak sırasıyla toplamasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:33-35` içindeki `dot(Vec2)` incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- x bileşenleri ve y bileşenleri ayrı ayrı f32 olarak çarpılır.
- İki ürün kaynakta yazıldığı sırayla f32 toplama işlemine verilir.
- Sonuç tek bir f32 değerdir; giriş vektörleri değişmez.
- Ara yuvarlama, cancellation, taşma, signed zero, Infinity ve NaN sonuçları ham IEEE-754 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu, farklı bileşenli iki Vec2 | Dot sonucu |
| `order-and-rounding` | Küçük f32 terim ile yuvarlama sınırı | Kaynak sıralı toplam |
| `signed-zero` | Negatif sıfır çarpımları | Sonuç işareti |
| `overflow` | En büyük sonlu f32 ile çarpım | Infinity bitleri |
| `special-values` | Infinity ile sıfır çarpımı | NaN bitleri |

Girdiler `family: "math"`, `operation: "vec2_dot"`, `first` ve `second` Vec2 nesnelerini kullanır. Bileşenler 8 haneli küçük harfli f32 bit string'leridir; sonuç `value` alanında aynı biçimde döner.

## Rust karşılığı

`crates/voxel-core/src/math.rs` içindeki `Vec2::dot`, iki f32 çarpımını ve kaynak sıralı toplamı uygular. `examples/behavior_adapter.rs` `vec2_dot` operasyonunu, `tests/behavior/java/MathOracle.java` ise beklenen değeri doğrudan Java `dot` çağrısını kullanarak bağlar.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Vec2.dot(Vec2)` metodunu karşılayan `Vec2::dot` metodunu içerir. Java oracle doğrudan `dot` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda temel çarpım, f32 yuvarlama, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 çifti ve dot sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 add, scale, normalize, distance ve diğer yardımcıları ayrı davranış dilimleridir.
