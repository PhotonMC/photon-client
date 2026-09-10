# P01.a — Mth inverseLerp doğrulaması

`src/math.rs`, Java `Mth.inverseLerp(float, float, float)` ve `Mth.inverseLerp(double, double, double)` overload'larını pay/payda çıkarma ve bölme sırasıyla taşır. `examples/behavior_adapter.rs` sonucu ilgili hassasiyetin raw IEEE-754 bitleri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- f32 ve f64 overload'ları için temel aralık, aralık dışı faktör, negatif/ters sınır, tam sınır, signed zero, sıfır aralık, subnormal, sonlu taşma, Infinity ve NaN içeren toplam **28 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1608/1608** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2110/2110** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `10c179a43201ead1d005dbd3e8f1deb7d1b4d5928bd228214792c84e07ef2b41`. Birleşik referans kimliği: `bb3698ca7b34e45053dd8865ed3b754080b1fb665c802937b38ccb0216ecede3`.

Adapter hashleri: debug `093dadfb72b58cdb548dc86ac07d1392b14d7d9d30e2dfc064973a213e00e8da`, release `35db30ea7c4bfc3b9d9c2e45950e6a7eefa07975b15170c80a9ade30c5eb5bbe`.

Yerel raporlar: `compare-pmcommj0` ve `compare-4g5l34ea` (math debug/release), `compare-mki3lw11` ve `compare-r66knife` (birleşik debug/release), `verify-_islsta6` (math) ve `verify-35w0iz71` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt iki `Mth.inverseLerp` overload'unun sabit f32/f64 girdilerde doğrudan aritmetik sonucunu kapsar. `lerp`, `clampedLerp`, `map`/`clampedMap`, vektör overload'ları ve bu helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
