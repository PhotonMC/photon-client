# P01.a — Mth yuvarlama yardımcıları doğrulaması

`src/math.rs`, Java `Mth.ceil(float)`, `ceil(double)`, `lfloor(double)` ve `ceilLong(double)` yollarını Java'nın ceil/floor yönü ve floating-point-to-int/long daraltma kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Dört `Mth` rounding overload'u için 36 özgün senaryo hazırlandı.
- Kesirli pozitif/negatif değerler, signed zero, int sınırları, long sınırları, NaN ve Infinity karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2100/2100** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2602/2602** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `f84b8d0a8b920d9e5455aefeebd2a9d768b7c9af5ace5dd1c90ff816a9e20abb`. Birleşik referans kimliği: `ec46e2f5bb40c4fdc045039ffcf03e500f593e7a5659dcd76ba9519601df03f2`.

Adapter hashleri: debug `08c3d7a5f592502ade6a94189a9046d939bb3530591885c02a0d6e734d3a67f5`, release `278394f6e917e92bfa810fe86c950acff09c5f0e74a5807ab45be9fe4ac423158`.

Yerel raporlar: `compare-f8rnhap6` ve `compare-9ykm10o3` (math debug/release), `compare-qo27kk4p` ve `compare-066pmptb` (birleşik debug/release), `verify-_rkkkp6r` (math) ve `verify-icag4r7w` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.ceil(float)`, `ceil(double)`, `lfloor(double)` ve `ceilLong(double)` sonuçlarıyla sınırlıdır. `Mth.floor` dışında kalan diğer sayısal yardımcılar ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
