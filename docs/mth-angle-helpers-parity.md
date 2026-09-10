# P01.a — Mth açı yardımcıları doğrulaması

`src/math.rs`, Java `Mth` açı paketleme, açma ve long açı sarmalama yardımcılarını f32 daraltma, byte daraltma ve signed kalan kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `packDegrees`, `unpackDegrees` ve `wrapDegrees(long)` için 21 özgün senaryo hazırlandı.
- F32 açı sınırları, NaN/Infinity, signed byte uçları ve `Long.MIN_VALUE`/`Long.MAX_VALUE` kalanları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2124/2124** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2626/2626** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.packDegrees(float)`, `unpackDegrees(byte)` ve `wrapDegrees(long)` sonuçlarıyla sınırlıdır. Diğer `wrapDegrees` overload'ları, derece farkı/rotation yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
