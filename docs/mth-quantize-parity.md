# P01.a — Mth quantize doğrulaması

`src/math.rs`, Java `Mth.quantize(double, int)` yolunu f64 bölme, Java `floor` i32 daraltması ve wrapping i32 çarpımıyla uygular. `examples/behavior_adapter.rs` sonucu Java oracle ile aynı signed i32 decimal biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Pozitif/negatif kesir, negatif ve minimum çözünürlük, sıfır ve signed zero, subnormal, Infinity, NaN, i32 sınır daraltması, sıfır çözünürlük ve çarpma taşmasını kapsayan toplam **19 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1398/1398** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `4cc46837e07b2a435f9a79ffddebd09d7fed0ad74e1310e64453de08cb4bf4bd`. Birleşik referans kimliği: `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea`.

Adapter hashleri: debug `7fcaaf04b4fa67068cb95022cc7e832f9013a10846c57ad8dee850949ac64f59`, release `b98e59cbf911f59510b79a9d182e31d455c93c6084ae18d3ce74b07bab212466`.

Yerel raporlar: `compare-xljwmsb7` ve `compare-s_oqvkq3` (math debug/release), `compare-uf5y7y3a` ve `compare-wrj9a6ex` (birleşik debug/release), `verify-_l31ex7g` (math) ve `verify-5ju3ok97` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.quantize(double, int)` için sabit girdili signed i32 sonuçlarına ve mevcut corpus regresyonuna aittir. Diğer yuvarlama/ceil yardımcıları ve quantize sonucunu kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
