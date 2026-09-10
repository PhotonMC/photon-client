# P01.a — Mth wobble(double) doğrulaması

`src/math.rs` içindeki `mth_wobble_f64`, Java `Mth.wobble(double)` yordamının floor-derived seed, tek legacy LCG `nextDouble()` çekişi ve kaynak sıralı f64 gürültü eklemesini taşır. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryosuna bağlar; beklenen değerleri üretmez.

- Math ailesinin 2100 senaryoluk Java referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2100/2100** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2602/2602** senaryoda hem debug hem release adaptörüyle eşleşti.
- Wobble için 20 sentetik koordinat; signed zero, floor sınırı, subnormal, int daraltma sınırı, Infinity ve NaN payload'ı içerir.

Math referans kimliği: `f84b8d0a8b920d9e5455aefeebd2a9d768b7c9af5ace5dd1c90ff816a9e20abb`. Birleşik referans kimliği: `ec46e2f5bb40c4fdc045039ffcf03e500f593e7a5659dcd76ba9519601df03f2`.

Adapter hashleri: debug `08c3d7a5f592502ade6a94189a9046d939bb3530591885c02a0d6e734d3a67f5`, release `278394f6e917e92bfa810fe86c950acff09c5f0e74a5807ab45be9fe4ac423158`.

Yerel raporlar: `compare-f8rnhap6` ve `compare-9ykm10o3` (math debug/release), `compare-qo27kk4p` ve `compare-066pmptb` (birleşik debug/release), `verify-_rkkkp6r` (math) ve `verify-icag4r7w` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansında `Mth.wobble(double)` ve bu dilimin 20 girdisiyle sınırlıdır. No-arg thread-local random üretimi ve diğer random source implementasyonları bu dilimin dışındadır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
