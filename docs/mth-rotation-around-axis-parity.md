# P01.a — Mth rotationAroundAxis doğrulaması

`src/math.rs`, Java `Mth.rotationAroundAxis(Vector3fc, Quaternionf, Quaternionf)` işlemini JOML'nin f32 dot/FMA izdüşümü, eksen bileşen çarpımları ve quaternion normalizasyonu ile taşır. `examples/behavior_adapter.rs` sonucu x/y/z/w ham f32 bitleri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Unit/non-unit ve negatif eksenleri, sıfır ve işaretli sıfırı, FMA işlem sırasını, subnormal değerleri, f32 sınır taşmasını, Infinity/NaN ve sıfır quaternion normalizasyonunu kapsayan toplam **24 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1450/1450** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `fb79a016fcf5041a8631f665991eaad422d1346ef0c1c0c12f2166cdb0057846`. Birleşik referans kimliği: `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11`.

Adapter hashleri: debug `1a871a434c144a587a2723327bd46aaf9a6d1b4612396a66fffe175369c31d12`, release `8c7871587c66a581ea4b33d57a1ef23281843488a7661885a1a704dee0ad3e74`.

Yerel raporlar: `compare-qtqxwt9b` ve `compare-ebava42g` (math debug/release), `compare-x0hkkvkv` ve `compare-x5wgwswa` (birleşik debug/release), `verify-m2eek8to` (math) ve `verify-rc21vyy9` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.rotationAroundAxis` helper'ının sabit Vector3f/Quaternionf girdileri, JOML f32 FMA ve normalizasyon sonuçları içindir. Quaternion'ın diğer işlemleri, mutable result aliasing, matrix/axis-angle dönüşümleri ve helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
