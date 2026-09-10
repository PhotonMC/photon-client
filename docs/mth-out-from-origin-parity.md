# P01.a — Mth outFromOrigin doğrulaması

`src/math.rs`, Java `Mth.outFromOrigin` üç ve dört argümanlı overload'larının sınır doğrulamasını, origin clamp'ini ve lazy stream tüketim sırasını i32 sarmasıyla taşır. `examples/behavior_adapter.rs` aynı sonucu JSON i32 listesi veya Java hata metni olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Default ve explicit step overload'larını, origin'in alt/üst sınıra clamp edilmesini, singleton/asimetrik aralıkları, aralığı aşan adımı, i32 origin sınırlarını ve lower/upper ile step hatalarını kapsayan toplam **24 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1843/1843** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `dfeb55b1b61cbc7649df7ae8af88469cbf7480a194892aacbcf8e5e5b93806bd`. Birleşik referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Adapter hashleri: debug `c3f51b7830e3c0554138ff428a979a8f54ffaf85f7e2082bc6962bdcf01ff1d9`, release `fa350ee97911ed79b9b6db86a8af51e8b7dc87401e8312957b94741266b4b51b`.

Yerel raporlar: `compare-5_avciij` ve `compare-e6e6agr4` (math debug/release), `compare-gvo3eind` ve `compare-wmabbl5t` (birleşik debug/release), `verify-zbtpetmh` (math) ve `verify-vzenwlxd` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.outFromOrigin` overload'larının sabit girdili, sonlanan stream sonuçlarına ve mevcut corpus regresyonuna aittir. Büyük overflow aralıkları, sonlanmayan akış olasılıkları ve bu helper'ı kullanan istemci davranışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
