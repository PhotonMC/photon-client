# P01.a — Mth wobble(double) doğrulaması

`src/math.rs` içindeki `mth_wobble_f64`, Java `Mth.wobble(double)` yordamının floor-derived seed, tek legacy LCG `nextDouble()` çekişi ve kaynak sıralı f64 gürültü eklemesini taşır. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryosuna bağlar; beklenen değerleri üretmez.

- Math ailesinin 1999 senaryoluk Java referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1999/1999** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2501/2501** senaryoda hem debug hem release adaptörüyle eşleşti.
- Wobble için 20 sentetik koordinat; signed zero, floor sınırı, subnormal, int daraltma sınırı, Infinity ve NaN payload'ı içerir.

Math referans kimliği: `d0cf0bb203c8ea48ad8144fae6a58022846d1a5cfb95b398018620bae7b0bc3d`. Birleşik referans kimliği: `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040`.

Adapter hashleri: debug `5d019ee46fe3c7981d2881f496667dfc50b632b55a72f1c124178a69528f7705`, release `6081f381a8d8391d22750c81b039167dc2cb4dabf8274f6410be0463eb1881a8`.

Yerel raporlar: `compare-druvbg3h` ve `compare-k2ed8y70` (math debug/release), `compare-h1kbcaxz` ve `compare-js7_mas4` (birleşik debug/release), `verify-izjnelsu` (math) ve `verify-d2ptfzig` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansında `Mth.wobble(double)` ve bu dilimin 20 girdisiyle sınırlıdır. No-arg thread-local random üretimi ve diğer random source implementasyonları bu dilimin dışındadır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
