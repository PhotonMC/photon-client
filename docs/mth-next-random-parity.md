# P01.a — Mth next random doğrulaması

`src/math.rs`, Java `Mth.nextInt(RandomSource, int, int)`, `nextFloat` ve `nextDouble` overload'larını uygular. `examples/behavior_adapter.rs` sabit seed ile oluşturulan `RandomSource` değerlerini Java oracle ile aynı JSON bit/değer biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 6 int, 7 f32 ve 7 f64 olmak üzere toplam **20 özgün senaryo** hazırlandı.
- Int için normal, eşit ve ters sınırlar, pozitif/negatif i32 uçları ve tek bitlik aralık; kayan nokta için eşit/ters sınırlar, signed zero, geniş sonlu aralık, Infinity ve NaN dalları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1301/1301** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1803/1803** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `a7681fdbc73c8faa861a3c0f74be7f2286257b8c494b6f1d3c85598ad65f6973`. Birleşik referans kimliği: `e26e5fb4cdc24beb08ca189d4d55904fcff008d93a9a99a606f787f60554f935`.

Adapter hashleri: debug `0612fecaa42e9ad3a6abed41dc6cf2df8dd739a3fd90d05b846233f866da6f50`, release `b4afd615d2f91a70f67b659baf315615f24eb5616422cda0fd87af58271de63a`.

Yerel raporlar: `compare-zq98bs3k` ve `compare-zgg863t0` (math debug/release), `compare-yz31bmcf` ve `compare-90dzj4qp` (birleşik debug/release), `verify-g1_f5udh` (math) ve `verify-l29xakrv` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç `Mth.next*` overload'unun sabit seed'li sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.randomBetweenInclusive`, `randomBetween`, `normal`, RandomSource'un diğer implementasyonları, overflow ile geçersiz bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
