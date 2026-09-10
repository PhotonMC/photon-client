# P01.a — Mth random helpers doğrulaması

`src/math.rs`, Java `Mth.randomBetweenInclusive(RandomSource, int, int)`, `randomBetween(RandomSource, float, float)` ve `normal(RandomSource, float, float)` yollarını uygular. `examples/behavior_adapter.rs` helper sonucunu ve kontrollü takip çekişini Java oracle ile aynı JSON bit/değer biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 6 inclusive int, 6 f32 aralık ve 6 normal olmak üzere toplam **18 özgün senaryo** hazırlandı.
- Inclusive int için eşit sınırda `nextInt(1)` tüketimi, pozitif/negatif i32 uçları ve geniş geçerli aralıklar; f32 aralık için ters/eşit sınırlar, signed zero, Infinity ve NaN; normal için mean/deviation, signed zero, Infinity ve negatif deviation yolları karşılaştırıldı.
- Her helper sonucundan sonra kontrollü bir takip çekişi yapıldı; normal yolunda takip Gaussian değeri cache davranışını da doğruladı.
- Rust debug ve release adaptörleri güncel **1548/1548** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2050/2050** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `8d3d3f4690e190086d874fa8e58bdb837efcc4649b07170de736639c38e7aa7d`. Birleşik referans kimliği: `0d4c76950a37451c226bb147399cfa08349c81c96e4a8ea2b85e47696f0caf11`.

Adapter hashleri: debug `5a18309c0cb6402cc39c0f09e9434461f609f7c57df7f6ec9deeae5bc6138fb0`, release `a7ce3c141fe387a5aa243e759a9ff22d3798bd7e7f3f57b404f68fecc66444f4`.

Yerel raporlar: `compare-m8bramo3` ve `compare-20kwpz7m` (math debug/release), `compare-hmu92wq0` ve `compare-ha45dyno` (birleşik debug/release), `verify-mzhe5jke` (math) ve `verify-46sjfnqt` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç `Mth` random helper overload'unun sabit seed'li sonuçlarına, takip çekişlerine ve mevcut corpus regresyonuna aittir. `Mth.nextInt/nextFloat/nextDouble`, RandomSource'un diğer implementasyonları, overflow ile geçersiz inclusive bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
