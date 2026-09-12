# Mth rayIntersectsAABB davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `rayIntersectsAABB(Vec3, Vec3, AABB)` yordamını kapsar. Yordam ray başlangıcı, yönü ve AABB uçlarından merkez/yarı kapsam değerlerini çıkarır; önce üç eksen dışlama koşulunu, sonra üç çapraz çarpım sınırını Java kaynak sırasıyla değerlendirir.

AABB Java kurucusuyla `Math.min`/`Math.max` üzerinden normalize edilir. Tüm ara değerler f64'tür. Dışlama ve son kabul karşılaştırmalarındaki `>`/`>=`/`<` ayrımı, sıfır yön bileşenleri, tangent sınırlar, signed zero ve özel f64 değerlerle birlikte korunur.

Behavior runner ray başlangıcı ve yönünü `x`/`y`/`z` raw f64 bit alanlarıyla, kutuyu altı raw f64 endpoint alanıyla taşır; sonuç JSON boolean `value` alanıdır. Java oracle `Mth.rayIntersectsAABB` yordamını doğrudan çağırır; beklenen sonuç Rust uygulamasından türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:352-389`.

Kapsam dışı kalanlar: AABB segment `clip`, VoxelShape kesişimleri, `atan2`, ters karekök yardımcıları ve ray yönünün normalize edilmesi.


## Client-parity doğrulaması

`src/math.rs` içindeki `mth_ray_intersects_aabb`, Java `Mth.rayIntersectsAABB(Vec3, Vec3, AABB)` yolunu JSONL adapter'a bağlar. Üç eksen dışlama ve üç Andrew Woo çapraz çarpım sınırı Java oracle'ın boolean `value` sonucu ile karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- İçeride, dışarıda, eksen yönlü, diagonal, tangent, sınır, paralel, sıfır/signed-zero, ters/degenerate kutu ve özel f64 girdileri için 29 özgün senaryo hazırlandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2299 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2299/2299 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2801/2801 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb82`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.rayIntersectsAABB` yordamı ve bu dilimin 29 sentetik ray/kutu girdisiyle sınırlıdır. AABB segment `clip`, VoxelShape kesişimleri, `atan2`, ters karekök yardımcıları ve ray yönünün normalize edilmesi ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
