# P01.a — Mth quantize doğrulaması

`src/math.rs`, Java `Mth.quantize(double, int)` yolunu f64 bölme, Java `floor` i32 daraltması ve wrapping i32 çarpımıyla uygular. `examples/behavior_adapter.rs` sonucu Java oracle ile aynı signed i32 decimal biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Pozitif/negatif kesir, negatif ve minimum çözünürlük, sıfır ve signed zero, subnormal, Infinity, NaN, i32 sınır daraltması, sıfır çözünürlük ve çarpma taşmasını kapsayan toplam **19 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1578/1578** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2080/2080** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `845bee8faa9bd67215b4c4a330fee9163ef37c01793940d730ec7655ee66c654`. Birleşik referans kimliği: `e1fbdb47e2227938261f4e5bd3238926120301582d50b1af7c84fefac9844a94`.

Adapter hashleri: debug `52aca32c3cc517ab19166f516a9e8edbb6919ddbd9ecf9c522cbbbeedb57a99e`, release `37157540d5781ee33ba26f55c955893baf917f72beac442340eecd884072c5c0`.

Yerel raporlar: `compare-l4cf39d7` ve `compare-jvzkn11s` (math debug/release), `compare-oc9sa8gq` ve `compare-m9ue0q18` (birleşik debug/release), `verify-lwzx0wr7` (math) ve `verify-np1asir2` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.quantize(double, int)` için sabit girdili signed i32 sonuçlarına ve mevcut corpus regresyonuna aittir. Diğer yuvarlama/ceil yardımcıları ve quantize sonucunu kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
