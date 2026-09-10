# P01.a — Mth smoothstep doğrulaması

`src/math.rs`, Java `Mth.smoothstep(double)` ve `Mth.smoothstepDerivative(double)` ifadelerini kaynak işlem sırasıyla taşır. `examples/behavior_adapter.rs` sonuçları raw f64 bit string'leri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her yordam için sıfır/bir, aralık içi ve dışı değerler, signed zero, en küçük subnormal, en büyük sonlu değer, Infinity, NaN ve 1.0 komşularını içeren toplam **28 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1743/1743** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2245/2245** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `6b06e5ddd2219dc868e358d02f9afcfbda6469cddd7391f150f9280440124f6a`. Birleşik referans kimliği: `eb48e23a1245d82fe150fe609e00283d8fa5a86dbd72ebeb887719a74df99968`.

Adapter hashleri: debug `65b6a51802401fd8e16a2ce346287fded6541bdbb762922fa5ed76876f2007f4`, release `99b722f883332e4576e669f7db57f98dac937e328b3a669185056e37f8dec786`.

Yerel raporlar: `compare-q3ryrs_k` ve `compare-xtfhknri` (math debug/release), `compare-ag5oxl30` ve `compare-fkrk2srf` (birleşik debug/release), `verify-hvezlnv1` (math) ve `verify-lstfjdye` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt iki `Mth` yordamının sabit raw f64 girdilerde doğrudan aritmetik sonucunu kapsar. `catmullrom`, `sign`, diğer interpolasyon yardımcıları ve bu yordamları kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
