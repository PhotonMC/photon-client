# P01.a — Mth positiveModulo doğrulaması

`src/math.rs`, Java `Mth.positiveModulo(float, float)` ve `positiveModulo(double, double)` yollarını aynı hassasiyette `(input % mod + mod) % mod` işlem sırasıyla uygular. `examples/behavior_adapter.rs` iki overload'u JSONL senaryolarına bağlar; sonuçlar raw f32/f64 bit string'i olarak karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 22, toplam **44 özgün senaryo** hazırlandı.
- Pozitif/negatif girdiler ve modüller, tam katlar, signed zero, f32/f64 subnormal ve maksimum sonlu girdiler, sıfır, Infinity ve NaN yolları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2076/2076** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2578/2578** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Java'nın sabitlenmiş Linux x86_64 runtime'ında invalid floating-point remainder sonuçları raw NaN işaretiyle gözlenir. Rust native remainder'ın işaretinden ayrılan bu yollar, Java çıktısına uygun canonical bitlerle korunur: sonlu input ve sonsuz modulus için negatif canonical NaN; diğer kapsanan invalid yollar için pozitif canonical NaN. Sonlu sonuçlar ve signed zero sonuçları doğrudan işlem sırasından gelir; karşılaştırıcı genel NaN normalizasyonu yapmaz.

Math referans kimliği: `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`. Birleşik referans kimliği: `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8`.

Adapter hashleri: debug `6eb1e772a2e06c20b8d0c09ba0b2a11b5f1df7a671ee8d1513127bc318d143ef`, release `318c5ef22a8d56277aa19f9da9f5becca9f575904eba79756b959bdc8c993fe88`.

Yerel raporlar: `compare-k2v65l83` ve `compare-9uf1gox2` (math debug/release), `compare-ptlm9ptr` ve `compare-86z18x_k` (birleşik debug/release), `verify-9dtf6rea` (math) ve `verify-31k8bjcu` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `positiveModulo` overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. `Mth.equal`, `absMax`, `clampedLerp` ve diğer floating-point yardımcıları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
