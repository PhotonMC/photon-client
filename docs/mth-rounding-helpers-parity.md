# P01.a — Mth yuvarlama yardımcıları doğrulaması

`src/math.rs`, Java `Mth.ceil(float)`, `ceil(double)`, `lfloor(double)` ve `ceilLong(double)` yollarını Java'nın ceil/floor yönü ve floating-point-to-int/long daraltma kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Dört `Mth` rounding overload'u için 36 özgün senaryo hazırlandı.
- Kesirli pozitif/negatif değerler, signed zero, int sınırları, long sınırları, NaN ve Infinity karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1335/1335** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1837/1837** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `8956f11f5ab0ba4c81a6aa2c11af8623a498bbb78a550a1b013695afc5029da2`. Birleşik referans kimliği: `f90c38c2c4e4eb83aec92229706d7f6b47bc90a128aa4908ed60897e8279a0d9`.

Adapter hashleri: debug `b7e457856fcdb6393a75a5130687fa1dc8f26fa2c8189563640e095b017a42b3`, release `76b25e8aa9ce62c15bba7e4c5df43331ab31d1618f93e136b2e945ff9a0a9531`.

Yerel raporlar: `compare-0dy1p_jx` ve `compare-fdzx3gij` (math debug/release), `compare-gridp_gxu` ve `compare-u8zwfn4r` (birleşik debug/release), `verify-cwah2qjd` (math) ve `verify-yhwxvq8l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.ceil(float)`, `ceil(double)`, `lfloor(double)` ve `ceilLong(double)` sonuçlarıyla sınırlıdır. `Mth.floor` dışında kalan diğer sayısal yardımcılar ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
