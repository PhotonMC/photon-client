# Mth kayan nokta temel yardımcıları davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `sin(double)`, `cos(double)`, `sqrt(float)`, `abs(float)` ve `clamp(float/double)` yordamlarını kapsar. Her sin/cos yolu için 10, diğer yollar için 10 sentetik senaryo hazırlanır.

`sin` ve `cos`, radyan double girdisini 65536 kuantizasyonlu Mth lookup tablosunda aynı ölçek ve maske ile indeksler; tabloda bulunan f32 değeri döndürür. `sqrt`, f32 girdiyi Java `Math.sqrt` double işlemine verip sonucu f32'ye daraltır. `abs`, f32 işaret bitini kaldırır ve NaN payload'ını koruyan Java `Math.abs(float)` davranışını izler.

`clamp` önce değerin alt sınıra göre strict karşılaştırmasını yapar, aksi durumda Java `Math.min(value, max)` yolunu izler. Bu nedenle ters sınırlar, NaN ve signed zero sonuçları genel bir aralık varsayımıyla değiştirilemez; f32 ve f64 overload'ları kendi hassasiyetinde çalışır.

Behavior runner sin/cos girdilerini raw f64, sqrt/abs/f32 clamp girdilerini raw f32 ve f64 clamp girdilerini raw f64 IEEE-754 bit string'leriyle taşır. Java oracle `Mth` çağrılarını doğrudan yürütür; sonuçlar raw bit string olarak döner. Beklenen değerler Rust uygulamasından türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:29-92`.

Kapsam dışı kalanlar: `floor`, `lfloor`, `ceil`, `ceilLong`, integer `abs/clamp`, `clampedLerp` ve Mth lookup tablosunu kullanan Vec2/Vec3 çağrı zincirleri.
