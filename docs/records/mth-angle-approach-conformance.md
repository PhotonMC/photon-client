# Mth açı yaklaşım davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `rotateIfNecessary(float, float, float)`, `approach(float, float, float)` ve `approachDegrees(float, float, float)` yollarını kapsar.

`rotateIfNecessary`, hedef ile taban arasındaki `degreesDifference` sonucunu `[-maxAngleDiff, maxAngleDiff]` sınırlarında Java f32 `clamp` ile sınırlar ve bu değeri hedef açıdan çıkarır. Sınırlar ayrıca normalize edilmez; ters sınırlar ve özel f32 değerleri Java karşılaştırma sırasına bağlıdır.

`approach`, artışın Java `abs(float)` sonucunu kullanır. Geçerli değer hedeften küçükse toplama sonucunu `[current, target]` aralığında, aksi durumda çıkarma sonucunu `[target, current]` aralığında clamp eder. Eşitlik, signed zero, taşma, Infinity ve NaN dalları bu sıralamanın parçasıdır.

`approachDegrees`, önce `degreesDifference(current, target)` ile açısal farkı `[-180, 180)` aralığına taşır, ardından `current + difference` hedefiyle `approach` yolunu çağırır. Ara f32 toplaması ve artışın mutlak değer alınması korunur.

Senaryo girdileri raw f32 bit string'leridir. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonuçları raw f32 bit string'i olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:263-277`.

Kapsam dışı kalanlar: `getInt`, `smallestEncompassingPowerOfTwo`, diğer `wrapDegrees` overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs` içindeki `mth_rotate_if_necessary_f32`, `mth_approach_f32` ve `mth_approach_degrees_f32`, Java `Mth.rotateIfNecessary`, `Mth.approach` ve `Mth.approachDegrees` yollarını JSONL adapter'a bağlar. Derece farkı sarmalama, f32 clamp sınırları, artış yönü, ara toplama, signed zero ve özel değer sonuçları Java oracle'ın raw f32 bitleriyle karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `rotateIfNecessary`, `approach` ve `approachDegrees` için 18'er olmak üzere 54 özgün senaryo hazırlandı.
- Derece farkının sarılmış sınırları, clamp altında/eşit/üstünde kalan değerler, negatif ve sonsuz artışlar, signed zero, subnormal, taşma, Infinity ve NaN payload yolları karşılaştırıldı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2299 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2299/2299 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2801/2801 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `96ef27535e7a4e280fd719f9f136eced5db51563e18c3fa857889d87d58f37fe`. Birleşik referans kimliği: `04ac5e2cd34a4961d5bae6f6de20187673ac1b301ab032b064de5b4e480f2450`.

Adapter hashleri: debug `0f4aa9b98fca1253120105ce9bdba4d700baf9838ee7003352152cf487344b12`, release `4625ad416249966b9fc6663769fb2f4042336651117ef7e8f363019bc5ebe760`.

Yerel raporlar: `compare-z5am_4yy` ve `compare-90s_kim4` (math debug/release), `compare-qldg25gg` ve `compare-gbyzt4se` (birleşik debug/release), `verify-7xgrgsmb` (math) ve `verify-8o0z3frr` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki üç Mth yordamı ve bu dilimin 54 raw f32 girdisiyle sınırlıdır. `getInt`, `smallestEncompassingPowerOfTwo`, diğer `wrapDegrees` overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
