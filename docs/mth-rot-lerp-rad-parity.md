# P01.a — Mth rotLerpRad doğrulaması

`src/math.rs`, Java `Mth.rotLerpRad(float, float, float)` yordamının f32 fark çıkarma, π sınırlarında tekrarlı açı sarmalama ve `from + alpha * fark` işlem sırasını taşır. `examples/behavior_adapter.rs` sonucu raw f32 bit dizgesi olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Temel interpolation, extrapolation, ±π sınırı, açı geçişi, tam tur, signed zero, subnormal, geniş sonlu fark ve NaN içeren **15 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1653/1653** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2155/2155** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `e9b1c1e55b2a03528b712cd304101c7b36378556372ebad60bb330a4a60eb192`. Birleşik referans kimliği: `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e`.

Adapter hashleri: debug `2371e57dd5b14642fa55352a223485b414fbdeb72599f35405f7e4517da68372`, release `ce1f002fb70be2fce7ebe09258c7d8ffbc7eed67c9643ef81eeb6da2dabb4ecb`.

Yerel raporlar: `compare-ceusm4y4` ve `compare-hconpy1a` (math debug/release), `compare-rhjx0z7e` ve `compare-jtl411eu` (birleşik debug/release), `verify-biycqhmq` (math) ve `verify-u5d5zd4c` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.rotLerpRad` yordamının sabit raw f32 girdilerdeki sonucunu kapsar. `rotLerp`, `triangleWave`, `square`, diğer açı yardımcıları ve `rotLerpRad` kullanan istemci akışları ayrı dilimlerdir. Java döngülerinin sonlanmadığı Infinity farkları senaryo kapsamı dışındadır. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
