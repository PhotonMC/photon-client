# P01.a — Mth random helpers doğrulaması

`src/math.rs`, Java `Mth.randomBetweenInclusive(RandomSource, int, int)`, `randomBetween(RandomSource, float, float)` ve `normal(RandomSource, float, float)` yollarını uygular. `examples/behavior_adapter.rs` helper sonucunu ve kontrollü takip çekişini Java oracle ile aynı JSON bit/değer biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 6 inclusive int, 6 f32 aralık ve 6 normal olmak üzere toplam **18 özgün senaryo** hazırlandı.
- Inclusive int için eşit sınırda `nextInt(1)` tüketimi, pozitif/negatif i32 uçları ve geniş geçerli aralıklar; f32 aralık için ters/eşit sınırlar, signed zero, Infinity ve NaN; normal için mean/deviation, signed zero, Infinity ve negatif deviation yolları karşılaştırıldı.
- Her helper sonucundan sonra kontrollü bir takip çekişi yapıldı; normal yolunda takip Gaussian değeri cache davranışını da doğruladı.
- Rust debug ve release adaptörleri güncel **1608/1608** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2110/2110** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `10c179a43201ead1d005dbd3e8f1deb7d1b4d5928bd228214792c84e07ef2b41`. Birleşik referans kimliği: `bb3698ca7b34e45053dd8865ed3b754080b1fb665c802937b38ccb0216ecede3`.

Adapter hashleri: debug `093dadfb72b58cdb548dc86ac07d1392b14d7d9d30e2dfc064973a213e00e8da`, release `35db30ea7c4bfc3b9d9c2e45950e6a7eefa07975b15170c80a9ade30c5eb5bbe`.

Yerel raporlar: `compare-pmcommj0` ve `compare-4g5l34ea` (math debug/release), `compare-mki3lw11` ve `compare-r66knife` (birleşik debug/release), `verify-_islsta6` (math) ve `verify-35w0iz71` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç `Mth` random helper overload'unun sabit seed'li sonuçlarına, takip çekişlerine ve mevcut corpus regresyonuna aittir. `Mth.nextInt/nextFloat/nextDouble`, RandomSource'un diğer implementasyonları, overflow ile geçersiz inclusive bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
