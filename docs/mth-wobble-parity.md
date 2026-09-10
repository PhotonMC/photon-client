# P01.a — Mth wobble(double) doğrulaması

`src/math.rs` içindeki `mth_wobble_f64`, Java `Mth.wobble(double)` yordamının floor-derived seed, tek legacy LCG `nextDouble()` çekişi ve kaynak sıralı f64 gürültü eklemesini taşır. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryosuna bağlar; beklenen değerleri üretmez.

- Math ailesinin 1963 senaryoluk Java referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1963/1963** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2465/2465** senaryoda hem debug hem release adaptörüyle eşleşti.
- Wobble için 20 sentetik koordinat; signed zero, floor sınırı, subnormal, int daraltma sınırı, Infinity ve NaN payload'ı içerir.

Math referans kimliği: `461f1ba035270ce58c69f06fec7167bb0517cd22aed6c78c25e0b076684b28fa`. Birleşik referans kimliği: `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573`.

Adapter hashleri: debug `30478caf53747391ac6513072beabe17af9a76637c23cc360c7a826700433a54`, release `0b3f2c6bf91361cff411ced1b36b1433ec25e47409a6b2623e9c0624e348115d`.

Yerel raporlar: `compare-hiau72j0` ve `compare-bttpme0o` (math debug/release), `compare-k17hah5m` ve `compare-hiiz6u35` (birleşik debug/release), `verify-cnbvzo8w` (math) ve `verify-82qi_x56` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansında `Mth.wobble(double)` ve bu dilimin 20 girdisiyle sınırlıdır. No-arg thread-local random üretimi ve diğer random source implementasyonları bu dilimin dışındadır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
