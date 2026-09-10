# Mth rotationAroundAxis davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `rotationAroundAxis(Vector3fc, Quaternionf, Quaternionf)` helper'ını ve sabitlenen JOML 1.10.8 runtime davranışını kapsar.

Helper, `Vector3fc.dot(float, float, float)` ile eksen ve quaternion'ın x/y/z bileşenlerinin f32 dot/FMA izdüşümünü hesaplar. İzdüşümü eksenin her bileşeniyle ayrı f32 çarpımına verir; dördüncü bileşen quaternion'ın w değeridir. Sonuç, JOML `Quaternionf.normalize()` yoluyla f32 kare toplamı, `Math.sqrt` tabanlı ters kök ve bileşen çarpımlarıyla normalize edilir.

Behavior runner eksen ve quaternion bileşenlerini sekiz haneli lowercase raw f32 bit string'leri olarak taşır. Oracle, Java `Vector3f`, `Quaternionf` ve yeni `Quaternionf` result nesnesiyle helper'ı çağırır. Sonuç x/y/z/w ham f32 bitleri altında taşınır; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:762-765`; JOML sınıfları `org.joml.Vector3f`, `org.joml.Quaternionf` ve `org.joml.Math`.

Kapsam dışı kalanlar: quaternion multiplication, slerp, axis-angle constructor'ları, matrix dönüşümleri, mutable result aliasing ve bu helper'ı kullanan istemci akışları.
