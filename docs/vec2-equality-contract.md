# P01.b — Vec2 equals/hashCode sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2.equals(Object)` ve `Vec2.hashCode()` metodlarının iki f32 bileşen üzerindeki davranışıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:45-60` içindeki `equals` ve `hashCode` gövdeleri incelendi. Oracle doğrudan Java çağrılarını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `equals(Object)`, Vec2 olmayan nesneleri reddeder; Vec2 bileşenlerini Java `==` ile karşılaştırır.
- `==` signed zero değerlerini eşit, NaN değerlerini eşitsiz ve aynı işaretli Infinity değerlerini eşit kabul eder.
- `hashCode()`, `Objects.hash(this.x, this.y)` yolunu kullanır: başlangıç değeri 1, her Float hash'i 31 tabanlı Java int aritmetiğiyle eklenir.
- Float hash'i NaN'i `0x7fc00000` bitlerine canonicalize eder; signed zero hash bitleri korunur. Java'daki eşitlik/hash ayrışması ham sonuçlarla korunur.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Aynı sonlu Vec2 | Eşitlik ve iki hash |
| `different-component` | Tek bileşeni farklı Vec2 | Eşitsizlik ve hash farkı |
| `signed-zero` | Negatif/pozitif sıfır | Java `==` eşitliği ve farklı hash bitleri |
| `nan-payloads` | Farklı NaN payload'ları | Eşitsizlik ve canonical hash'ler |
| `infinities` | Aynı pozitif/negatif Infinity | Eşitlik ve Infinity hash'leri |
| `special-mismatch` | Farklı Infinity ve NaN bileşenleri | Eşitsizlik ve hash sonuçları |

Girdiler `family: "math"`, `operation: "vec2_equality"` ve f32 bit dizgeli `first`/`second` Vec2 nesnelerinden oluşur. Sonuç `equals` boolean'ı ile iki Java int hash değerini decimal string olarak taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec2::equals_java` ve `Vec2::hash_code_java`, Java'nın f32 `==`, canonical NaN ve wrapping int hash aritmetiğini uygular. `examples/behavior_adapter.rs` sonucu taşır; `tests/behavior/java/MathOracle.java` aynı Java metodlarını çağırır.
