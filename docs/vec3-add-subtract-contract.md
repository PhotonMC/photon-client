# P01.b — Vec3 add/subtract overload sözleşmesi

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](vec3-add-subtract-parity.md)). Kapsam, Java `Vec3` sınıfının `add` ve `subtract` overload'larını içerir.

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

`src/math.rs` içindeki `Vec3::add_components`, `add_scalar`, `add_vec3`, `subtract_components`, `subtract_scalar` ve `subtract_vec3` Java overload'larını üretim API'sine taşır. `subtract` yolları Java unary negation'ın NaN işaretini release optimizasyonlarından bağımsız koruyan bit düzeyinde yardımcıyı kullanır. `examples/behavior_adapter.rs` bu çağrıları JSONL adımlarına bağlar.
