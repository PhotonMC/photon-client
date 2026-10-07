# Mth açı sarma ve fark davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `wrapDegrees90(float)`, `degreesDifference(float, float)` ve `degreesDifferenceAbs(float, float)` yollarını kapsar.

`wrapDegrees90`, açıyı Java f32 remainder işlemiyle `90.0F` değerine göre sarar. `45.0F` ve üstü sonuçtan `90.0F` çıkarılır; `-45.0F` altı sonuca `90.0F` eklenir. Böylece sınırların karşılaştırma yönü korunarak sonuç `[-45, 45)` aralığına taşınır. Remainder sırası, signed zero ve özel değer sonuçları sözleşmenin parçasıdır.

`degreesDifference`, önce `toAngle - fromAngle` f32 çıkarımını yapar ve sonucu `wrapDegrees(float)` ile `[-180, 180)` aralığına taşır. `degreesDifferenceAbs`, bu sonuca Java `abs(float)` uygular; negatif sıfırın pozitif sıfıra ve NaN payload bitlerinin Java `Math.abs` kuralına göre davranması korunur.

Senaryo girdileri raw f32 bit string'leridir. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonucu raw f32 bit string'i olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:242-261`.

Kapsam dışı kalanlar: `rotateIfNecessary`, `approach`, `approachDegrees`, diğer `wrapDegrees` overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs` içindeki `mth_wrap_degrees_90_f32`, `mth_degrees_difference_f32` ve `mth_degrees_difference_abs_f32`, Java `Mth.wrapDegrees90`, `Mth.degreesDifference` ve `Mth.degreesDifferenceAbs` yollarını JSONL adapter'a bağlar. F32 remainder, sınır karşılaştırmaları, derece farkı sarmalama, signed zero ve özel değer sonuçları Java oracle'ın raw f32 bitleriyle karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `wrapDegrees90` için 18, `degreesDifference` için 16 ve `degreesDifferenceAbs` için 16 olmak üzere 50 özgün senaryo hazırlandı.
- 90/45 derece sınırları, signed zero, subnormal değerler, tam dönüşler, büyük sonlu farklar, Infinity ve NaN payload yolları karşılaştırıldı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2299 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2299/2299 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2801/2801 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `83ecec7ae6c7b3a479f7e803acc6f69c131c24c99c742795b3636da166600d4b`. Birleşik referans kimliği: `09f408f28585c8496c25687a9409d42bfa5efb95844c6581107fbf59b5513efa`.

Adapter hashleri: debug `e0980aa623aef291af5753b5f4e473bf636ccd4ff9d9e0649c4c685cdc7cbb3b`, release `5b142e398f14241a7208328e09597a210354a46737ee304a262800146eaaa438`.

Yerel raporlar: `compare-boooy2ze` ve `compare-ffgr9vpe` (math debug/release), `compare-r6z37qc_` ve `compare-e4dpiwut` (birleşik debug/release), `verify-9i3pah3p` (math) ve `verify-nksv5gwn` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki üç Mth yordamı ve bu dilimin 50 raw f32 girdisiyle sınırlıdır. `rotateIfNecessary`, `approach`, `approachDegrees`, diğer `wrapDegrees` overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
