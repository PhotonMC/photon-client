# P03.b — ChunkPos stream codec doğrulaması

`src/binary.rs`, Java `ChunkPos.STREAM_CODEC` için packed x/z long taşımasını karşılar. Java oracle `ChunkPos.STREAM_CODEC`'i doğrudan çağırır; Rust adapter aynı input sırası, prefix ve byte sonuçlarıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos için 6 encode, 6 tam decode ve 8 partial decode olmak üzere 20 özgün senaryo hazırlandı.
- Encode sonuçları düşük 32 bit x, yüksek 32 bit z düzenini; negative ve i32 sınırlarını korudu.
- Decode sonuçları signed i32 yeniden yorumlamasını, kalan byte'ları ve kısa readLong tüketim davranışını korudu.
- Debug ve release Rust adapter'ları güncel binary ailesinde **164/164** eşleşti.
- Birleşik altı aile corpus'u **1542/1542** senaryoda debug ve release adapter'larıyla eşleşti.
- `verify` çalıştırması kayıtlı 1542 senaryoluk Java referansını iki JVM ile yeniden doğruladı.

Binary referans kimliği: `faa64f0548d442718b7a8e5783155e27604ddd916aaf9595070c51b91232437e`. Birleşik referans kimliği: `b28d88fa06544e5713186c8c0c44bacbc4c7be8b946c7932a025574a1b3b2126`.

Yerel raporlar: `compare-ijck1p8v` (binary debug), `compare-vuahjf17` (binary release), `compare-1lz8q4ar` (birleşik debug), `compare-fvdcbn0r` (birleşik release) ve `verify-nud8oh4e` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `ChunkPos.STREAM_CODEC` packed long biçimini ve buffer tüketimini kapsar. `ChunkPos.CODEC`, region/block yardımcıları, chunk paketleri ve dünya yaşam döngüsü bu dilimin dışındadır. Java kaynakları, capture girdileri ve türetilmiş çıktılar `.data/` altında yerel kalır.
