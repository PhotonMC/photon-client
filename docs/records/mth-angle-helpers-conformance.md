# Mth açı yardımcıları davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `packDegrees(float)`, `unpackDegrees(byte)` ve `wrapDegrees(long)` yollarını kapsar.

`packDegrees`, f32 açıyı Java f32 çarpma/bölme sırasıyla 256/360 ölçeğine taşır, `floor` uygular ve sonucu byte genişliğine daraltır. NaN, sonsuzluk ve 256 katı sınırlarında Java'nın float-to-int ve int-to-byte dönüşümleri korunur. `unpackDegrees`, signed byte değeri önce int çarpımına, ardından f32 bölmeye çevirir.

`wrapDegrees(long)`, long açıdan Java signed `% 360L` kalanını alır ve sonucu f32'ye daraltır. `180` ve üstü 360 azaltılır, `-180` altı 360 artırılır; sonuç `[-180, 180)` aralığındadır. Long sınırlarında kalan ve son f32 bitleri gözlenebilir sözleşmenin parçasıdır.

Senaryo girdileri pack için raw f32 bitleri, unpack için signed byte decimal string'i, long wrap için signed long decimal string'i taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonucu byte decimal string'i veya raw f32 bitleri olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:178-188` (`packDegrees`, `unpackDegrees`), `:203-214` (`wrapDegrees(long)`).

Kapsam dışı kalanlar: `wrapDegrees(int)` ve float/double overload'ları, `wrapDegrees90`, derece farkı/rotation yardımcıları, sıfıra bölme hata biçimi ve diğer Mth açı/tamsayı yardımcıları.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Mth` açı paketleme, açma ve long açı sarmalama yardımcılarını f32 daraltma, byte daraltma ve signed kalan kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `packDegrees`, `unpackDegrees` ve `wrapDegrees(long)` için 21 özgün senaryo hazırlandı.
- F32 açı sınırları, NaN/Infinity, signed byte uçları ve `Long.MIN_VALUE`/`Long.MAX_VALUE` kalanları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.packDegrees(float)`, `unpackDegrees(byte)` ve `wrapDegrees(long)` sonuçlarıyla sınırlıdır. Diğer `wrapDegrees` overload'ları, derece farkı/rotation yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
