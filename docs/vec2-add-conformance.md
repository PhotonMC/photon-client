# P01.b — Vec2 add overload'ları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2.add(Vec2)` ve `Vec2.add(float)` overload'larının f32 bileşen toplamasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:37-43` içindeki iki `add` overload'u incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `add(Vec2)`, x bileşenlerini ve y bileşenlerini ayrı ayrı f32 olarak toplar.
- `add(float)`, aynı scalar değeri iki bileşene de ayrı ayrı ekler.
- Her bileşen kaynakta yazıldığı sırayla hesaplanır; sonuç yeni Vec2 içindedir.
- Ara yuvarlama, signed zero, taşma, Infinity ve NaN sonuçları ham IEEE-754 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-basic` | Sonlu iki Vec2 | Vec2 toplamı |
| `vector-signed-zero` | Negatif sıfır bileşenleri | Sonuç işaretleri |
| `vector-overflow` | En büyük sonlu f32 bileşenleri | Infinity bitleri |
| `scalar-basic` | Sonlu Vec2 ve scalar | Her bileşene scalar eklenmesi |
| `scalar-special-values` | Infinity ve negatif Infinity | NaN/Infinity bitleri |

Girdiler `family: "math"`, `operation: "vec2_add_vec2"` veya `operation: "vec2_add_scalar"` kullanır. Vec2 bileşenleri 8 haneli küçük harfli f32 bit string'leridir; sonuç `value` alanında Vec2 snapshot'ı olarak döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec2::add_vec2` ve `Vec2::add_scalar`, Java overload'larının bileşen işlem sırasını uygular. `examples/behavior_adapter.rs` iki operasyonu, `tests/behavior/java/MathOracle.java` ise beklenen değerleri doğrudan Java `add` çağrılarıyla bağlar.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec2.add(Vec2)` ve `Vec2.add(float)` metodlarını karşılayan `Vec2::add_vec2` ve `Vec2::add_scalar` metodlarını içerir. Java oracle doğrudan `add` çağrılarını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec2 ve scalar overload'ları, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve add sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 dot, rotate, distance ve diğer yardımcıları ayrı davranış dilimleridir.
