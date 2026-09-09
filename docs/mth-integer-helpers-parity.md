# P01.a — Mth tamsayı yardımcıları doğrulaması

`src/math.rs`, Java `Mth` tamsayı bölme, pozitif modulo ve yuvarlama yardımcılarını Java'nın işaret ve taşma kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `floorDiv`, `positiveModulo`, `positiveCeilDiv` ve `roundToward` için 15 özgün senaryo hazırlandı.
- Negatif kalan ve bölen işaretleri, `Integer.MIN_VALUE`/`Long.MIN_VALUE` unary negation ve bölüm taşmaları ile sonuç çarpması taşmaları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1193/1193** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1695/1695** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `3a6a2f5a3d8ff6b65e565413ca95aeeb802f3ca965620ed874e898bcc52c5d61`. Birleşik referans kimliği: `5f58cd401556c40d3fc3eb7c3e511a8dbc2553c3390034c888e648d9d8a3ad4c`.

Adapter hashleri: debug `632535c3aabe8ce4f0613f6b855c54e499456585aa71beaa93412b83b0c57a54`, release `7c059afa0db707e2c536034aab21d80bc8d27f9b04cbe81728a84261b43691a7`.

Yerel raporlar: `compare-dgq7mp_7` ve `compare-2hmy86uj` (math debug/release), `compare-06i9lguy` ve `compare-r6guli2r` (birleşik debug/release), `verify-9n41v5aa` (math) ve `verify-k_vt31c0` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.floorDiv(int,int)`, `positiveModulo(int,int)`, `positiveCeilDiv(int,int)`, `positiveCeilDiv(long,long)`, `roundToward(int,int)` ve `roundToward(long,long)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, diğer Mth tamsayı overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
