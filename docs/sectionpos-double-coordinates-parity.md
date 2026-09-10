# P01.c — SectionPos double koordinat doğrulaması

`src/binary.rs`, SectionPos `posToSectionCoord(double)` ve `blockToSectionCoord(double)` overload'larını Java'nın floor, narrowing cast ve arithmetic shift sonuçlarıyla eşleştirir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos double koordinatları için 6 özgün senaryo hazırlandı.
- İki overload; pozitif/negatif sınırlar, subnormal, en büyük sonlu değer ve NaN girdileriyle doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2047/2047** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2549/2549** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `cf0843cad3317af0dc342572b3e0a3e0a9e482697ceeec73754750ceb27c15cd`. Birleşik referans kimliği: `510e1d4575f7dd92a09a6d3f896c2b8077a8fb2c01dd57ea32b9b48ce94a7583`.

Adapter hashleri: debug `794b1627f841615c5f200db08f1b8be0061ba9791a3dfa61ac439b9a2eace997`, release `986944806fde5347db60e40c7761984d390aaea2a0c4eceeeab651c2e0d30fd58`.

Yerel raporlar: `compare-j7dw7d6f` (math debug), `compare-6na1_w6w` (math release), `compare-qz7665of` (birleşik debug), `compare-2e2muf1s` (birleşik release), `verify-e6j0lf9a` (math) ve `verify-s7mqu5wk` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, iki SectionPos double overload'unun altı son int çıktısıyla sınırlıdır. `Mth.floor(float)`, `Mth.lfloor(double)`, diğer SectionPos overload'ları ve gerçek istemci kullanım zincirleri ayrı kapsamdır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
