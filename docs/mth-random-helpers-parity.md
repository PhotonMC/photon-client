# P01.a — Mth random helpers doğrulaması

`src/math.rs`, Java `Mth.randomBetweenInclusive(RandomSource, int, int)`, `randomBetween(RandomSource, float, float)` ve `normal(RandomSource, float, float)` yollarını uygular. `examples/behavior_adapter.rs` helper sonucunu ve kontrollü takip çekişini Java oracle ile aynı JSON bit/değer biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 6 inclusive int, 6 f32 aralık ve 6 normal olmak üzere toplam **18 özgün senaryo** hazırlandı.
- Inclusive int için eşit sınırda `nextInt(1)` tüketimi, pozitif/negatif i32 uçları ve geniş geçerli aralıklar; f32 aralık için ters/eşit sınırlar, signed zero, Infinity ve NaN; normal için mean/deviation, signed zero, Infinity ve negatif deviation yolları karşılaştırıldı.
- Her helper sonucundan sonra kontrollü bir takip çekişi yapıldı; normal yolunda takip Gaussian değeri cache davranışını da doğruladı.
- Rust debug ve release adaptörleri güncel **1763/1763** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2265/2265** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `7e0ba63794c7ec98b0e5be4494e1fd999edd354071389371ea9497d0096242aa`. Birleşik referans kimliği: `0ad19435d5e4101420352fbb4920646f636d51c748f4cfe43f9a29d2da727cd5`.

Adapter hashleri: debug `0d3406b9f649da89947fcf975aa7e45601195191d4845847803216ab69b7b923`, release `7f8d8d0d89a26ed30abf37dc651545a2304ed76be50bbede380384e51e304d98`.

Yerel raporlar: `compare-7mgwi148` ve `compare-rq3_yvog` (math debug/release), `compare-efi7e5qh` ve `compare-qw_kmael` (birleşik debug/release), `verify-tljqi7wo` (math) ve `verify-oxhrmmem` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç `Mth` random helper overload'unun sabit seed'li sonuçlarına, takip çekişlerine ve mevcut corpus regresyonuna aittir. `Mth.nextInt/nextFloat/nextDouble`, RandomSource'un diğer implementasyonları, overflow ile geçersiz inclusive bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
