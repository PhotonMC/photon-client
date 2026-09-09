# P01.a — Mth açı yardımcıları doğrulaması

`src/math.rs`, Java `Mth` açı paketleme, açma ve long açı sarmalama yardımcılarını f32 daraltma, byte daraltma ve signed kalan kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `packDegrees`, `unpackDegrees` ve `wrapDegrees(long)` için 21 özgün senaryo hazırlandı.
- F32 açı sınırları, NaN/Infinity, signed byte uçları ve `Long.MIN_VALUE`/`Long.MAX_VALUE` kalanları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1249/1249** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1751/1751** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `29be5693737a944f40d9534d4a126556f5aeaac24dd9d2f5fccd0c6b1db3a3aa`. Birleşik referans kimliği: `a92e7ef25d7ffcf101102bab4b05061d96a3b0511495a77464f282a0775abfd0`.

Adapter hashleri: debug `9437f241df809f7c41acbf27ebf40a730cba810baaebd620ac0006230687c3d9`, release `2749c23cb3aa8019363af01df229657d8d954faa4a8063bc6cfcc0ce5f11820d`.

Yerel raporlar: `compare-dj1scyn3` ve `compare-1g72i2us` (math debug/release), `compare-f0bjlm2c` ve `compare-eaqfihsj` (birleşik debug/release), `verify-_qy4rx_r` (math) ve `verify-p2v9lu2o` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.packDegrees(float)`, `unpackDegrees(byte)` ve `wrapDegrees(long)` sonuçlarıyla sınırlıdır. Diğer `wrapDegrees` overload'ları, derece farkı/rotation yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
