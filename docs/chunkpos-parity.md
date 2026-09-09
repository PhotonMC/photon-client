# P03.b — ChunkPos stream codec doğrulaması

`src/binary.rs`, Java `ChunkPos.STREAM_CODEC` için packed x/z long taşımasını karşılar. Java oracle `ChunkPos.STREAM_CODEC`'i doğrudan çağırır; Rust adapter aynı input sırası, prefix ve byte sonuçlarıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos için 6 encode, 6 tam decode ve 8 partial decode olmak üzere 20 özgün senaryo hazırlandı.
- Encode sonuçları düşük 32 bit x, yüksek 32 bit z düzenini; negative ve i32 sınırlarını korudu.
- Decode sonuçları signed i32 yeniden yorumlamasını, kalan byte'ları ve kısa readLong tüketim davranışını korudu.
- Debug ve release Rust adapter'ları güncel binary ailesinde **164/164** eşleşti.
- Birleşik altı aile corpus'u **1651/1651** senaryoda debug ve release adapter'larıyla eşleşti.
- `verify` çalıştırması kayıtlı 1651 senaryoluk Java referansını iki JVM ile yeniden doğruladı.

Binary referans kimliği: `faa64f0548d442718b7a8e5783155e27604ddd916aaf9595070c51b91232437e`. Birleşik referans kimliği: `f8cfd3cf28e07e5859a339a4844b50e474cb8351ea003bb36ba96ccaf38abec1`.

Yerel raporlar: `compare-ijck1p8v` (binary debug), `compare-vuahjf17` (binary release), `compare-nc1496mg` (birleşik debug), `compare-ljx3yrp0` (birleşik release) ve `verify-k950tc5l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `ChunkPos.STREAM_CODEC` packed long biçimini ve buffer tüketimini kapsar. `ChunkPos.CODEC`, region/block yardımcıları, chunk paketleri ve dünya yaşam döngüsü bu dilimin dışındadır. Java kaynakları, capture girdileri ve türetilmiş çıktılar `.data/` altında yerel kalır.
