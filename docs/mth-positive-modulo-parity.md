# P01.a — Mth positiveModulo doğrulaması

`src/math.rs`, Java `Mth.positiveModulo(float, float)` ve `positiveModulo(double, double)` yollarını aynı hassasiyette `(input % mod + mod) % mod` işlem sırasıyla uygular. `examples/behavior_adapter.rs` iki overload'u JSONL senaryolarına bağlar; sonuçlar raw f32/f64 bit string'i olarak karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 22, toplam **44 özgün senaryo** hazırlandı.
- Pozitif/negatif girdiler ve modüller, tam katlar, signed zero, f32/f64 subnormal ve maksimum sonlu girdiler, sıfır, Infinity ve NaN yolları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1301/1301** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1803/1803** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Java'nın sabitlenmiş Linux x86_64 runtime'ında invalid floating-point remainder sonuçları raw NaN işaretiyle gözlenir. Rust native remainder'ın işaretinden ayrılan bu yollar, Java çıktısına uygun canonical bitlerle korunur: sonlu input ve sonsuz modulus için negatif canonical NaN; diğer kapsanan invalid yollar için pozitif canonical NaN. Sonlu sonuçlar ve signed zero sonuçları doğrudan işlem sırasından gelir; karşılaştırıcı genel NaN normalizasyonu yapmaz.

Math referans kimliği: `a7681fdbc73c8faa861a3c0f74be7f2286257b8c494b6f1d3c85598ad65f6973`. Birleşik referans kimliği: `e26e5fb4cdc24beb08ca189d4d55904fcff008d93a9a99a606f787f60554f935`.

Adapter hashleri: debug `0612fecaa42e9ad3a6abed41dc6cf2df8dd739a3fd90d05b846233f866da6f50`, release `b4afd615d2f91a70f67b659baf315615f24eb5616422cda0fd87af58271de63a`.

Yerel raporlar: `compare-zq98bs3k` ve `compare-zgg863t0` (math debug/release), `compare-yz31bmcf` ve `compare-90dzj4qp` (birleşik debug/release), `verify-g1_f5udh` (math) ve `verify-l29xakrv` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `positiveModulo` overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.equal`, `absMax`, `clampedLerp` ve diğer floating-point yardımcıları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
