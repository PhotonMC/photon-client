# P01.a — Mth absMax doğrulaması

`src/math.rs`, Java `Mth.absMax(float, float)` ve `absMax(double, double)` yollarını her girdide Java `Math.abs`, ardından Java `Math.max` sırasıyla uygular. Mutlak değer için işaret biti temizlenir; NaN dalları Rust genel `max` davranışına bırakılmaz. `examples/behavior_adapter.rs` sonuçları raw f32/f64 bit string'i olarak taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 14, toplam **28 özgün senaryo** hazırlandı.
- Pozitif/negatif girdiler, eşit mutlak değerler, sıfır ve signed zero, minimum subnormal, maksimum sonlu değer, Infinity ve pozitif/negatif NaN girdileri karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1608/1608** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2110/2110** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `10c179a43201ead1d005dbd3e8f1deb7d1b4d5928bd228214792c84e07ef2b41`. Birleşik referans kimliği: `bb3698ca7b34e45053dd8865ed3b754080b1fb665c802937b38ccb0216ecede3`.

Adapter hashleri: debug `093dadfb72b58cdb548dc86ac07d1392b14d7d9d30e2dfc064973a213e00e8da`, release `35db30ea7c4bfc3b9d9c2e45950e6a7eefa07975b15170c80a9ade30c5eb5bbe`.

Yerel raporlar: `compare-pmcommj0` ve `compare-4g5l34ea` (math debug/release), `compare-mki3lw11` ve `compare-r66knife` (birleşik debug/release), `verify-_islsta6` (math) ve `verify-35w0iz71` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `absMax` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. Integer `absMax`, `Mth.abs(int)`, `equal`, positiveModulo, clamp/lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
