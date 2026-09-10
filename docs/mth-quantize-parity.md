# P01.a — Mth quantize doğrulaması

`src/math.rs`, Java `Mth.quantize(double, int)` yolunu f64 bölme, Java `floor` i32 daraltması ve wrapping i32 çarpımıyla uygular. `examples/behavior_adapter.rs` sonucu Java oracle ile aynı signed i32 decimal biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Pozitif/negatif kesir, negatif ve minimum çözünürlük, sıfır ve signed zero, subnormal, Infinity, NaN, i32 sınır daraltması, sıfır çözünürlük ve çarpma taşmasını kapsayan toplam **19 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1548/1548** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2050/2050** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `8d3d3f4690e190086d874fa8e58bdb837efcc4649b07170de736639c38e7aa7d`. Birleşik referans kimliği: `0d4c76950a37451c226bb147399cfa08349c81c96e4a8ea2b85e47696f0caf11`.

Adapter hashleri: debug `5a18309c0cb6402cc39c0f09e9434461f609f7c57df7f6ec9deeae5bc6138fb0`, release `a7ce3c141fe387a5aa243e759a9ff22d3798bd7e7f3f57b404f68fecc66444f4`.

Yerel raporlar: `compare-m8bramo3` ve `compare-20kwpz7m` (math debug/release), `compare-hmu92wq0` ve `compare-ha45dyno` (birleşik debug/release), `verify-mzhe5jke` (math) ve `verify-46sjfnqt` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.quantize(double, int)` için sabit girdili signed i32 sonuçlarına ve mevcut corpus regresyonuna aittir. Diğer yuvarlama/ceil yardımcıları ve quantize sonucunu kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
