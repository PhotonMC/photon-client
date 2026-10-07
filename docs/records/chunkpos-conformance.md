# P03.b — ChunkPos.STREAM_CODEC sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **20 Java davranış testi hazırlanmıştır**. Kapsam, `ChunkPos.STREAM_CODEC` tarafından kullanılan tek long'luk x/z binary taşımasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:11-28` içindeki stream codec ile `network/FriendlyByteBuf.java:402-418` içindeki ChunkPos okuma/yazma yolları incelendi. Codec, packed long'un düşük 32 bitinde x'i, yüksek 32 bitinde z'yi taşır.

Bu sözleşme yalnızca stream binary biçimini kapsar. `ChunkPos.CODEC`, region hesapları, block koordinatı yardımcıları ve dünya/chunk yaşam döngüsü ayrı davranışlardır.

## Davranış sözleşmesi

- Encode, signed i32 x ve z değerlerini iki unsigned 32 bit alanın bitlerini değiştirmeden tek big-endian 8 baytlık long'a koyar; x düşük, z yüksek yarıdadır.
- Decode, düşük ve yüksek yarıları signed i32 olarak yeniden yorumlayıp x ve z döndürür. Değerler için geçerlilik veya dünya sınırı denetimi yapmaz.
- Decode tek bir `readLong` kullandığı için 8 bayttan kısa girdide hata verir ve giriş konumunu ilerletmez.
- Tam girdiden sonra kalan baytlar korunur; yazma mevcut prefix'in sonuna tam 8 bayt ekler.
- Senaryo girdilerinde 0–7 byte'lık partial payload sonuna `55aa` eklenir. 0–5 uzunlukları underflow üretir; 6–7 uzunluklarında bu ek baytlar long'u tamamladığı için Java bunları veri olarak tüketip başarı döndürür. Bu, fixture'ın gözlenen buffer davranışıdır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Encode | 6 | Köken, pozitif, negatif, alterne bitler ve i32 sınır çiftleri |
| Decode tam payload | 6 | Köken, küçük değerler, negatif, alterne bitler, i32 sınırları ve örnek packed long |
| Decode partial payload | 8 | 0–7 byte payload ile atomik readLong sınırı ve ek bayt tüketimi |

İstekler `family: "binary"`, `codec: "chunkpos"` ve `operation: "read" | "write"` kullanır. Encode x/z alanlarını signed decimal string olarak, read ise byte hex string olarak taşır. Başarılı sonuç x/z, `outcome`, byte tüketimi veya yazımı ve prefix/kalan hex alanlarını içerir.

## Rust karşılığı

`crates/voxel-core/src/binary.rs` içindeki `ChunkPos::pack` ve `ChunkPos::unpack`, iki 32 bit alanın yerleşimini taşır. `read_chunkpos`, yeterli veri yoksa slice'ı değiştirmeden `BufferUnderflow` döndürür; `write_chunkpos` aynı packed long'u big-endian olarak ekler. JSONL adapter bu API'yi [behavior_adapter.rs](../../crates/voxel-oracle/src/main.rs) üzerinden çağırır; beklenen değer Java oracle'dan gelir.


## Client-parity doğrulaması

`crates/voxel-core/src/binary.rs`, Java `ChunkPos.STREAM_CODEC` için packed x/z long taşımasını karşılar. Java oracle `ChunkPos.STREAM_CODEC`'i doğrudan çağırır; Rust adapter aynı input sırası, prefix ve byte sonuçlarıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos için 6 encode, 6 tam decode ve 8 partial decode olmak üzere 20 özgün senaryo hazırlandı.
- Encode sonuçları düşük 32 bit x, yüksek 32 bit z düzenini; negative ve i32 sınırlarını korudu.
- Decode sonuçları signed i32 yeniden yorumlamasını, kalan byte'ları ve kısa readLong tüketim davranışını korudu.
- Debug ve release Rust adapter'ları güncel binary ailesinde **164/164** eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda debug ve release adapter'larıyla eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını iki JVM ile yeniden doğruladı.

Binary referans kimliği: `faa64f0548d442718b7a8e5783155e27604ddd916aaf9595070c51b91232437e`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-ijck1p8v` (binary debug), `compare-vuahjf17` (binary release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `ChunkPos.STREAM_CODEC` packed long biçimini ve buffer tüketimini kapsar. `ChunkPos.CODEC`, region/block yardımcıları, chunk paketleri ve dünya yaşam döngüsü bu dilimin dışındadır. Java kaynakları, capture girdileri ve türetilmiş çıktılar `.data/` altında yerel kalır.
