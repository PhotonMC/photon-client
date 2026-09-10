# P01.a — Mth açı yardımcıları doğrulaması

`src/math.rs`, Java `Mth` açı paketleme, açma ve long açı sarmalama yardımcılarını f32 daraltma, byte daraltma ve signed kalan kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `packDegrees`, `unpackDegrees` ve `wrapDegrees(long)` için 21 özgün senaryo hazırlandı.
- F32 açı sınırları, NaN/Infinity, signed byte uçları ve `Long.MIN_VALUE`/`Long.MAX_VALUE` kalanları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1843/1843** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `dfeb55b1b61cbc7649df7ae8af88469cbf7480a194892aacbcf8e5e5b93806bd`. Birleşik referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Adapter hashleri: debug `c3f51b7830e3c0554138ff428a979a8f54ffaf85f7e2082bc6962bdcf01ff1d9`, release `fa350ee97911ed79b9b6db86a8af51e8b7dc87401e8312957b94741266b4b51b`.

Yerel raporlar: `compare-5_avciij` ve `compare-e6e6agr4` (math debug/release), `compare-gvo3eind` ve `compare-wmabbl5t` (birleşik debug/release), `verify-zbtpetmh` (math) ve `verify-vzenwlxd` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.packDegrees(float)`, `unpackDegrees(byte)` ve `wrapDegrees(long)` sonuçlarıyla sınırlıdır. Diğer `wrapDegrees` overload'ları, derece farkı/rotation yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
