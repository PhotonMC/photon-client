# Mth outFromOrigin davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `outFromOrigin(int, int, int)` ve `outFromOrigin(int, int, int, int)` overload'larını kapsar.

Üç argümanlı overload, dört argümanlı yolu `stepSize = 1` ile çağırır. Dört argümanlı yol önce `lowerBound <= upperBound` ve `stepSize >= 1` koşullarını denetler. İlk koşul bozulduğunda üst sınır ve alt sınır değerlerini içeren Java `IllegalArgumentException` metni; ikinci koşul bozulduğunda adım değerini içeren ayrı metin oluşur.

Geçerli girdide origin, iki sınır arasına clamp edilerek stream'in ilk değeri olur. Stream predicate'i mevcut cursor'ın origin etrafındaki iki yöndeki mesafelerden en az birinin aralıkta kalıp kalmadığını denetler. Sonraki cursor, Java'nın kaynak sırasına göre pozitif aday ve negatif aday arasında ilerler; aralıkta yalnızca bir yön kaldığında o yön sürdürülür. Liste sırası bu lazy `IntStream` akışının tüketim sırasıdır.

Mesafe, toplama, çıkarma ve karşılaştırmalar Java i32 aritmetiğiyle gözlenir. Bu dilimin sonlu corpus girdileri stream'in sonlanmasını koruyan dar aralıklardır; büyük taşmalı aralıklar ve pratikte sonlanmayabilecek overflow döngüleri kapsama alınmadı.

Behavior runner girdilerinde `mode: "default"` üç argümanlı overload'u, `mode: "step"` dört argümanlı overload'u seçer. Signed i32 alanları decimal string'dir. Başarılı sonuç `outcome: "value"` ve JSON i32 listesi; doğrulama hatası `outcome: "error"` ve Java hata metni olarak taşınır. Java exception sınıf adı yalnızca tanısaldır ve sonuç karşılaştırmasına girmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:730-760`.

Kapsam dışı kalanlar: overflow ile sonlanmayan veya çok büyük stream'ler, bu helper'ın istemci çağrı zincirleri ve sonraki `Mth` yardımcıları.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Mth.outFromOrigin` üç ve dört argümanlı overload'larının sınır doğrulamasını, origin clamp'ini ve lazy stream tüketim sırasını i32 sarmasıyla taşır. `examples/behavior_adapter.rs` aynı sonucu JSON i32 listesi veya Java hata metni olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Default ve explicit step overload'larını, origin'in alt/üst sınıra clamp edilmesini, singleton/asimetrik aralıkları, aralığı aşan adımı, i32 origin sınırlarını ve lower/upper ile step hatalarını kapsayan toplam **24 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.outFromOrigin` overload'larının sabit girdili, sonlanan stream sonuçlarına ve mevcut corpus regresyonuna aittir. Büyük overflow aralıkları, sonlanmayan akış olasılıkları ve bu helper'ı kullanan istemci davranışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
