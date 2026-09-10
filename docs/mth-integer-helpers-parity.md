# P01.a — Mth tamsayı yardımcıları doğrulaması

`src/math.rs`, Java `Mth` tamsayı bölme, pozitif modulo ve yuvarlama yardımcılarını Java'nın işaret ve taşma kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `floorDiv`, `positiveModulo`, `positiveCeilDiv` ve `roundToward` için 15 özgün senaryo hazırlandı.
- Negatif kalan ve bölen işaretleri, `Integer.MIN_VALUE`/`Long.MIN_VALUE` unary negation ve bölüm taşmaları ile sonuç çarpması taşmaları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1335/1335** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1837/1837** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `8956f11f5ab0ba4c81a6aa2c11af8623a498bbb78a550a1b013695afc5029da2`. Birleşik referans kimliği: `f90c38c2c4e4eb83aec92229706d7f6b47bc90a128aa4908ed60897e8279a0d9`.

Adapter hashleri: debug `b7e457856fcdb6393a75a5130687fa1dc8f26fa2c8189563640e095b017a42b3`, release `76b25e8aa9ce62c15bba7e4c5df43331ab31d1618f93e136b2e945ff9a0a9531`.

Yerel raporlar: `compare-0dy1p_jx` ve `compare-fdzx3gij` (math debug/release), `compare-gridp_gxu` ve `compare-u8zwfn4r` (birleşik debug/release), `verify-cwah2qjd` (math) ve `verify-yhwxvq8l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.floorDiv(int,int)`, `positiveModulo(int,int)`, `positiveCeilDiv(int,int)`, `positiveCeilDiv(long,long)`, `roundToward(int,int)` ve `roundToward(long,long)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, diğer Mth tamsayı overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
