# P03.b — ChunkPos stream codec doğrulaması

`src/binary.rs`, Java `ChunkPos.STREAM_CODEC` için packed x/z long taşımasını karşılar. Java oracle `ChunkPos.STREAM_CODEC`'i doğrudan çağırır; Rust adapter aynı input sırası, prefix ve byte sonuçlarıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos için 6 encode, 6 tam decode ve 8 partial decode olmak üzere 20 özgün senaryo hazırlandı.
- Encode sonuçları düşük 32 bit x, yüksek 32 bit z düzenini; negative ve i32 sınırlarını korudu.
- Decode sonuçları signed i32 yeniden yorumlamasını, kalan byte'ları ve kısa readLong tüketim davranışını korudu.
- Debug ve release Rust adapter'ları güncel binary ailesinde **164/164** eşleşti.
- Birleşik altı aile corpus'u **2626/2626** senaryoda debug ve release adapter'larıyla eşleşti.
- `verify` çalıştırması kayıtlı 2626 senaryoluk Java referansını iki JVM ile yeniden doğruladı.

Binary referans kimliği: `faa64f0548d442718b7a8e5783155e27604ddd916aaf9595070c51b91232437e`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-ijck1p8v` (binary debug), `compare-vuahjf17` (binary release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `ChunkPos.STREAM_CODEC` packed long biçimini ve buffer tüketimini kapsar. `ChunkPos.CODEC`, region/block yardımcıları, chunk paketleri ve dünya yaşam döngüsü bu dilimin dışındadır. Java kaynakları, capture girdileri ve türetilmiş çıktılar `.data/` altında yerel kalır.
