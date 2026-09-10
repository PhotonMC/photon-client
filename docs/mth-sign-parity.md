# P01.a — Mth sign doğrulaması

`src/math.rs`, Java `Mth.sign(double)` yordamının sıfır, pozitif ve negatif dallarını; NaN için negatif sonucu koruyarak taşır. `examples/behavior_adapter.rs` sonucu signed decimal i32 string olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Negatif/pozitif sonlu değerler, `-0.0`, `+0.0`, subnormal, en büyük sonlu değer, Infinity ve NaN içeren **14 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **1903/1903** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2405/2405** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `0cc821a10c9af88c8563c153e55d57b691a8c69e33d90a8bd3022a5b3410bfd0`. Birleşik referans kimliği: `2daaa6bb28eca57adfb18563e32f011d764209b98ec6de5934a92a72e80abdfa`.

Adapter hashleri: debug `b2f092b374faa99b0fbb325d175fd2ca57c0c81b8a1b913240e9305da476fe74`, release `c1bd320746e6a2b869befca2f1abe4e529fccd8e94fdd8204a5cd0ca6176269a`.

Yerel raporlar: `compare-xiw125c3` ve `compare-v5h63z7q` (math debug/release), `compare-chdvlz2_` ve `compare-oywxz0ky` (birleşik debug/release), `verify-f023qdmh` (math) ve `verify-da0drjib` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.sign` sonucunu sabit raw f64 girdilerde kapsar. `rotLerp`, `rotLerpRad`, `triangleWave`, diğer açı yardımcıları ve `sign` kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
