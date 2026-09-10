# P01.a — Mth square(double) doğrulaması

`src/math.rs`, Java `Mth.square(double)` yordamının tek f64 çarpımını taşır. `examples/behavior_adapter.rs` sonucu raw f64 bit dizgisi olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Sıfır, signed zero, negatif ve kesirli değerler, pozitif minimum subnormal girdi, subnormal sonuç, büyük ve taşan sonlu değerler, Infinity, NaN ve yuvarlama sınırı içeren **15 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1963/1963** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2465/2465** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `461f1ba035270ce58c69f06fec7167bb0517cd22aed6c78c25e0b076684b28fa`. Birleşik referans kimliği: `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573`.

Adapter hashleri: debug `30478caf53747391ac6513072beabe17af9a76637c23cc360c7a826700433a54`, release `0b3f2c6bf91361cff411ced1b36b1433ec25e47409a6b2623e9c0624e348115d`.

Yerel raporlar: `compare-hiau72j0` ve `compare-bttpme0o` (math debug/release), `compare-k17hah5m` ve `compare-hiiz6u35` (birleşik debug/release), `verify-cnbvzo8w` (math) ve `verify-82qi_x56` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.square(double)` yordamının sabit raw f64 girdilerdeki sonucunu kapsar. Diğer `square` overload'ları, `cube`, `clampedMap`, `map`, diğer Mth yardımcıları ve bu yordamı kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
