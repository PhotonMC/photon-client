# P01.a — Mth absMax doğrulaması

`src/math.rs`, Java `Mth.absMax(float, float)` ve `absMax(double, double)` yollarını her girdide Java `Math.abs`, ardından Java `Math.max` sırasıyla uygular. Mutlak değer için işaret biti temizlenir; NaN dalları Rust genel `max` davranışına bırakılmaz. `examples/behavior_adapter.rs` sonuçları raw f32/f64 bit string'i olarak taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 14, toplam **28 özgün senaryo** hazırlandı.
- Pozitif/negatif girdiler, eşit mutlak değerler, sıfır ve signed zero, minimum subnormal, maksimum sonlu değer, Infinity ve pozitif/negatif NaN girdileri karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1506/1506** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2008/2008** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `521c517003287d78e7dee051e04062b28c1fdcf7965213cb691cb7e1e0a3f9df`. Birleşik referans kimliği: `8a534e59c23e5da8152cd10fc0f559f6e05cac44a30f5600b40c9a6b5a45ce57`.

Adapter hashleri: debug `52e3bc734f101621ceeedaaab373e873cb8ff23b03f5e311af53dca952ea5d65`, release `cc56dec0269a5529f0c2179449bd12ee4a89acc1dbe7aba6c4c6422d930460ad`.

Yerel raporlar: `compare-wn40y3iw` ve `compare-9phkvs2k` (math debug/release), `compare-a415_00_` ve `compare-djb85w4w` (birleşik debug/release), `verify-ih3gzbil` (math) ve `verify-nqxvpkjm` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `absMax` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. Integer `absMax`, `Mth.abs(int)`, `equal`, positiveModulo, clamp/lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
