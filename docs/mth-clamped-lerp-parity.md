# P01.a — Mth clampedLerp doğrulaması

`src/math.rs`, Java `Mth.clampedLerp(float, float, float)` ve `clampedLerp(double, double, double)` yollarındaki clamp dallarını ve `min + factor * (max - min)` işlem sırasını uygular. `examples/behavior_adapter.rs` sonucu raw f32/f64 bitleri olarak JSONL sözleşmesine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 16, toplam **32 özgün senaryo** hazırlandı.
- Negatif, sıfır, signed zero, aralık içi, bir, üst sınır ve Infinity/NaN faktörleri; ters aralık, subnormal, sonlu overflow, sonsuz aralık ve NaN endpoint dalları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2023/2023** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2525/2525** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `1a217debf64171ea8f99e220ffd6aca1130ecceef3e6015b82f0c12a4e3f9f0d`. Birleşik referans kimliği: `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5`.

Adapter hashleri: debug `0f53bfe9b4fa3f51b5f1cac8229fea64a4d485c5c5fae4f8b5607a300ecb94e8`, release `61ae7de80355a2cf73c393fbd30be18b82042d583a1918baf991fa8a22629e888`.

Yerel raporlar: `compare-4sx1lm2m` ve `compare-m3ib6sjn` (math debug/release), `compare-czvhou3w` ve `compare-oz19j96e` (birleşik debug/release), `verify-xjnq5poq` (math) ve `verify-nukwd4yc` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `clampedLerp` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.clamp`, bağımsız `Mth.lerp`, `inverseLerp`, Vec3 lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
