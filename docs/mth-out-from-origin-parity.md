# P01.a — Mth outFromOrigin doğrulaması

`src/math.rs`, Java `Mth.outFromOrigin` üç ve dört argümanlı overload'larının sınır doğrulamasını, origin clamp'ini ve lazy stream tüketim sırasını i32 sarmasıyla taşır. `examples/behavior_adapter.rs` aynı sonucu JSON i32 listesi veya Java hata metni olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Default ve explicit step overload'larını, origin'in alt/üst sınıra clamp edilmesini, singleton/asimetrik aralıkları, aralığı aşan adımı, i32 origin sınırlarını ve lower/upper ile step hatalarını kapsayan toplam **24 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1638/1638** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2140/2140** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `18248ddafaffd46fa827971d82704febaf2aa40a94cc0b653078329db1f4cacf`. Birleşik referans kimliği: `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f`.

Adapter hashleri: debug `4a9c082a05009b8ac6c37d12ee8814942a447b8e2e3a445f3ed95942b6a8356e`, release `750a8467b38e832bff19cfcaa43a5a4f94653270c98fd4a4d3f46248f0665f5d`.

Yerel raporlar: `compare-ep8fiu6` ve `compare-z9iwcisf` (math debug/release), `compare-a21c2t4m` ve `compare-uwg2_ewv` (birleşik debug/release), `verify-8mq2_8pm` (math) ve `verify-yscj0_fq` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.outFromOrigin` overload'larının sabit girdili, sonlanan stream sonuçlarına ve mevcut corpus regresyonuna aittir. Büyük overflow aralıkları, sonlanmayan akış olasılıkları ve bu helper'ı kullanan istemci davranışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
