# P01.c — ChunkPos blok koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` blok koordinatı yardımcılarını aynı i32 kaydırma ve toplama kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik x/z/offset girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos blok koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Minimum, orta, maksimum ve offset'li koordinatlar x/z sırasını korudu; negatif chunk koordinatları, shift taşması ve offset toplama taşması Java i32 sarma davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1007/1007** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1509/1509** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1509 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `7b8b9ea0f5717fc78d7092f1d3563134bfdae91e1c70cf411bb3269f1ad58d4d`. Birleşik referans kimliği: `f71b676b2ae17505734ec3d87f208055714fa5dd6b9cc9417d155abf92e2ac0a`.

Yerel raporlar: `compare-upfzoh9c` (math debug), `compare-czpd1pgo` (math release), `compare-_3h0xf61` (birleşik debug), `compare-a2h2wcm2` (birleşik release) ve `verify-19j4rwux` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'un blok x/z alt-orta-üst uçlarını ve offset'li blok koordinatlarını kapsar. Region koordinatları, `BlockPos` üretimi, contains/distance/range yardımcıları, stream/JSON codec'leri ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
