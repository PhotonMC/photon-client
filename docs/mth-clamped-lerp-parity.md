# P01.a — Mth clampedLerp doğrulaması

`src/math.rs`, Java `Mth.clampedLerp(float, float, float)` ve `clampedLerp(double, double, double)` yollarındaki clamp dallarını ve `min + factor * (max - min)` işlem sırasını uygular. `examples/behavior_adapter.rs` sonucu raw f32/f64 bitleri olarak JSONL sözleşmesine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 16, toplam **32 özgün senaryo** hazırlandı.
- Negatif, sıfır, signed zero, aralık içi, bir, üst sınır ve Infinity/NaN faktörleri; ters aralık, subnormal, sonlu overflow, sonsuz aralık ve NaN endpoint dalları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1653/1653** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2155/2155** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `e9b1c1e55b2a03528b712cd304101c7b36378556372ebad60bb330a4a60eb192`. Birleşik referans kimliği: `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e`.

Adapter hashleri: debug `2371e57dd5b14642fa55352a223485b414fbdeb72599f35405f7e4517da68372`, release `ce1f002fb70be2fce7ebe09258c7d8ffbc7eed67c9643ef81eeb6da2dabb4ecb`.

Yerel raporlar: `compare-ceusm4y4` ve `compare-hconpy1a` (math debug/release), `compare-rhjx0z7e` ve `compare-jtl411eu` (birleşik debug/release), `verify-biycqhmq` (math) ve `verify-u5d5zd4c` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `clampedLerp` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.clamp`, bağımsız `Mth.lerp`, `inverseLerp`, Vec3 lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
