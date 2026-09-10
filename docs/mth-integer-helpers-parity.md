# P01.a — Mth tamsayı yardımcıları doğrulaması

`src/math.rs`, Java `Mth` tamsayı bölme, pozitif modulo ve yuvarlama yardımcılarını Java'nın işaret ve taşma kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `floorDiv`, `positiveModulo`, `positiveCeilDiv` ve `roundToward` için 15 özgün senaryo hazırlandı.
- Negatif kalan ve bölen işaretleri, `Integer.MIN_VALUE`/`Long.MIN_VALUE` unary negation ve bölüm taşmaları ile sonuç çarpması taşmaları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1683/1683** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2185/2185** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `d91d4fb976af018c6ced8eb9d475396c09c64f7ef8429742661a9ba2d8c34267`. Birleşik referans kimliği: `595049f32d6822eff3379ec8f2d5211ef192043bf7f6a11db9b9d7c609fe01a1`.

Adapter hashleri: debug `a8905dba481a438cfebc22b452d1af22f2d3f1e7de4d2d3a376e85d6141eff61`, release `b6b7d1e7f5cd800ac6971e91b29dee4b36db61aef4e537adc1de1d2f5bff5f93`.

Yerel raporlar: `compare-arjeniq4` ve `compare-_0386q8q` (math debug/release), `compare-63qzbune` ve `compare-1pb7x57b` (birleşik debug/release), `verify-wl7nytn8` (math) ve `verify-f_mau559` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.floorDiv(int,int)`, `positiveModulo(int,int)`, `positiveCeilDiv(int,int)`, `positiveCeilDiv(long,long)`, `roundToward(int,int)` ve `roundToward(long,long)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, diğer Mth tamsayı overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
