# P01.a — Mth next random doğrulaması

`src/math.rs`, Java `Mth.nextInt(RandomSource, int, int)`, `nextFloat` ve `nextDouble` overload'larını uygular. `examples/behavior_adapter.rs` sabit seed ile oluşturulan `RandomSource` değerlerini Java oracle ile aynı JSON bit/değer biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 6 int, 7 f32 ve 7 f64 olmak üzere toplam **20 özgün senaryo** hazırlandı.
- Int için normal, eşit ve ters sınırlar, pozitif/negatif i32 uçları ve tek bitlik aralık; kayan nokta için eşit/ters sınırlar, signed zero, geniş sonlu aralık, Infinity ve NaN dalları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2076/2076** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2578/2578** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`. Birleşik referans kimliği: `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8`.

Adapter hashleri: debug `6eb1e772a2e06c20b8d0c09ba0b2a11b5f1df7a671ee8d1513127bc318d143ef`, release `318c5ef22a8d56277aa19f9da9f5becca9f575904eba79756b959bdc8c993fe88`.

Yerel raporlar: `compare-k2v65l83` ve `compare-9uf1gox2` (math debug/release), `compare-ptlm9ptr` ve `compare-86z18x_k` (birleşik debug/release), `verify-9dtf6rea` (math) ve `verify-31k8bjcu` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç `Mth.next*` overload'unun sabit seed'li sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.randomBetweenInclusive`, `randomBetween`, `normal`, RandomSource'un diğer implementasyonları, overflow ile geçersiz bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
