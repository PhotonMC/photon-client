# P01.a — Mth next random doğrulaması

`src/math.rs`, Java `Mth.nextInt(RandomSource, int, int)`, `nextFloat` ve `nextDouble` overload'larını uygular. `examples/behavior_adapter.rs` sabit seed ile oluşturulan `RandomSource` değerlerini Java oracle ile aynı JSON bit/değer biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 6 int, 7 f32 ve 7 f64 olmak üzere toplam **20 özgün senaryo** hazırlandı.
- Int için normal, eşit ve ters sınırlar, pozitif/negatif i32 uçları ve tek bitlik aralık; kayan nokta için eşit/ters sınırlar, signed zero, geniş sonlu aralık, Infinity ve NaN dalları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1422/1422** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1924/1924** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `cc96d24cc80c682020b8b8d28795f7b434dad5677010e1608fc1b6e2a730e5d7`. Birleşik referans kimliği: `a346583dc8bd7a5d672fe737c86d9febc8a2861cc99d0c486416f41813870b7c`.

Adapter hashleri: debug `4e30afd61f45820df0f57541587fa8df456d9a1374ec71795c01f465ec26de87`, release `73c1c9ea2bef7322d279fc4f84d077a2009745e23f36ce680884b73fe11b7d45`.

Yerel raporlar: `compare-ox0b8e23` ve `compare-8yyrdrg5` (math debug/release), `compare-xbim8rgs` ve `compare-kg978z18` (birleşik debug/release), `verify-p7_v_2m3` (math) ve `verify-tw8nqf_h` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç `Mth.next*` overload'unun sabit seed'li sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.randomBetweenInclusive`, `randomBetween`, `normal`, RandomSource'un diğer implementasyonları, overflow ile geçersiz bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
