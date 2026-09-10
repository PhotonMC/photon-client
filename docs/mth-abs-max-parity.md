# P01.a — Mth absMax doğrulaması

`src/math.rs`, Java `Mth.absMax(float, float)` ve `absMax(double, double)` yollarını her girdide Java `Math.abs`, ardından Java `Math.max` sırasıyla uygular. Mutlak değer için işaret biti temizlenir; NaN dalları Rust genel `max` davranışına bırakılmaz. `examples/behavior_adapter.rs` sonuçları raw f32/f64 bit string'i olarak taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 14, toplam **28 özgün senaryo** hazırlandı.
- Pozitif/negatif girdiler, eşit mutlak değerler, sıfır ve signed zero, minimum subnormal, maksimum sonlu değer, Infinity ve pozitif/negatif NaN girdileri karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1593/1593** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2095/2095** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `ec7f2431c2fc0a873c374068f226b4309821be58380c7e5eadccdc15fe690723`. Birleşik referans kimliği: `17d40db8607a5df22fc08a42ccd0065b6ffe818cef98507474d8663297c0ac04`.

Adapter hashleri: debug `10f1cebf91a6374fb899fea510944d01aebbdc72d1dd134cb8cf090110794e99`, release `72a957c0b141e6a151fce06d38b47a42ba85f1aa0fa123a3001a10dc58c77b79`.

Yerel raporlar: `compare-gqgspijr` ve `compare-z8zpkwnx` (math debug/release), `compare-94x3ynna` ve `compare-2cii6gte` (birleşik debug/release), `verify-7bmsjnn9` (math) ve `verify-lu6dvbz4` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `absMax` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. Integer `absMax`, `Mth.abs(int)`, `equal`, positiveModulo, clamp/lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
