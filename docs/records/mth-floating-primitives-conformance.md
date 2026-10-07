# Mth kayan nokta temel yardımcıları davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `sin(double)`, `cos(double)`, `sqrt(float)`, `abs(float)` ve `clamp(float/double)` yordamlarını kapsar. Her sin/cos yolu için 10, diğer yollar için 10 sentetik senaryo hazırlanır.

`sin` ve `cos`, radyan double girdisini 65536 kuantizasyonlu Mth lookup tablosunda aynı ölçek ve maske ile indeksler; tabloda bulunan f32 değeri döndürür. `sqrt`, f32 girdiyi Java `Math.sqrt` double işlemine verip sonucu f32'ye daraltır. `abs`, f32 işaret bitini kaldırır ve NaN payload'ını koruyan Java `Math.abs(float)` davranışını izler.

`clamp` önce değerin alt sınıra göre strict karşılaştırmasını yapar, aksi durumda Java `Math.min(value, max)` yolunu izler. Bu nedenle ters sınırlar, NaN ve signed zero sonuçları genel bir aralık varsayımıyla değiştirilemez; f32 ve f64 overload'ları kendi hassasiyetinde çalışır.

Behavior runner sin/cos girdilerini raw f64, sqrt/abs/f32 clamp girdilerini raw f32 ve f64 clamp girdilerini raw f64 IEEE-754 bit string'leriyle taşır. Java oracle `Mth` çağrılarını doğrudan yürütür; sonuçlar raw bit string olarak döner. Beklenen değerler Rust uygulamasından türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:29-92`.

Kapsam dışı kalanlar: `floor`, `lfloor`, `ceil`, `ceilLong`, integer `abs/clamp`, `clampedLerp` ve Mth lookup tablosunu kullanan Vec2/Vec3 çağrı zincirleri.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs` içindeki `mth_sin_f64`, `mth_cos_f64`, `mth_sqrt_f32`, `mth_abs_f32` ve mevcut hassasiyetli clamp yolları, Java `Mth` kayan nokta temel yordamlarını JSONL adapter'a bağlar. Adapter beklenen değerleri üretmez; her sonuç Java oracle çıktısındaki raw IEEE-754 bitleriyle karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `sin` ve `cos` için 10'ar, `sqrt`, `abs` ve f32/f64 `clamp` için 10'ar olmak üzere 60 özgün senaryo hazırlandı. Lookup kuantizasyonu, f32/f64 hassasiyeti, signed zero, subnormal, Infinity, NaN payload'ı ve ters clamp sınırları kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2299 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2299/2299 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2801/2801 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki Mth kayan nokta temel yordamları ve bu dilimin 60 girdisiyle sınırlıdır. `floor`, `ceil`, `lfloor`, `clampedLerp` ve diğer Mth kullanıcıları ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
