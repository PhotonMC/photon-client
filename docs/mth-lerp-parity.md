# P01.a — Mth lerp doğrulaması

`src/math.rs`, Java `Mth.lerp(float, float, float)` ve `Mth.lerp(double, double, double)` overload'larını fark, çarpma ve toplama sırasıyla taşır. `examples/behavior_adapter.rs` sonucu ilgili hassasiyetin raw IEEE-754 bitleri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- f32 ve f64 overload'ları için temel aralık, iki yönlü extrapolation, negatif/ters uçlar, eşit uçlar, signed zero, subnormal, sonlu taşma, Infinity ve NaN içeren toplam **28 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1843/1843** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `dfeb55b1b61cbc7649df7ae8af88469cbf7480a194892aacbcf8e5e5b93806bd`. Birleşik referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Adapter hashleri: debug `c3f51b7830e3c0554138ff428a979a8f54ffaf85f7e2082bc6962bdcf01ff1d9`, release `fa350ee97911ed79b9b6db86a8af51e8b7dc87401e8312957b94741266b4b51b`.

Yerel raporlar: `compare-5_avciij` ve `compare-e6e6agr4` (math debug/release), `compare-gvo3eind` ve `compare-wmabbl5t` (birleşik debug/release), `verify-zbtpetmh` (math) ve `verify-vzenwlxd` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt iki scalar `Mth.lerp` overload'unun sabit f32/f64 girdilerde doğrudan aritmetik sonucunu kapsar. `lerpInt`, `lerpDiscrete`, Vec3 overload'u, `lerp2`, `lerp3`, `catmullrom`, `clampedLerp`, `inverseLerp` ve bu helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
