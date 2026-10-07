# Mth smoothstep ve smoothstepDerivative davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `smoothstep(double)` ve `smoothstepDerivative(double)` yordamlarını kapsar.

`smoothstep`, `x * x * x * (x * (x * 6.0 - 15.0) + 10.0)` işlem sırasını korur. `smoothstepDerivative`, `30.0 * x * x * (x - 1.0) * (x - 1.0)` ifadesini aynı sırayla yürütür. Girdi aralığı clamp edilmez; taşma, signed zero ve özel kayan nokta sonuçları Java f64 aritmetiğinden gelir.

Behavior runner `x` alanını on altı haneli lowercase raw f64 bit string'i olarak taşır. Oracle, Java `Mth.smoothstep` veya `Mth.smoothstepDerivative` çağrısını doğrudan yürütür ve sonucu `result.bits` altında raw f64 bit string olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:592-598`.

Kapsam dışı kalanlar: `catmullrom`, `sign`, diğer `Mth` interpolasyon yardımcıları ve bu yordamları kullanan istemci akışları.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Mth.smoothstep(double)` ve `Mth.smoothstepDerivative(double)` ifadelerini kaynak işlem sırasıyla taşır. `examples/behavior_adapter.rs` sonuçları raw f64 bit string'leri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her yordam için sıfır/bir, aralık içi ve dışı değerler, signed zero, en küçük subnormal, en büyük sonlu değer, Infinity, NaN ve 1.0 komşularını içeren toplam **28 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt iki `Mth` yordamının sabit raw f64 girdilerde doğrudan aritmetik sonucunu kapsar. `catmullrom`, `sign`, diğer interpolasyon yardımcıları ve bu yordamları kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
