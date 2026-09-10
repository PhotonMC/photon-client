# P01.a — Mth fastInvSqrt doğrulaması

`src/math.rs` içindeki `mth_fast_inv_sqrt_f64`, Java `Mth.fastInvSqrt(double)` yolunu JSONL adapter'a bağlar. Raw bit tahmini, signed long kaydırma/çıkarma ve Newton düzeltmesi Java oracle'ın raw f64 `bits` sonucu ile karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Sıfır, signed zero, 1.0 bit komşuları, normal/subnormal pozitif ve negatif değerler, f64 sınırları, sonsuzluk ve farklı NaN payload'ları için 24 özgün senaryo hazırlandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2100 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2100/2100 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2602/2602 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `f84b8d0a8b920d9e5455aefeebd2a9d768b7c9af5ace5dd1c90ff816a9e20abb`. Birleşik referans kimliği: `ec46e2f5bb40c4fdc045039ffcf03e500f593e7a5659dcd76ba9519601df03f2`.

Adapter hashleri: debug `08c3d7a5f592502ade6a94189a9046d939bb3530591885c02a0d6e734d3a67f5`, release `278394f6e917e92bfa810fe86c950acff09c5f0e74a5807ab45be9fe4ac42315`.

Yerel raporlar: `compare-f8rnhap6` ve `compare-9ykm10o3` (math debug/release), `compare-qo27kk4p` ve `compare-066pmptb` (birleşik debug/release), `verify-_rkkkp6r` (math) ve `verify-icag4r7w` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.fastInvSqrt(double)` yordamı ve bu dilimin 24 raw f64 girdisiyle sınırlıdır. `atan2`, JOML `invSqrt`, `fastInvCubeRoot` ve gerçek ray/vektör kullanım sıraları ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
