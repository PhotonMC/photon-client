# P01.a — Mth tamsayı temel yardımcıları doğrulaması

`src/math.rs`, Java `Mth` mutlak değer, tamsayı clamp, katlık ve int açı sarmalama yardımcılarını Java'nın sınır ve signed kalan kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `abs`, `clamp`, `isMultipleOf` ve `wrapDegrees(int)` için 25 özgün senaryo hazırlandı.
- `Integer.MIN_VALUE`, `Long.MIN_VALUE`, ters clamp sınırları, negatif katlık ve `[-180, 180)` açı uçları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1221/1221** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1723/1723** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `df8b0fd452c7f12175fe9ced3c534c1f64bdb7df75cc28529becd0eb8ab97b3c`. Birleşik referans kimliği: `86380f621c6f188dddfa1e68a94057a8ada6d638b9f5c04d3a2c9682b1ca75cf`.

Adapter hashleri: debug `22427f83f6b3a0169daf440b83d79ed848c59505a9c69301cc728150450ce6a1`, release `773f4440cec6e289ab00a08177c316caf2110bfd08b3a13cd3d426a0ed8da05a`.

Yerel raporlar: `compare-6dvmyv_h` ve `compare-ak0n9_u3` (math debug/release), `compare-_8esk0jy` ve `compare-g98k9sn0` (birleşik debug/release), `verify-6_r80rf4` (math) ve `verify-72fv37y0` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.abs(int)`, `clamp(int,int,int)`, `clamp(long,long,long)`, `isMultipleOf(int,int)` ve `wrapDegrees(int)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, açı ve tamsayı overload'larının geri kalanı ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
