# P01.a — Mth square(int) doğrulaması

`src/math.rs`, Java `Mth.square(int)` yordamının tek i32 çarpımını Java'nın 32 bit sarmasıyla taşır. `examples/behavior_adapter.rs` sonucu signed decimal i32 string olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Sıfır, birim ve negatif girdiler, 46340/46341 overflow sınırı, iki'nin kuvvetiyle tam sarmalar ve i32 minimum/maksimum sınırlarını içeren **15 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1703/1703** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2205/2205** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `b93cda4c5257c03011378a24fe916e8b8cd7e11d2fe75e1a1de108e4b4ebd630`. Birleşik referans kimliği: `413b981c2a619373545c6c4c74d3e7fec6b1a2e1dc34d5a49ee2f4eed5231162`.

Adapter hashleri: debug `c1464c3995fc085081d8d7d5957a9eca938185a498315c143c30d63e829ffe90`, release `a68437319cbfe090fc182a3c8ec431a6c95605056e456fc47471c96ea253ffda`.

Yerel raporlar: `compare-a8qhmnrl` ve `compare-usm27y8z` (math debug/release), `compare-8vwtvwwl` ve `compare-uey51w68` (birleşik debug/release), `verify-d3ywhdwp` (math) ve `verify-l3irw60r` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.square(int)` yordamının sabit signed i32 girdilerdeki sonucunu kapsar. Diğer `square` overload'ları, `cube`, `clampedMap`, `map`, diğer Mth yardımcıları ve bu yordamı kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
