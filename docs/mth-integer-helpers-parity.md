# P01.a — Mth tamsayı yardımcıları doğrulaması

`src/math.rs`, Java `Mth` tamsayı bölme, pozitif modulo ve yuvarlama yardımcılarını Java'nın işaret ve taşma kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `floorDiv`, `positiveModulo`, `positiveCeilDiv` ve `roundToward` için 15 özgün senaryo hazırlandı.
- Negatif kalan ve bölen işaretleri, `Integer.MIN_VALUE`/`Long.MIN_VALUE` unary negation ve bölüm taşmaları ile sonuç çarpması taşmaları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2076/2076** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2578/2578** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`. Birleşik referans kimliği: `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8`.

Adapter hashleri: debug `6eb1e772a2e06c20b8d0c09ba0b2a11b5f1df7a671ee8d1513127bc318d143ef`, release `318c5ef22a8d56277aa19f9da9f5becca9f575904eba79756b959bdc8c993fe88`.

Yerel raporlar: `compare-k2v65l83` ve `compare-9uf1gox2` (math debug/release), `compare-ptlm9ptr` ve `compare-86z18x_k` (birleşik debug/release), `verify-9dtf6rea` (math) ve `verify-31k8bjcu` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.floorDiv(int,int)`, `positiveModulo(int,int)`, `positiveCeilDiv(int,int)`, `positiveCeilDiv(long,long)`, `roundToward(int,int)` ve `roundToward(long,long)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, diğer Mth tamsayı overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
