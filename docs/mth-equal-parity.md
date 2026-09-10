# P01.a — Mth equal doğrulaması

`src/math.rs`, Java `Mth.equal(float, float)` ve `equal(double, double)` yollarını `Math.abs(second - first) < 1.0E-5F` sırasıyla uygular. `examples/behavior_adapter.rs` sonucu boolean olarak JSONL sözleşmesine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 14, toplam **28 özgün senaryo** hazırlandı.
- Eşik altı, eşik üzeri ve eşik eşitliği, signed zero, subnormal, f32/f64 hassasiyet farkı, Infinity ve NaN girdileri karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1623/1623** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2125/2125** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `ad2753dc2b6d8419f59cf4874d459be1bbd0128324ea6d37e1afd0b11e7854cf`. Birleşik referans kimliği: `14062308d34164b4d537528e94d89e360ce28e02bc1488fc00ccd7a15390b0eb`.

Adapter hashleri: debug `c3969f81492d63e29e9009584e90bb6dc01c762199c440bd17b357bc80e0919a`, release `84c183f560401d08c3a5a0f1f61ffc0c10820995f937749df5807cb3198f1c19`.

Yerel raporlar: `compare-ny3j5bmv` ve `compare-8wh6otww` (math debug/release), `compare-us5kn_kw` ve `compare-qnl_96jt` (birleşik debug/release), `verify-dybssizj` (math) ve `verify-m1vz27n6` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `equal` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.absMax`, positiveModulo, clamp/lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
