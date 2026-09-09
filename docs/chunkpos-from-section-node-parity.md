# P01.c — ChunkPos section node dönüşümü doğrulaması

`src/binary.rs`, Java `ChunkPos.fromSectionNode(long)` yolunu SectionPos packed long’undan signed x/z alanlarını çıkarıp ChunkPos packed long’una dönüştürerek karşılar. Rust adaptörü sonucu Java ile aynı unsigned lowercase hex biçiminde taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos section node dönüşümü için 6 özgün senaryo hazırlandı.
- Pozitif/negatif section koordinatları, signed 22 bit minimum/maksimum uçları ve y alanının atılması eşleşti.
- Rust debug ve release adaptörleri güncel **965/965** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **1467/1467** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1467 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `bbc2c1dc1c594cc962e6acbf6fcdc98815c896f66dcad6a72c2da4be5b0a0f81`. Birleşik referans kimliği: `5cb121b3bc1e12571ad171166a785300fd6e581cf78813e4ddaf2b99d19ab7d1`.

Yerel raporlar: `compare-ctepk6ji` (math debug), `compare-5wubt_if` (math release), `compare-lsdsnr89` (birleşik debug), `compare-b5y_ka7p` (birleşik release) ve `verify-o3reu3wd` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.fromSectionNode(long)` dönüşümünün x/z çıkarma, y atma ve packed long yerleşimini kapsar. SectionPos üreticileri, node offset işlemleri, BlockPos dönüşümleri, `ChunkPos.rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
