# P01.a — Mth tamsayı temel yardımcıları doğrulaması

`src/math.rs`, Java `Mth` mutlak değer, tamsayı clamp, katlık ve int açı sarmalama yardımcılarını Java'nın sınır ve signed kalan kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `abs`, `clamp`, `isMultipleOf` ve `wrapDegrees(int)` için 25 özgün senaryo hazırlandı.
- `Integer.MIN_VALUE`, `Long.MIN_VALUE`, ters clamp sınırları, negatif katlık ve `[-180, 180)` açı uçları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1593/1593** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2095/2095** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `ec7f2431c2fc0a873c374068f226b4309821be58380c7e5eadccdc15fe690723`. Birleşik referans kimliği: `17d40db8607a5df22fc08a42ccd0065b6ffe818cef98507474d8663297c0ac04`.

Adapter hashleri: debug `10f1cebf91a6374fb899fea510944d01aebbdc72d1dd134cb8cf090110794e99`, release `72a957c0b141e6a151fce06d38b47a42ba85f1aa0fa123a3001a10dc58c77b79`.

Yerel raporlar: `compare-gqgspijr` ve `compare-z8zpkwnx` (math debug/release), `compare-94x3ynna` ve `compare-2cii6gte` (birleşik debug/release), `verify-7bmsjnn9` (math) ve `verify-lu6dvbz4` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.abs(int)`, `clamp(int,int,int)`, `clamp(long,long,long)`, `isMultipleOf(int,int)` ve `wrapDegrees(int)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, açı ve tamsayı overload'larının geri kalanı ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
