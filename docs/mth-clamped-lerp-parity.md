# P01.a — Mth clampedLerp doğrulaması

`src/math.rs`, Java `Mth.clampedLerp(float, float, float)` ve `clampedLerp(double, double, double)` yollarındaki clamp dallarını ve `min + factor * (max - min)` işlem sırasını uygular. `examples/behavior_adapter.rs` sonucu raw f32/f64 bitleri olarak JSONL sözleşmesine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 16, toplam **32 özgün senaryo** hazırlandı.
- Negatif, sıfır, signed zero, aralık içi, bir, üst sınır ve Infinity/NaN faktörleri; ters aralık, subnormal, sonlu overflow, sonsuz aralık ve NaN endpoint dalları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1478/1478** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1980/1980** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `abe3d8cdf014db1ee9b2fa953ff1b7ba7bc40e31c9d35672c0cd8adb20896f50`. Birleşik referans kimliği: `e068e38cc1d0ab64ac2844f20d474f313e2325b42cc620af39051ca8aa6748cd`.

Adapter hashleri: debug `639376b4d3cf372f630ca79f26c76c925bf13904a7589eed8fca87f128139dd9`, release `766e0548962d2a8ef9defd19efec60a2b73ae570040cbdac0c16bb9a136e8d85`.

Yerel raporlar: `compare-xrmyz06w` ve `compare-zrxwzjp_` (math debug/release), `compare-dxdmken0` ve `compare-cvif6gdr` (birleşik debug/release), `verify-nrvzr4d3` (math) ve `verify-jlkp_8wf` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `clampedLerp` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.clamp`, bağımsız `Mth.lerp`, `inverseLerp`, Vec3 lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
