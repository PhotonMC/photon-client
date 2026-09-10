# P01.a — Mth outFromOrigin doğrulaması

`src/math.rs`, Java `Mth.outFromOrigin` üç ve dört argümanlı overload'larının sınır doğrulamasını, origin clamp'ini ve lazy stream tüketim sırasını i32 sarmasıyla taşır. `examples/behavior_adapter.rs` aynı sonucu JSON i32 listesi veya Java hata metni olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Default ve explicit step overload'larını, origin'in alt/üst sınıra clamp edilmesini, singleton/asimetrik aralıkları, aralığı aşan adımı, i32 origin sınırlarını ve lower/upper ile step hatalarını kapsayan toplam **24 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2047/2047** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2549/2549** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `cf0843cad3317af0dc342572b3e0a3e0a9e482697ceeec73754750ceb27c15cd`. Birleşik referans kimliği: `510e1d4575f7dd92a09a6d3f896c2b8077a8fb2c01dd57ea32b9b48ce94a7583`.

Adapter hashleri: debug `794b1627f841615c5f200db08f1b8be0061ba9791a3dfa61ac439b9a2eace997`, release `986944806fde5347db60e40c7761984d390aaea2a0c4eceeeab651c2e0d30fd58`.

Yerel raporlar: `compare-j7dw7d6f` ve `compare-6na1_w6w` (math debug/release), `compare-qz7665of` ve `compare-2e2muf1s` (birleşik debug/release), `verify-e6j0lf9a` (math) ve `verify-s7mqu5wk` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.outFromOrigin` overload'larının sabit girdili, sonlanan stream sonuçlarına ve mevcut corpus regresyonuna aittir. Büyük overflow aralıkları, sonlanmayan akış olasılıkları ve bu helper'ı kullanan istemci davranışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
