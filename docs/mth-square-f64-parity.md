# P01.a — Mth square(double) doğrulaması

`src/math.rs`, Java `Mth.square(double)` yordamının tek f64 çarpımını taşır. `examples/behavior_adapter.rs` sonucu raw f64 bit dizgisi olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Sıfır, signed zero, negatif ve kesirli değerler, pozitif minimum subnormal girdi, subnormal sonuç, büyük ve taşan sonlu değerler, Infinity, NaN ve yuvarlama sınırı içeren **15 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1668/1668** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2170/2170** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `a025973c1ef8351cb5437b997d6b70b36792edd52713c0b5f087210263fec060`. Birleşik referans kimliği: `0ab94c7aca8ab0f093d23176b8a698767746ba84238949d19b77f4b5732d94d7`.

Adapter hashleri: debug `0feff16b39b41d5108009538a31c8ca27bae1eb27830e05f355e249e825a3c59`, release `c26e0d6d9adf0985a9f18bdf551d783dddaf31550ac373bdc7233cdebc70707f`.

Yerel raporlar: `compare-di4orhey` ve `compare-9os9l34y` (math debug/release), `compare-spny1_zm` ve `compare-bu52w5ts` (birleşik debug/release), `verify-dh3f392l` (math) ve `verify-j7u8aby_` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.square(double)` yordamının sabit raw f64 girdilerdeki sonucunu kapsar. Diğer `square` overload'ları, `cube`, `clampedMap`, `map`, diğer Mth yardımcıları ve bu yordamı kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
