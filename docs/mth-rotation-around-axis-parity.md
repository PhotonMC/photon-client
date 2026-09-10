# P01.a — Mth rotationAroundAxis doğrulaması

`src/math.rs`, Java `Mth.rotationAroundAxis(Vector3fc, Quaternionf, Quaternionf)` işlemini JOML'nin f32 dot/FMA izdüşümü, eksen bileşen çarpımları ve quaternion normalizasyonu ile taşır. `examples/behavior_adapter.rs` sonucu x/y/z/w ham f32 bitleri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Unit/non-unit ve negatif eksenleri, sıfır ve işaretli sıfırı, FMA işlem sırasını, subnormal değerleri, f32 sınır taşmasını, Infinity/NaN ve sıfır quaternion normalizasyonunu kapsayan toplam **24 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1638/1638** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2140/2140** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `18248ddafaffd46fa827971d82704febaf2aa40a94cc0b653078329db1f4cacf`. Birleşik referans kimliği: `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f`.

Adapter hashleri: debug `4a9c082a05009b8ac6c37d12ee8814942a447b8e2e3a445f3ed95942b6a8356e`, release `750a8467b38e832bff19cfcaa43a5a4f94653270c98fd4a4d3f46248f0665f5d`.

Yerel raporlar: `compare-ep8fiu6` ve `compare-z9iwcisf` (math debug/release), `compare-a21c2t4m` ve `compare-uwg2_ewv` (birleşik debug/release), `verify-8mq2_8pm` (math) ve `verify-yscj0_fq` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.rotationAroundAxis` helper'ının sabit Vector3f/Quaternionf girdileri, JOML f32 FMA ve normalizasyon sonuçları içindir. Quaternion'ın diğer işlemleri, mutable result aliasing, matrix/axis-angle dönüşümleri ve helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
