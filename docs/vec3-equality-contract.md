# P01.b — Vec3 equals/hashCode sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `Vec3.equals(Object)` ve `Vec3.hashCode()` metodlarının üç f64 bileşen üzerindeki Java karşılaştırma ve hash kurallarını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:196-221` içindeki `equals` ve `hashCode` gövdeleri incelendi. Oracle doğrudan Java metodlarını çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `equals`, her bileşeni `Double.compare` ile karşılaştırır ve üç karşılaştırmanın da sıfır olmasını ister.
- Java karşılaştırmasında `+0.0` ile `-0.0` farklıdır; bütün NaN payload'ları canonical double karşılaştırması nedeniyle eşit kabul edilir.
- `hashCode`, her bileşen için `Double.doubleToLongBits` sonucunu yüksek ve düşük 32 bitlerin XOR'una indirger, sonra bileşenleri 31 tabanında birleştirir.
- Hash hesabı Java `int` aritmetiğiyle taşar; NaN payload'ları aynı canonical hash değerine sahiptir.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Eşit sonlu bileşenler | Eşitlik ve temel hash formülü |
| `different-component` | Tek bileşeni farklı vektörler | Eşitsizlik ve hash ayrımı |
| `signed-zero` | `-0.0` ile `+0.0` | Signed zero karşılaştırması ve hash'i |
| `nan-payloads` | Farklı NaN payload'ları | NaN canonicalization |
| `infinities` | Aynı pozitif/negatif Infinity bileşenleri | Sonsuzluk eşitliği |
| `special-mismatch` | Infinity, NaN ve signed zero farkları | Özel değerlerin birlikte karşılaştırılması |

Girdiler `family: "math"`, `operation: "vec3_equality"` ve f64 bit dizgeli `first`/`second` Vec3 nesnelerinden oluşur. Sonuç eşitlik boolean'ı ile iki Java int hash değerini decimal string olarak taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::equals_java` ve `Vec3::hash_code_java`, Java'nın canonical double karşılaştırmasını ve wrapping int hash aritmetiğini uygular. `examples/behavior_adapter.rs` sonucu taşır; `tests/behavior/java/MathOracle.java` aynı Java metodlarını çağırır.
