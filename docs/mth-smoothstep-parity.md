# P01.a — Mth smoothstep doğrulaması

`src/math.rs`, Java `Mth.smoothstep(double)` ve `Mth.smoothstepDerivative(double)` ifadelerini kaynak işlem sırasıyla taşır. `examples/behavior_adapter.rs` sonuçları raw f64 bit string'leri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her yordam için sıfır/bir, aralık içi ve dışı değerler, signed zero, en küçük subnormal, en büyük sonlu değer, Infinity, NaN ve 1.0 komşularını içeren toplam **28 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1783/1783** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2285/2285** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `c9c0c64d34c86e78a480fb2f63bd3bd5633a857332ad5997d97039c193be5338`. Birleşik referans kimliği: `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90`.

Adapter hashleri: debug `843cc0c04c00803acc1e8bf774f290df5e273c7777789a8df5b971cb6a7f1ec0`, release `200c25d47d60d5c096012435e86b4b9f28f06965fa1ac6a453b948b2e6e33f4e`.

Yerel raporlar: `compare-1g27oorn` ve `compare-bkcvw7oi` (math debug/release), `compare-qmmqnzwk` ve `compare-ogma6668` (birleşik debug/release), `verify-_e108v7f` (math) ve `verify-1hv399w4` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt iki `Mth` yordamının sabit raw f64 girdilerde doğrudan aritmetik sonucunu kapsar. `catmullrom`, `sign`, diğer interpolasyon yardımcıları ve bu yordamları kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
