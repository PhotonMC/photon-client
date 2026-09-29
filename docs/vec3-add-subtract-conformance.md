# P01.b — Vec3 add/subtract overload sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `Vec3` sınıfının `add` ve `subtract` overload'larını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:96-120` içindeki `add` ve `subtract` overload'ları incelendi. Oracle doğrudan Java çağrılarını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `add(double, double, double)`, her bileşeni karşılık gelen miktarla toplar ve yeni bir `Vec3` üretir.
- `add(Vec3)`, diğer vektörün üç bileşenini kullanır; `add(double)` aynı scalar değeri üç eksene verir.
- `subtract(double, double, double)`, her miktarı Java unary negation ile ters işaretleyip `add` yoluna yönlendirir.
- `subtract(Vec3)` diğer vektörün bileşenlerini, `subtract(double)` aynı scalar değeri üç eksene verir.
- Bileşenler f64'tür. Signed zero, NaN ve sonsuzluk bitleri Java aritmetiğinin gözlenen sonucuyla karşılaştırılır; tolerans uygulanmaz.
- Her senaryo başlangıç ve her adım sonrası `x`, `y`, `z`, `length`, `length_squared` bitlerini kaydeder.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-chain` | Vec3 operand'larıyla add/subtract ve normalize zinciri | Vektör ve türetilmiş ölçümler |
| `scalar-chain` | Scalar add/subtract overload'ları | Sıralı vektör snapshot'ları |
| `component-chain` | Üç bileşenli overload'lar ve karma zincir | Sıralı vektör snapshot'ları |
| `signed-zero` | Signed zero başlangıç ve miktarları | Bileşen bitleri |
| `special-values` | Infinity ve NaN içeren miktarlar | Özel değer bitleri |

Girdiler `family: "math"`, `operation: "vec3"` ve `steps` içinde `add_vec3`, `add_scalar`, `add_components`, `subtract_vec3`, `subtract_scalar` veya `subtract_components` kullanır. Vec3 bileşenleri ve miktarlar 16 haneli küçük harfli f64 bit string'leridir.

## Rust karşılığı

`crates/voxel-core/src/math.rs` içindeki `Vec3::add_components`, `add_scalar`, `add_vec3`, `subtract_components`, `subtract_scalar` ve `subtract_vec3` Java overload'larını üretim API'sine taşır. `subtract` yolları Java unary negation'ın NaN işaretini release optimizasyonlarından bağımsız koruyan bit düzeyinde yardımcıyı kullanır. `examples/behavior_adapter.rs` bu çağrıları JSONL adımlarına bağlar.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Vec3.add` ve `Vec3.subtract` overload'larını karşılayan altı Rust metodu içerir. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3, scalar ve üç bileşenli overload zincirleri; signed zero, Infinity ve NaN girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
