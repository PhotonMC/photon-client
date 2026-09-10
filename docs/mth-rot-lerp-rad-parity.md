# P01.a — Mth rotLerpRad doğrulaması

`src/math.rs`, Java `Mth.rotLerpRad(float, float, float)` yordamının f32 fark çıkarma, π sınırlarında tekrarlı açı sarmalama ve `from + alpha * fark` işlem sırasını taşır. `examples/behavior_adapter.rs` sonucu raw f32 bit dizgesi olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Temel interpolation, extrapolation, ±π sınırı, açı geçişi, tam tur, signed zero, subnormal, geniş sonlu fark ve NaN içeren **15 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1668/1668** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2170/2170** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `a025973c1ef8351cb5437b997d6b70b36792edd52713c0b5f087210263fec060`. Birleşik referans kimliği: `0ab94c7aca8ab0f093d23176b8a698767746ba84238949d19b77f4b5732d94d7`.

Adapter hashleri: debug `0feff16b39b41d5108009538a31c8ca27bae1eb27830e05f355e249e825a3c59`, release `c26e0d6d9adf0985a9f18bdf551d783dddaf31550ac373bdc7233cdebc70707f`.

Yerel raporlar: `compare-di4orhey` ve `compare-9os9l34y` (math debug/release), `compare-spny1_zm` ve `compare-bu52w5ts` (birleşik debug/release), `verify-dh3f392l` (math) ve `verify-j7u8aby_` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.rotLerpRad` yordamının sabit raw f32 girdilerdeki sonucunu kapsar. `rotLerp`, `triangleWave`, `square`, diğer açı yardımcıları ve `rotLerpRad` kullanan istemci akışları ayrı dilimlerdir. Java döngülerinin sonlanmadığı Infinity farkları senaryo kapsamı dışındadır. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
