# P01.c — ChunkPos section node dönüşümü doğrulaması

`src/binary.rs`, Java `ChunkPos.fromSectionNode(long)` yolunu SectionPos packed long’undan signed x/z alanlarını çıkarıp ChunkPos packed long’una dönüştürerek karşılar. Rust adaptörü sonucu Java ile aynı unsigned lowercase hex biçiminde taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos section node dönüşümü için 6 özgün senaryo hazırlandı.
- Pozitif/negatif section koordinatları, signed 22 bit minimum/maksimum uçları ve y alanının atılması eşleşti.
- Rust debug ve release adaptörleri güncel **1193/1193** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **1695/1695** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1695 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `3a6a2f5a3d8ff6b65e565413ca95aeeb802f3ca965620ed874e898bcc52c5d61`. Birleşik referans kimliği: `5f58cd401556c40d3fc3eb7c3e511a8dbc2553c3390034c888e648d9d8a3ad4c`.

Yerel raporlar: `compare-dgq7mp_7` (math debug), `compare-2hmy86uj` (math release), `compare-06i9lguy` (birleşik debug), `compare-r6guli2r` (birleşik release) ve `verify-k_vt31c0` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.fromSectionNode(long)` dönüşümünün x/z çıkarma, y atma ve packed long yerleşimini kapsar. SectionPos üreticileri, node offset işlemleri, BlockPos dönüşümleri, `ChunkPos.rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
