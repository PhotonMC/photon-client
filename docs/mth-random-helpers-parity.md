# P01.a — Mth random helpers doğrulaması

`src/math.rs`, Java `Mth.randomBetweenInclusive(RandomSource, int, int)`, `randomBetween(RandomSource, float, float)` ve `normal(RandomSource, float, float)` yollarını uygular. `examples/behavior_adapter.rs` helper sonucunu ve kontrollü takip çekişini Java oracle ile aynı JSON bit/değer biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 6 inclusive int, 6 f32 aralık ve 6 normal olmak üzere toplam **18 özgün senaryo** hazırlandı.
- Inclusive int için eşit sınırda `nextInt(1)` tüketimi, pozitif/negatif i32 uçları ve geniş geçerli aralıklar; f32 aralık için ters/eşit sınırlar, signed zero, Infinity ve NaN; normal için mean/deviation, signed zero, Infinity ve negatif deviation yolları karşılaştırıldı.
- Her helper sonucundan sonra kontrollü bir takip çekişi yapıldı; normal yolunda takip Gaussian değeri cache davranışını da doğruladı.
- Rust debug ve release adaptörleri güncel **1783/1783** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2285/2285** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `c9c0c64d34c86e78a480fb2f63bd3bd5633a857332ad5997d97039c193be5338`. Birleşik referans kimliği: `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90`.

Adapter hashleri: debug `843cc0c04c00803acc1e8bf774f290df5e273c7777789a8df5b971cb6a7f1ec0`, release `200c25d47d60d5c096012435e86b4b9f28f06965fa1ac6a453b948b2e6e33f4e`.

Yerel raporlar: `compare-1g27oorn` ve `compare-bkcvw7oi` (math debug/release), `compare-qmmqnzwk` ve `compare-ogma6668` (birleşik debug/release), `verify-_e108v7f` (math) ve `verify-1hv399w4` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç `Mth` random helper overload'unun sabit seed'li sonuçlarına, takip çekişlerine ve mevcut corpus regresyonuna aittir. `Mth.nextInt/nextFloat/nextDouble`, RandomSource'un diğer implementasyonları, overflow ile geçersiz inclusive bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
