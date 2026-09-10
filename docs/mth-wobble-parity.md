# P01.a — Mth wobble(double) doğrulaması

`src/math.rs` içindeki `mth_wobble_f64`, Java `Mth.wobble(double)` yordamının floor-derived seed, tek legacy LCG `nextDouble()` çekişi ve kaynak sıralı f64 gürültü eklemesini taşır. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryosuna bağlar; beklenen değerleri üretmez.

- Math ailesinin 1783 senaryoluk Java referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1783/1783** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2285/2285** senaryoda hem debug hem release adaptörüyle eşleşti.
- Wobble için 20 sentetik koordinat; signed zero, floor sınırı, subnormal, int daraltma sınırı, Infinity ve NaN payload'ı içerir.

Math referans kimliği: `c9c0c64d34c86e78a480fb2f63bd3bd5633a857332ad5997d97039c193be5338`. Birleşik referans kimliği: `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90`.

Adapter hashleri: debug `843cc0c04c00803acc1e8bf774f290df5e273c7777789a8df5b971cb6a7f1ec0`, release `200c25d47d60d5c096012435e86b4b9f28f06965fa1ac6a453b948b2e6e33f4e`.

Yerel raporlar: `compare-1g27oorn` ve `compare-bkcvw7oi` (math debug/release), `compare-qmmqnzwk` ve `compare-ogma6668` (birleşik debug/release), `verify-_e108v7f` (math) ve `verify-1hv399w4` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansında `Mth.wobble(double)` ve bu dilimin 20 girdisiyle sınırlıdır. No-arg thread-local random üretimi ve diğer random source implementasyonları bu dilimin dışındadır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
