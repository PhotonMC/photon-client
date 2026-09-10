# P01.a — Mth tamsayı temel yardımcıları doğrulaması

`src/math.rs`, Java `Mth` mutlak değer, tamsayı clamp, katlık ve int açı sarmalama yardımcılarını Java'nın sınır ve signed kalan kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `abs`, `clamp`, `isMultipleOf` ve `wrapDegrees(int)` için 25 özgün senaryo hazırlandı.
- `Integer.MIN_VALUE`, `Long.MIN_VALUE`, ters clamp sınırları, negatif katlık ve `[-180, 180)` açı uçları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1963/1963** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2465/2465** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `461f1ba035270ce58c69f06fec7167bb0517cd22aed6c78c25e0b076684b28fa`. Birleşik referans kimliği: `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573`.

Adapter hashleri: debug `30478caf53747391ac6513072beabe17af9a76637c23cc360c7a826700433a54`, release `0b3f2c6bf91361cff411ced1b36b1433ec25e47409a6b2623e9c0624e348115d`.

Yerel raporlar: `compare-hiau72j0` ve `compare-bttpme0o` (math debug/release), `compare-k17hah5m` ve `compare-hiiz6u35` (birleşik debug/release), `verify-cnbvzo8w` (math) ve `verify-82qi_x56` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.abs(int)`, `clamp(int,int,int)`, `clamp(long,long,long)`, `isMultipleOf(int,int)` ve `wrapDegrees(int)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, açı ve tamsayı overload'larının geri kalanı ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
