# Mth rayIntersectsAABB davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `rayIntersectsAABB(Vec3, Vec3, AABB)` yordamını kapsar. Yordam ray başlangıcı, yönü ve AABB uçlarından merkez/yarı kapsam değerlerini çıkarır; önce üç eksen dışlama koşulunu, sonra üç çapraz çarpım sınırını Java kaynak sırasıyla değerlendirir.

AABB Java kurucusuyla `Math.min`/`Math.max` üzerinden normalize edilir. Tüm ara değerler f64'tür. Dışlama ve son kabul karşılaştırmalarındaki `>`/`>=`/`<` ayrımı, sıfır yön bileşenleri, tangent sınırlar, signed zero ve özel f64 değerlerle birlikte korunur.

Behavior runner ray başlangıcı ve yönünü `x`/`y`/`z` raw f64 bit alanlarıyla, kutuyu altı raw f64 endpoint alanıyla taşır; sonuç JSON boolean `value` alanıdır. Java oracle `Mth.rayIntersectsAABB` yordamını doğrudan çağırır; beklenen sonuç Rust uygulamasından türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:352-389`.

Kapsam dışı kalanlar: AABB segment `clip`, VoxelShape kesişimleri, `atan2`, ters karekök yardımcıları ve ray yönünün normalize edilmesi.
