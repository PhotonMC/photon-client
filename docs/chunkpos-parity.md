# P03.b — ChunkPos stream codec doğrulaması

`src/binary.rs`, Java `ChunkPos.STREAM_CODEC` için packed x/z long taşımasını karşılar. Java oracle `ChunkPos.STREAM_CODEC`'i doğrudan çağırır; Rust adapter aynı input sırası, prefix ve byte sonuçlarıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos için 6 encode, 6 tam decode ve 8 partial decode olmak üzere 20 özgün senaryo hazırlandı.
- Encode sonuçları düşük 32 bit x, yüksek 32 bit z düzenini; negative ve i32 sınırlarını korudu.
- Decode sonuçları signed i32 yeniden yorumlamasını, kalan byte'ları ve kısa readLong tüketim davranışını korudu.
- Debug ve release Rust adapter'ları güncel binary ailesinde **164/164** eşleşti.
- Birleşik altı aile corpus'u **2185/2185** senaryoda debug ve release adapter'larıyla eşleşti.
- `verify` çalıştırması kayıtlı 2185 senaryoluk Java referansını iki JVM ile yeniden doğruladı.

Binary referans kimliği: `faa64f0548d442718b7a8e5783155e27604ddd916aaf9595070c51b91232437e`. Birleşik referans kimliği: `595049f32d6822eff3379ec8f2d5211ef192043bf7f6a11db9b9d7c609fe01a1`.

Yerel raporlar: `compare-ijck1p8v` (binary debug), `compare-vuahjf17` (binary release), `compare-63qzbune` (birleşik debug), `compare-1pb7x57b` (birleşik release) ve `verify-f_mau559` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `ChunkPos.STREAM_CODEC` packed long biçimini ve buffer tüketimini kapsar. `ChunkPos.CODEC`, region/block yardımcıları, chunk paketleri ve dünya yaşam döngüsü bu dilimin dışındadır. Java kaynakları, capture girdileri ve türetilmiş çıktılar `.data/` altında yerel kalır.
