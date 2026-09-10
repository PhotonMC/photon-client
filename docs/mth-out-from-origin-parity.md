# P01.a — Mth outFromOrigin doğrulaması

`src/math.rs`, Java `Mth.outFromOrigin` üç ve dört argümanlı overload'larının sınır doğrulamasını, origin clamp'ini ve lazy stream tüketim sırasını i32 sarmasıyla taşır. `examples/behavior_adapter.rs` aynı sonucu JSON i32 listesi veya Java hata metni olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Default ve explicit step overload'larını, origin'in alt/üst sınıra clamp edilmesini, singleton/asimetrik aralıkları, aralığı aşan adımı, i32 origin sınırlarını ve lower/upper ile step hatalarını kapsayan toplam **24 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1506/1506** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2008/2008** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `521c517003287d78e7dee051e04062b28c1fdcf7965213cb691cb7e1e0a3f9df`. Birleşik referans kimliği: `8a534e59c23e5da8152cd10fc0f559f6e05cac44a30f5600b40c9a6b5a45ce57`.

Adapter hashleri: debug `52e3bc734f101621ceeedaaab373e873cb8ff23b03f5e311af53dca952ea5d65`, release `cc56dec0269a5529f0c2179449bd12ee4a89acc1dbe7aba6c4c6422d930460ad`.

Yerel raporlar: `compare-wn40y3iw` ve `compare-9phkvs2k` (math debug/release), `compare-a415_00_` ve `compare-djb85w4w` (birleşik debug/release), `verify-ih3gzbil` (math) ve `verify-nqxvpkjm` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.outFromOrigin` overload'larının sabit girdili, sonlanan stream sonuçlarına ve mevcut corpus regresyonuna aittir. Büyük overflow aralıkları, sonlanmayan akış olasılıkları ve bu helper'ı kullanan istemci davranışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
