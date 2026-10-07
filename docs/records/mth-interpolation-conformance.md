# Mth lerp2/lerp3/catmullrom davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `lerp2(double)`, `lerp3(double)` ve `catmullrom(float)` yordamlarını kapsar. Her yordam için 20 sentetik senaryo hazırlanır.

`lerp2`, önce iki x ekseni interpolasyonunu aynı `alpha1` ile hesaplar, ardından bu iki sonucu `alpha2` ile birleştirir. `lerp3`, alt ve üst düzlemlerde `lerp2(alpha1, alpha2, ...)` çağrılarını kaynak sırasıyla yapar ve iki sonucu `alpha3` ile birleştirir. Bu yollar alpha değerlerini clamp etmez; çıkarma, çarpma ve toplama sırası Java scalar `lerp` yordamıyla korunur.

`catmullrom`, dört f32 kontrol noktasını kaynakta yazılan kübik polinomla işler. `alpha * alpha` ve `alpha * alpha * alpha` alt ifadeleri, katsayı çarpımları ve toplamlar Java f32 işlem sırasını korur. Signed zero, subnormal, taşma, Infinity ve NaN sonuçları bu aritmetik yolun parçasıdır.

Behavior runner `lerp2` ve `lerp3` girdilerini `alpha1`, `alpha2`, `alpha3` ve ilgili `x...` alanlarında lowercase raw f64 IEEE-754 bit string'leriyle; `catmullrom` girdilerini `alpha`, `p0`, `p1`, `p2`, `p3` alanlarında raw f32 bit string'leriyle taşır. Java oracle yordamı doğrudan çağırır ve sonucu hassasiyetine göre `result.bits` altında döndürür. Beklenen değerler Rust uygulamasından türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:558-590`.

Kapsam dışı kalanlar: scalar `lerp`, `lerpInt`, `lerpDiscrete`, Vec3 `lerp`, `clampedLerp`, `inverseLerp`, `map`, `wobble` ve bu helper'ları kullanan istemci akışları.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs` içindeki `mth_lerp2_f64`, `mth_lerp3_f64` ve `mth_catmullrom_f32`, Java `Mth` interpolasyon yordamlarının iç içe çağrı ve kaynak sıralı polinom aritmetiğini taşır. `examples/behavior_adapter.rs` bu üretim yordamlarını JSONL senaryolarına bağlar; beklenen değerleri üretmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `lerp2`, `lerp3` ve `catmullrom` için her biri 20 olmak üzere 60 özgün senaryo hazırlandı. Alpha extrapolation'ı, ters ve signed-zero girdiler, subnormal değerler, hassasiyet sırası, taşma, Infinity ve NaN sonuçları kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2299 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2299/2299 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2801/2801 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki üç `Mth` yordamı ve bu dilimin 60 girdisiyle sınırlıdır. Scalar `lerp`, `lerpInt`, `lerpDiscrete`, Vec3 `lerp` ve diğer Mth interpolasyon kullanıcıları ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
