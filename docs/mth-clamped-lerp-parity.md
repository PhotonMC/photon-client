# P01.a — Mth clampedLerp doğrulaması

`src/math.rs`, Java `Mth.clampedLerp(float, float, float)` ve `clampedLerp(double, double, double)` yollarındaki clamp dallarını ve `min + factor * (max - min)` işlem sırasını uygular. `examples/behavior_adapter.rs` sonucu raw f32/f64 bitleri olarak JSONL sözleşmesine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 16, toplam **32 özgün senaryo** hazırlandı.
- Negatif, sıfır, signed zero, aralık içi, bir, üst sınır ve Infinity/NaN faktörleri; ters aralık, subnormal, sonlu overflow, sonsuz aralık ve NaN endpoint dalları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2100/2100** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2602/2602** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `f84b8d0a8b920d9e5455aefeebd2a9d768b7c9af5ace5dd1c90ff816a9e20abb`. Birleşik referans kimliği: `ec46e2f5bb40c4fdc045039ffcf03e500f593e7a5659dcd76ba9519601df03f2`.

Adapter hashleri: debug `08c3d7a5f592502ade6a94189a9046d939bb3530591885c02a0d6e734d3a67f5`, release `278394f6e917e92bfa810fe86c950acff09c5f0e74a5807ab45be9fe4ac423158`.

Yerel raporlar: `compare-f8rnhap6` ve `compare-9ykm10o3` (math debug/release), `compare-qo27kk4p` ve `compare-066pmptb` (birleşik debug/release), `verify-_rkkkp6r` (math) ve `verify-icag4r7w` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `clampedLerp` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.clamp`, bağımsız `Mth.lerp`, `inverseLerp`, Vec3 lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
