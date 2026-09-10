# P01.a — Mth quantize doğrulaması

`src/math.rs`, Java `Mth.quantize(double, int)` yolunu f64 bölme, Java `floor` i32 daraltması ve wrapping i32 çarpımıyla uygular. `examples/behavior_adapter.rs` sonucu Java oracle ile aynı signed i32 decimal biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Pozitif/negatif kesir, negatif ve minimum çözünürlük, sıfır ve signed zero, subnormal, Infinity, NaN, i32 sınır daraltması, sıfır çözünürlük ve çarpma taşmasını kapsayan toplam **19 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1608/1608** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2110/2110** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `10c179a43201ead1d005dbd3e8f1deb7d1b4d5928bd228214792c84e07ef2b41`. Birleşik referans kimliği: `bb3698ca7b34e45053dd8865ed3b754080b1fb665c802937b38ccb0216ecede3`.

Adapter hashleri: debug `093dadfb72b58cdb548dc86ac07d1392b14d7d9d30e2dfc064973a213e00e8da`, release `35db30ea7c4bfc3b9d9c2e45950e6a7eefa07975b15170c80a9ade30c5eb5bbe`.

Yerel raporlar: `compare-pmcommj0` ve `compare-4g5l34ea` (math debug/release), `compare-mki3lw11` ve `compare-r66knife` (birleşik debug/release), `verify-_islsta6` (math) ve `verify-35w0iz71` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.quantize(double, int)` için sabit girdili signed i32 sonuçlarına ve mevcut corpus regresyonuna aittir. Diğer yuvarlama/ceil yardımcıları ve quantize sonucunu kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
