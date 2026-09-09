# P01.b — Vec3 multiply overload sözleşmesi

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](vec3-multiply-parity.md)). Kapsam, Java `Vec3.multiply` overload'larıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:160-166` içindeki `multiply(Vec3)` ve `multiply(double, double, double)` gövdeleri incelendi. Oracle doğrudan Java overload çağrılarını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `multiply(Vec3)`, ölçek vektörünün x, y ve z bileşenlerini üç bileşenli overload'a kaynak sırasıyla aktarır.
- Üç bileşenli overload mevcut vektörün x, y ve z bileşenlerini karşılık gelen ölçeklerle çarpar ve yeni bir Vec3 üretir.
- Girdi vektörü değiştirilmez; her adımın sonucu sonraki adımın girdisi olur.
- Signed zero, subnormal, NaN, Infinity ve taşma sonuçları ham IEEE-754 f64 bitleriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-basic` | Vec3 ölçek operand'ı | `multiply(Vec3)` sonrası snapshot |
| `components-basic` | Ayrı x/y/z ölçekleri | Üç bileşenli overload sonrası snapshot |
| `overload-chain` | İki overload'ın ardışık kullanımı | Kaynak işlem sırasındaki snapshot'lar |
| `signed-zero` | İşaretli sıfır bileşenleri ve ölçekleri | Bileşen bitleri |
| `overflow-and-special-values` | En büyük sonlu değer, Infinity ve NaN | Taşma/özel değer bitleri |

Girdiler `family: "math"`, `operation: "vec3"` ve `steps` içinde `multiply_vec3` veya `multiply_components` kullanır. Vec3 bileşenleri ve ölçekler 16 haneli küçük harfli f64 bit string'leridir; `result.events` başlangıç ve her adım sonrası snapshot'ı taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::multiply_vec3` ve `Vec3::multiply_components`, Java overload yönlendirmesini ve x/y/z çarpım sırasını üretim API'sine taşır. `examples/behavior_adapter.rs` bu çağrıları JSONL adımlarına, `tests/behavior/java/MathOracle.java` ise doğrudan Java çağrılarına bağlar.
