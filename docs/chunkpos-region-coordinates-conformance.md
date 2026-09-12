# P01.c — ChunkPos region koordinatı yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir `ChunkPos` değerinden region koordinatlarını ve 32×32 region içindeki yerel indeksleri çıkarmak; packed ChunkPos değerinden aynı region koordinatlarını okumak; ayrıca region koordinatından ilk ve son chunk'ı üretmektir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:53-59` içindeki `minFromRegion`/`maxFromRegion` ile `world/level/ChunkPos.java:128-150` içindeki instance/static region ve local yardımcıları incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde `ChunkPos` packing biçiminin tamamı, `isValid`, `hash`, block koordinatı yardımcıları, `BlockPos` dönüşümleri ve range/distance yardımcıları ölçülmez. Packed region sorguları için packed değer aynı `ChunkPos` nesnesinin Java `pack()` sonucu olarak oluşturulur.

## Davranış sözleşmesi

- `getRegionX/Z`, chunk koordinatını 5 bit aritmetik sağa kaydırır; negatif değerlerde sonuç sıfıra doğru kesilmez.
- Static `getRegionX/Z(long)`, packed long içinden signed x/z alanlarını çıkarır ve aynı 5 bit aritmetik kaydırmayı uygular.
- `getRegionLocalX/Z`, chunk koordinatının düşük 5 bitini maskeler ve 0–31 aralığındaki region içi indeksi verir.
- `minFromRegion`, region koordinatını 5 bit sola kaydırarak ilk chunk'ı üretir.
- `maxFromRegion`, aynı ilk chunk koordinatlarına 31 ekleyerek son chunk'ı üretir.
- Java int kaydırma ve toplama taşmaları i32 içinde sarar.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| Region koordinatı yardımcıları | 6 | Köken, pozitif/negatif region sınırları, karışık koordinatlar, shift taşması ve i32 koordinat sınırları |

Girdiler `family: "math"`, `operation: "chunkpos_region_coordinates"`, chunk `x`/`z` ve region `region_x`/`region_z` alanlarını kullanır. `value` altında instance region, packed region, local indeks ve region'dan üretilen min/max `ChunkPos` snapshot'ları taşınır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::get_region_x`, `get_region_z`, `get_region_x_from_packed`, `get_region_z_from_packed`, `get_region_local_x`, `get_region_local_z`, `min_from_region` ve `max_from_region` Java'nın shift, mask ve i32 wrapping aritmetiğini korur. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java çıktısını doğrudan gözlemler.


## Client-parity doğrulaması

`src/binary.rs`, Java `ChunkPos` region ve region-local yardımcılarını signed shift, düşük 5 bit maskesi ve region min/max üretimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk/region girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Region koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Instance ve packed region sorguları, 0–31 local indeksleri, negatif chunk koordinatları ve region shift/add taşmaları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, region x/z çıkarımını, packed region sorgularını, region-local x/z maskesini ve region'dan ilk/son chunk üretimini kapsar. ChunkPos packing'in tamamı, `isValid`, hash, block koordinatları, BlockPos dönüşümleri, range/distance yardımcıları ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
