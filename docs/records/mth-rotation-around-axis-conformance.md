# Mth rotationAroundAxis davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `rotationAroundAxis(Vector3fc, Quaternionf, Quaternionf)` helper'ını ve sabitlenen JOML 1.10.8 runtime davranışını kapsar.

Helper, `Vector3fc.dot(float, float, float)` ile eksen ve quaternion'ın x/y/z bileşenlerinin f32 dot/FMA izdüşümünü hesaplar. İzdüşümü eksenin her bileşeniyle ayrı f32 çarpımına verir; dördüncü bileşen quaternion'ın w değeridir. Sonuç, JOML `Quaternionf.normalize()` yoluyla f32 kare toplamı, `Math.sqrt` tabanlı ters kök ve bileşen çarpımlarıyla normalize edilir.

Behavior runner eksen ve quaternion bileşenlerini sekiz haneli lowercase raw f32 bit string'leri olarak taşır. Oracle, Java `Vector3f`, `Quaternionf` ve yeni `Quaternionf` result nesnesiyle helper'ı çağırır. Sonuç x/y/z/w ham f32 bitleri altında taşınır; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:762-765`; JOML sınıfları `org.joml.Vector3f`, `org.joml.Quaternionf` ve `org.joml.Math`.

Kapsam dışı kalanlar: quaternion multiplication, slerp, axis-angle constructor'ları, matrix dönüşümleri, mutable result aliasing ve bu helper'ı kullanan istemci akışları.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Mth.rotationAroundAxis(Vector3fc, Quaternionf, Quaternionf)` işlemini JOML'nin f32 dot/FMA izdüşümü, eksen bileşen çarpımları ve quaternion normalizasyonu ile taşır. `examples/behavior_adapter.rs` sonucu x/y/z/w ham f32 bitleri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Unit/non-unit ve negatif eksenleri, sıfır ve işaretli sıfırı, FMA işlem sırasını, subnormal değerleri, f32 sınır taşmasını, Infinity/NaN ve sıfır quaternion normalizasyonunu kapsayan toplam **24 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.rotationAroundAxis` helper'ının sabit Vector3f/Quaternionf girdileri, JOML f32 FMA ve normalizasyon sonuçları içindir. Quaternion'ın diğer işlemleri, mutable result aliasing, matrix/axis-angle dönüşümleri ve helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
