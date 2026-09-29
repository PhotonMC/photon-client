# Mth murmurHash3Mixer davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `murmurHash3Mixer(int)` yordamını kapsar. Yordam tek bir signed i32 girdiyi dört unsigned sağa kaydırma/XOR ve iki i32 çarpma adımıyla karıştırır.

Java `>>>` adımları, girdinin signed olması durumunda üst bitleri sıfırla doldurur. Her çarpma Java i32 genişliğinde wrapping davranışla yürür; son XOR da signed i32 sonucu korur. Çıktı signed decimal i32 string olarak taşınır.

Java oracle `Mth.murmurHash3Mixer` yordamını doğrudan çağırır; beklenen değer Rust uygulamasından türetilmez. Senaryolar sıfır, işaretli sınırlar, bit desenleri, kaydırma bitleri ve çarpma taşmalarını kapsar.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:516-522`.

Kapsam dışı kalanlar: `binarySearch`, `rayIntersectsAABB`, `atan2`, ters karekök yardımcıları ve renk dönüşümleri.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs` içindeki `mth_murmur_hash3_mixer`, Java `Mth.murmurHash3Mixer(int)` yolunu JSONL adapter'a bağlar. Unsigned sağa kaydırma/XOR adımları ve i32 wrapping çarpımları Java oracle'ın signed `value` sonucu ile karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Sıfır, signed i32 sınırları, yüksek ve düşük bitler, bit desenleri, kaydırma konumları ve çarpma taşmaları için 24 özgün senaryo hazırlandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2299 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2299/2299 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2801/2801 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb82`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.murmurHash3Mixer` yordamı ve bu dilimin 24 signed i32 girdisiyle sınırlıdır. `binarySearch`, `rayIntersectsAABB`, `atan2`, ters karekök yardımcıları ve renk dönüşümleri ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
