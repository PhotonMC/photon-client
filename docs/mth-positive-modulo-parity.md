# P01.a — Mth positiveModulo doğrulaması

`src/math.rs`, Java `Mth.positiveModulo(float, float)` ve `positiveModulo(double, double)` yollarını aynı hassasiyette `(input % mod + mod) % mod` işlem sırasıyla uygular. `examples/behavior_adapter.rs` iki overload'u JSONL senaryolarına bağlar; sonuçlar raw f32/f64 bit string'i olarak karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 22, toplam **44 özgün senaryo** hazırlandı.
- Pozitif/negatif girdiler ve modüller, tam katlar, signed zero, f32/f64 subnormal ve maksimum sonlu girdiler, sıfır, Infinity ve NaN yolları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1450/1450** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Java'nın sabitlenmiş Linux x86_64 runtime'ında invalid floating-point remainder sonuçları raw NaN işaretiyle gözlenir. Rust native remainder'ın işaretinden ayrılan bu yollar, Java çıktısına uygun canonical bitlerle korunur: sonlu input ve sonsuz modulus için negatif canonical NaN; diğer kapsanan invalid yollar için pozitif canonical NaN. Sonlu sonuçlar ve signed zero sonuçları doğrudan işlem sırasından gelir; karşılaştırıcı genel NaN normalizasyonu yapmaz.

Math referans kimliği: `fb79a016fcf5041a8631f665991eaad422d1346ef0c1c0c12f2166cdb0057846`. Birleşik referans kimliği: `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11`.

Adapter hashleri: debug `1a871a434c144a587a2723327bd46aaf9a6d1b4612396a66fffe175369c31d12`, release `8c7871587c66a581ea4b33d57a1ef23281843488a7661885a1a704dee0ad3e74`.

Yerel raporlar: `compare-qtqxwt9b` ve `compare-ebava42g` (math debug/release), `compare-x0hkkvkv` ve `compare-x5wgwswa` (birleşik debug/release), `verify-m2eek8to` (math) ve `verify-rc21vyy9` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `positiveModulo` overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.equal`, `absMax`, `clampedLerp` ve diğer floating-point yardımcıları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
