# P01.a — Mth rotationAroundAxis doğrulaması

`src/math.rs`, Java `Mth.rotationAroundAxis(Vector3fc, Quaternionf, Quaternionf)` işlemini JOML'nin f32 dot/FMA izdüşümü, eksen bileşen çarpımları ve quaternion normalizasyonu ile taşır. `examples/behavior_adapter.rs` sonucu x/y/z/w ham f32 bitleri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Unit/non-unit ve negatif eksenleri, sıfır ve işaretli sıfırı, FMA işlem sırasını, subnormal değerleri, f32 sınır taşmasını, Infinity/NaN ve sıfır quaternion normalizasyonunu kapsayan toplam **24 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1763/1763** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2265/2265** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `7e0ba63794c7ec98b0e5be4494e1fd999edd354071389371ea9497d0096242aa`. Birleşik referans kimliği: `0ad19435d5e4101420352fbb4920646f636d51c748f4cfe43f9a29d2da727cd5`.

Adapter hashleri: debug `0d3406b9f649da89947fcf975aa7e45601195191d4845847803216ab69b7b923`, release `7f8d8d0d89a26ed30abf37dc651545a2304ed76be50bbede380384e51e304d98`.

Yerel raporlar: `compare-7mgwi148` ve `compare-rq3_yvog` (math debug/release), `compare-efi7e5qh` ve `compare-qw_kmael` (birleşik debug/release), `verify-tljqi7wo` (math) ve `verify-oxhrmmem` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.rotationAroundAxis` helper'ının sabit Vector3f/Quaternionf girdileri, JOML f32 FMA ve normalizasyon sonuçları içindir. Quaternion'ın diğer işlemleri, mutable result aliasing, matrix/axis-angle dönüşümleri ve helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
