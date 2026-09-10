# P01.a — Mth rotLerpRad doğrulaması

`src/math.rs`, Java `Mth.rotLerpRad(float, float, float)` yordamının f32 fark çıkarma, π sınırlarında tekrarlı açı sarmalama ve `from + alpha * fark` işlem sırasını taşır. `examples/behavior_adapter.rs` sonucu raw f32 bit dizgesi olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Temel interpolation, extrapolation, ±π sınırı, açı geçişi, tam tur, signed zero, subnormal, geniş sonlu fark ve NaN içeren **15 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1593/1593** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2095/2095** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `ec7f2431c2fc0a873c374068f226b4309821be58380c7e5eadccdc15fe690723`. Birleşik referans kimliği: `17d40db8607a5df22fc08a42ccd0065b6ffe818cef98507474d8663297c0ac04`.

Adapter hashleri: debug `10f1cebf91a6374fb899fea510944d01aebbdc72d1dd134cb8cf090110794e99`, release `72a957c0b141e6a151fce06d38b47a42ba85f1aa0fa123a3001a10dc58c77b79`.

Yerel raporlar: `compare-gqgspijr` ve `compare-z8zpkwnx` (math debug/release), `compare-94x3ynna` ve `compare-2cii6gte` (birleşik debug/release), `verify-7bmsjnn9` (math) ve `verify-lu6dvbz4` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.rotLerpRad` yordamının sabit raw f32 girdilerdeki sonucunu kapsar. `rotLerp`, `triangleWave`, `square`, diğer açı yardımcıları ve `rotLerpRad` kullanan istemci akışları ayrı dilimlerdir. Java döngülerinin sonlanmadığı Infinity farkları senaryo kapsamı dışındadır. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
