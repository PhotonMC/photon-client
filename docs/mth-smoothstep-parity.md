# P01.a — Mth smoothstep doğrulaması

`src/math.rs`, Java `Mth.smoothstep(double)` ve `Mth.smoothstepDerivative(double)` ifadelerini kaynak işlem sırasıyla taşır. `examples/behavior_adapter.rs` sonuçları raw f64 bit string'leri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her yordam için sıfır/bir, aralık içi ve dışı değerler, signed zero, en küçük subnormal, en büyük sonlu değer, Infinity, NaN ve 1.0 komşularını içeren toplam **28 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2166/2166** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2668/2668** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt iki `Mth` yordamının sabit raw f64 girdilerde doğrudan aritmetik sonucunu kapsar. `catmullrom`, `sign`, diğer interpolasyon yardımcıları ve bu yordamları kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
