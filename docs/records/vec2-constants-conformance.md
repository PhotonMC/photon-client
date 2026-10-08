# P01.b — Vec2 sabitleri sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **8 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2` sınıfının public static sabitlerinin bileşen değerleridir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:9-17` incelendi. Oracle sabitleri doğrudan okur; beklenen sonuçlar Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `ZERO`, iki pozitif sıfır bileşeninden oluşur.
- `ONE`, iki adet `1.0F` bileşeni taşır.
- `UNIT_X`, `NEG_UNIT_X`, `UNIT_Y` ve `NEG_UNIT_Y`, ilgili eksende `1.0F` veya `-1.0F`, diğer eksende pozitif `0.0F` kullanır.
- `MAX`, iki `Float.MAX_VALUE` bileşeni taşır.
- `MIN`, Java `Float.MIN_VALUE` değerini kullanır. Bu değer en küçük pozitif subnormal f32’dir ve ham bitleri `0x00000001` olur; Rust `f32::MIN_POSITIVE` ile aynı değildir.
- Her sabit sorgusu bileşenleri ve mevcut Vec2 snapshot’ındaki uzunluk değerlerini ham f32 bitleriyle karşılaştırır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `zero` | `Vec2.ZERO` adı | Vec2 snapshot’ı |
| `one` | `Vec2.ONE` adı | Vec2 snapshot’ı |
| `unit_x` / `neg_unit_x` | X ekseni sabiti adı | Vec2 snapshot’ı |
| `unit_y` / `neg_unit_y` | Y ekseni sabiti adı | Vec2 snapshot’ı |
| `max` | `Vec2.MAX` adı | Vec2 snapshot’ı |
| `min` | `Vec2.MIN` adı | Vec2 snapshot’ı |

Girdiler `family: "math"`, `operation: "vec2_constant"` ve `name` alanını kullanır. Sonuç `value` içinde f32 `x`, `y`, `length` ve `length_squared` bitlerini taşır.

## Rust karşılığı

`crates/photon-core/src/math.rs` içindeki `Vec2` associated constant’ları Java değerlerini taşır. `MIN`, Java’nın pozitif subnormal sınırını `f32::from_bits(1)` ile kurar. `examples/behavior_adapter.rs` sabiti doğrudan Rust API’sinden sorgular; `tests/behavior/java/MathOracle.java` aynı adı doğrudan Java sabitine bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec2.ZERO`, `ONE`, `UNIT_X`, `NEG_UNIT_X`, `UNIT_Y`, `NEG_UNIT_Y`, `MAX` ve `MIN` sabitlerini associated constant olarak sunar. Java oracle sabitleri doğrudan okur; Rust adaptörü aynı değerleri üretim API’sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Sekiz yeni senaryoda bütün public Vec2 sabitlerinin f32 bileşenleri, `MAX` sınırı ve `MIN` pozitif subnormal değeri karşılaştırıldı.
- Birleşik altı aile corpus’u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt sekiz özgün sabit sorgusu ve Vec2 snapshot sonuçlarıyla sınırlıdır. Sabitlerin oyun davranışındaki tüm tüketicileri sonraki istemci dilimleridir.
