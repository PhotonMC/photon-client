# P01.a — Mth lerp doğrulaması

`src/math.rs`, Java `Mth.lerp(float, float, float)` ve `Mth.lerp(double, double, double)` overload'larını fark, çarpma ve toplama sırasıyla taşır. `examples/behavior_adapter.rs` sonucu ilgili hassasiyetin raw IEEE-754 bitleri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- f32 ve f64 overload'ları için temel aralık, iki yönlü extrapolation, negatif/ters uçlar, eşit uçlar, signed zero, subnormal, sonlu taşma, Infinity ve NaN içeren toplam **28 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1683/1683** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2185/2185** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `d91d4fb976af018c6ced8eb9d475396c09c64f7ef8429742661a9ba2d8c34267`. Birleşik referans kimliği: `595049f32d6822eff3379ec8f2d5211ef192043bf7f6a11db9b9d7c609fe01a1`.

Adapter hashleri: debug `a8905dba481a438cfebc22b452d1af22f2d3f1e7de4d2d3a376e85d6141eff61`, release `b6b7d1e7f5cd800ac6971e91b29dee4b36db61aef4e537adc1de1d2f5bff5f93`.

Yerel raporlar: `compare-arjeniq4` ve `compare-_0386q8q` (math debug/release), `compare-63qzbune` ve `compare-1pb7x57b` (birleşik debug/release), `verify-wl7nytn8` (math) ve `verify-f_mau559` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt iki scalar `Mth.lerp` overload'unun sabit f32/f64 girdilerde doğrudan aritmetik sonucunu kapsar. `lerpInt`, `lerpDiscrete`, Vec3 overload'u, `lerp2`, `lerp3`, `catmullrom`, `clampedLerp`, `inverseLerp` ve bu helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
