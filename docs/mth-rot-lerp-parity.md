# P01.a — Mth rotLerp doğrulaması

`src/math.rs`, Java `Mth.rotLerp` float ve double overload'larının fark çıkarma, `wrapDegrees` ve `from + alpha * fark` işlem sırasını taşır. `examples/behavior_adapter.rs` sonucu overload hassasiyetine göre raw f32 veya f64 bit dizgesi olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Temel interpolation, extrapolation, ±180 sınırı, tam tur, signed zero, subnormal, en büyük sonlu değer, Infinity ve NaN içeren **30 özgün senaryo** hazırlandı; 15 f32 ve 15 f64 senaryosu vardır.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2100/2100** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2602/2602** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `f84b8d0a8b920d9e5455aefeebd2a9d768b7c9af5ace5dd1c90ff816a9e20abb`. Birleşik referans kimliği: `ec46e2f5bb40c4fdc045039ffcf03e500f593e7a5659dcd76ba9519601df03f2`.

Adapter hashleri: debug `08c3d7a5f592502ade6a94189a9046d939bb3530591885c02a0d6e734d3a67f5`, release `278394f6e917e92bfa810fe86c950acff09c5f0e74a5807ab45be9fe4ac423158`.

Yerel raporlar: `compare-f8rnhap6` ve `compare-9ykm10o3` (math debug/release), `compare-qo27kk4p` ve `compare-066pmptb` (birleşik debug/release), `verify-_rkkkp6r` (math) ve `verify-icag4r7w` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.rotLerp` iki overload'unun sabit raw f32/f64 girdilerdeki sonucunu kapsar. `rotLerpRad`, `triangleWave`, `square`, diğer açı yardımcıları ve `rotLerp` kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
