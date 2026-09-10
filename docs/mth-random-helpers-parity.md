# P01.a — Mth random helpers doğrulaması

`src/math.rs`, Java `Mth.randomBetweenInclusive(RandomSource, int, int)`, `randomBetween(RandomSource, float, float)` ve `normal(RandomSource, float, float)` yollarını uygular. `examples/behavior_adapter.rs` helper sonucunu ve kontrollü takip çekişini Java oracle ile aynı JSON bit/değer biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 6 inclusive int, 6 f32 aralık ve 6 normal olmak üzere toplam **18 özgün senaryo** hazırlandı.
- Inclusive int için eşit sınırda `nextInt(1)` tüketimi, pozitif/negatif i32 uçları ve geniş geçerli aralıklar; f32 aralık için ters/eşit sınırlar, signed zero, Infinity ve NaN; normal için mean/deviation, signed zero, Infinity ve negatif deviation yolları karşılaştırıldı.
- Her helper sonucundan sonra kontrollü bir takip çekişi yapıldı; normal yolunda takip Gaussian değeri cache davranışını da doğruladı.
- Rust debug ve release adaptörleri güncel **1398/1398** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `4cc46837e07b2a435f9a79ffddebd09d7fed0ad74e1310e64453de08cb4bf4bd`. Birleşik referans kimliği: `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea`.

Adapter hashleri: debug `7fcaaf04b4fa67068cb95022cc7e832f9013a10846c57ad8dee850949ac64f59`, release `b98e59cbf911f59510b79a9d182e31d455c93c6084ae18d3ce74b07bab212466`.

Yerel raporlar: `compare-xljwmsb7` ve `compare-s_oqvkq3` (math debug/release), `compare-uf5y7y3a` ve `compare-wrj9a6ex` (birleşik debug/release), `verify-_l31ex7g` (math) ve `verify-5ju3ok97` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç `Mth` random helper overload'unun sabit seed'li sonuçlarına, takip çekişlerine ve mevcut corpus regresyonuna aittir. `Mth.nextInt/nextFloat/nextDouble`, RandomSource'un diğer implementasyonları, overflow ile geçersiz inclusive bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
