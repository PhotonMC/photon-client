# P01.a — Mth lerpInt ve lerpDiscrete doğrulaması

`src/math.rs`, Java `Mth.lerpInt(float, int, int)` ve `Mth.lerpDiscrete(float, int, int)` overload'larını i32 delta/floor ve toplama sırasıyla taşır. `examples/behavior_adapter.rs` sonuçları signed decimal i32 string'leri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her overload için temel alpha, sıfır/bir/taşan alpha, negatif extrapolation, signed zero, subnormal, NaN, Infinity, delta taşması ve i32 sınırlarını içeren toplam **28 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1653/1653** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2155/2155** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `e9b1c1e55b2a03528b712cd304101c7b36378556372ebad60bb330a4a60eb192`. Birleşik referans kimliği: `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e`.

Adapter hashleri: debug `2371e57dd5b14642fa55352a223485b414fbdeb72599f35405f7e4517da68372`, release `ce1f002fb70be2fce7ebe09258c7d8ffbc7eed67c9643ef81eeb6da2dabb4ecb`.

Yerel raporlar: `compare-ceusm4y4` ve `compare-hconpy1a` (math debug/release), `compare-rhjx0z7e` ve `compare-jtl411eu` (birleşik debug/release), `verify-biycqhmq` (math) ve `verify-u5d5zd4c` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.lerpInt` ve `Mth.lerpDiscrete` sonuçlarını sabit f32 alpha ve i32 uçlarla kapsar. Scalar f32/f64 `lerp`, Vec3 overload'u, `lerp2`, `lerp3`, `catmullrom`, `clampedLerp`, `inverseLerp` ve bu helper'ları kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
