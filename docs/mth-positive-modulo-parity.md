# P01.a — Mth positiveModulo doğrulaması

`src/math.rs`, Java `Mth.positiveModulo(float, float)` ve `positiveModulo(double, double)` yollarını aynı hassasiyette `(input % mod + mod) % mod` işlem sırasıyla uygular. `examples/behavior_adapter.rs` iki overload'u JSONL senaryolarına bağlar; sonuçlar raw f32/f64 bit string'i olarak karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 22, toplam **44 özgün senaryo** hazırlandı.
- Pozitif/negatif girdiler ve modüller, tam katlar, signed zero, f32/f64 subnormal ve maksimum sonlu girdiler, sıfır, Infinity ve NaN yolları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1743/1743** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2245/2245** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Java'nın sabitlenmiş Linux x86_64 runtime'ında invalid floating-point remainder sonuçları raw NaN işaretiyle gözlenir. Rust native remainder'ın işaretinden ayrılan bu yollar, Java çıktısına uygun canonical bitlerle korunur: sonlu input ve sonsuz modulus için negatif canonical NaN; diğer kapsanan invalid yollar için pozitif canonical NaN. Sonlu sonuçlar ve signed zero sonuçları doğrudan işlem sırasından gelir; karşılaştırıcı genel NaN normalizasyonu yapmaz.

Math referans kimliği: `6b06e5ddd2219dc868e358d02f9afcfbda6469cddd7391f150f9280440124f6a`. Birleşik referans kimliği: `eb48e23a1245d82fe150fe609e00283d8fa5a86dbd72ebeb887719a74df99968`.

Adapter hashleri: debug `65b6a51802401fd8e16a2ce346287fded6541bdbb762922fa5ed76876f2007f4`, release `99b722f883332e4576e669f7db57f98dac937e328b3a669185056e37f8dec786`.

Yerel raporlar: `compare-q3ryrs_k` ve `compare-xtfhknri` (math debug/release), `compare-ag5oxl30` ve `compare-fkrk2srf` (birleşik debug/release), `verify-hvezlnv1` (math) ve `verify-lstfjdye` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `positiveModulo` overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.equal`, `absMax`, `clampedLerp` ve diğer floating-point yardımcıları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
