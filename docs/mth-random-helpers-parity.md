# P01.a — Mth random helpers doğrulaması

`src/math.rs`, Java `Mth.randomBetweenInclusive(RandomSource, int, int)`, `randomBetween(RandomSource, float, float)` ve `normal(RandomSource, float, float)` yollarını uygular. `examples/behavior_adapter.rs` helper sonucunu ve kontrollü takip çekişini Java oracle ile aynı JSON bit/değer biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 6 inclusive int, 6 f32 aralık ve 6 normal olmak üzere toplam **18 özgün senaryo** hazırlandı.
- Inclusive int için eşit sınırda `nextInt(1)` tüketimi, pozitif/negatif i32 uçları ve geniş geçerli aralıklar; f32 aralık için ters/eşit sınırlar, signed zero, Infinity ve NaN; normal için mean/deviation, signed zero, Infinity ve negatif deviation yolları karşılaştırıldı.
- Her helper sonucundan sonra kontrollü bir takip çekişi yapıldı; normal yolunda takip Gaussian değeri cache davranışını da doğruladı.
- Rust debug ve release adaptörleri güncel **2023/2023** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2525/2525** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `1a217debf64171ea8f99e220ffd6aca1130ecceef3e6015b82f0c12a4e3f9f0d`. Birleşik referans kimliği: `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5`.

Adapter hashleri: debug `0f53bfe9b4fa3f51b5f1cac8229fea64a4d485c5c5fae4f8b5607a300ecb94e8`, release `61ae7de80355a2cf73c393fbd30be18b82042d583a1918baf991fa8a22629e888`.

Yerel raporlar: `compare-4sx1lm2m` ve `compare-m3ib6sjn` (math debug/release), `compare-czvhou3w` ve `compare-oz19j96e` (birleşik debug/release), `verify-xjnq5poq` (math) ve `verify-nukwd4yc` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç `Mth` random helper overload'unun sabit seed'li sonuçlarına, takip çekişlerine ve mevcut corpus regresyonuna aittir. `Mth.nextInt/nextFloat/nextDouble`, RandomSource'un diğer implementasyonları, overflow ile geçersiz inclusive bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
