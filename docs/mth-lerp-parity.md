# P01.a — Mth lerp doğrulaması

`src/math.rs`, Java `Mth.lerp(float, float, float)` ve `Mth.lerp(double, double, double)` overload'larını fark, çarpma ve toplama sırasıyla taşır. `examples/behavior_adapter.rs` sonucu ilgili hassasiyetin raw IEEE-754 bitleri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- f32 ve f64 overload'ları için temel aralık, iki yönlü extrapolation, negatif/ters uçlar, eşit uçlar, signed zero, subnormal, sonlu taşma, Infinity ve NaN içeren toplam **28 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1763/1763** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2265/2265** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `7e0ba63794c7ec98b0e5be4494e1fd999edd354071389371ea9497d0096242aa`. Birleşik referans kimliği: `0ad19435d5e4101420352fbb4920646f636d51c748f4cfe43f9a29d2da727cd5`.

Adapter hashleri: debug `0d3406b9f649da89947fcf975aa7e45601195191d4845847803216ab69b7b923`, release `7f8d8d0d89a26ed30abf37dc651545a2304ed76be50bbede380384e51e304d98`.

Yerel raporlar: `compare-7mgwi148` ve `compare-rq3_yvog` (math debug/release), `compare-efi7e5qh` ve `compare-qw_kmael` (birleşik debug/release), `verify-tljqi7wo` (math) ve `verify-oxhrmmem` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt iki scalar `Mth.lerp` overload'unun sabit f32/f64 girdilerde doğrudan aritmetik sonucunu kapsar. `lerpInt`, `lerpDiscrete`, Vec3 overload'u, `lerp2`, `lerp3`, `catmullrom`, `clampedLerp`, `inverseLerp` ve bu helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
