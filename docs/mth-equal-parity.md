# P01.a — Mth equal doğrulaması

`src/math.rs`, Java `Mth.equal(float, float)` ve `equal(double, double)` yollarını `Math.abs(second - first) < 1.0E-5F` sırasıyla uygular. `examples/behavior_adapter.rs` sonucu boolean olarak JSONL sözleşmesine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 14, toplam **28 özgün senaryo** hazırlandı.
- Eşik altı, eşik üzeri ve eşik eşitliği, signed zero, subnormal, f32/f64 hassasiyet farkı, Infinity ve NaN girdileri karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1548/1548** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2050/2050** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `8d3d3f4690e190086d874fa8e58bdb837efcc4649b07170de736639c38e7aa7d`. Birleşik referans kimliği: `0d4c76950a37451c226bb147399cfa08349c81c96e4a8ea2b85e47696f0caf11`.

Adapter hashleri: debug `5a18309c0cb6402cc39c0f09e9434461f609f7c57df7f6ec9deeae5bc6138fb0`, release `a7ce3c141fe387a5aa243e759a9ff22d3798bd7e7f3f57b404f68fecc66444f4`.

Yerel raporlar: `compare-m8bramo3` ve `compare-20kwpz7m` (math debug/release), `compare-hmu92wq0` ve `compare-ha45dyno` (birleşik debug/release), `verify-mzhe5jke` (math) ve `verify-46sjfnqt` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `equal` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.absMax`, positiveModulo, clamp/lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
