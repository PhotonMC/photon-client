# Mth fastInvSqrt davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `fastInvSqrt(double)` yordamını kapsar. Yordam f64 girdinin raw bitlerini signed long görünümüne alır, sabit bit tahmini üretir ve tek Newton düzeltmesini Java kaynak sırasıyla uygular.

`Double.doubleToRawLongBits` NaN payload'ını korur; `i >> 1` signed long aritmetik sağa kaydırmadır ve sabit çıkarma long genişliğinde değerlendirilir. Sonraki çarpma/çıkarma işlemleri f64'tür. Çıktı raw f64 bit string olarak taşınır.

Java oracle `Mth.fastInvSqrt` yordamını doğrudan çağırır; beklenen bitler Rust uygulamasından türetilmez. Senaryolar sıfır, signed zero, pozitif/negatif normal ve subnormal değerler, sınırlar, sonsuzluk ve NaN payload'larını kapsar.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:449-455`.

Kapsam dışı kalanlar: `atan2`, JOML `invSqrt`, `fastInvCubeRoot` ve gerçek ray/vektör kullanım sıraları.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs` içindeki `mth_fast_inv_sqrt_f64`, Java `Mth.fastInvSqrt(double)` yolunu JSONL adapter'a bağlar. Raw bit tahmini, signed long kaydırma/çıkarma ve Newton düzeltmesi Java oracle'ın raw f64 `bits` sonucu ile karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Sıfır, signed zero, 1.0 bit komşuları, normal/subnormal pozitif ve negatif değerler, f64 sınırları, sonsuzluk ve farklı NaN payload'ları için 24 özgün senaryo hazırlandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2299 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2299/2299 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2801/2801 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb82`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.fastInvSqrt(double)` yordamı ve bu dilimin 24 raw f64 girdisiyle sınırlıdır. `atan2`, JOML `invSqrt`, `fastInvCubeRoot` ve gerçek ray/vektör kullanım sıraları ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
