# P01.a — Mth wobble(double) doğrulaması

`src/math.rs` içindeki `mth_wobble_f64`, Java `Mth.wobble(double)` yordamının floor-derived seed, tek legacy LCG `nextDouble()` çekişi ve kaynak sıralı f64 gürültü eklemesini taşır. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryosuna bağlar; beklenen değerleri üretmez.

- Math ailesinin 1987 senaryoluk Java referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1987/1987** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2489/2489** senaryoda hem debug hem release adaptörüyle eşleşti.
- Wobble için 20 sentetik koordinat; signed zero, floor sınırı, subnormal, int daraltma sınırı, Infinity ve NaN payload'ı içerir.

Math referans kimliği: `665bdda6827e71f559f4e9a959ed126f3d3e055594f3b677359c51d0d3a74e16`. Birleşik referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Adapter hashleri: debug `bf472fbe984c9033c542831c832c85f3ea36283e022040542677049db4615a06`, release `04de447e6d530b9e66134316577c0e326f349761b64ee41c18f4e76fd6d3b226`.

Yerel raporlar: `compare-zt_h_egg` ve `compare-7afcvx21` (math debug/release), `compare-251aed0_` ve `compare-jqj2kta_` (birleşik debug/release), `verify-5v_ilol4` (math) ve `verify-csdh4b83` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansında `Mth.wobble(double)` ve bu dilimin 20 girdisiyle sınırlıdır. No-arg thread-local random üretimi ve diğer random source implementasyonları bu dilimin dışındadır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
