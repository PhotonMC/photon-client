# P03.b — Binary Rust davranış dilimi

`src/binary.rs`, sabit ve değişken boyutlu binary kodlamaları içerir. `examples/behavior_adapter.rs` üretim kütüphanesini mevcut JSONL senaryolarına bağlar; beklenen değer üretmez. NBT ve diğer P03 biçimleri bu dilime dahil değildir.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `FriendlyByteBuf` içindeki UUID/BlockPos/long array yolları ve `BlockPos` paketleme kaynak incelemesinde belirlendi. `BinaryOracle.java` doğrudan orijinal sınıfları çağırır; senaryo ve beklenen çıktılar değiştirilmedi. Ayrıntılı sözleşme ve 92 senaryoluk kapsam [binary referans belgesindedir](binary-contract.md).

| Rust API | Korunan davranış |
| --- | --- |
| `Uuid::new`, `read_uuid`, `write_uuid` | İki big-endian u64; her 16 bayt geçerli, tek hata kesik veri |
| `BlockPos::new`, `pack`, `unpack` | X/Z 26 bit ve Y 12 bit maskeleme ile sarma; aritmetik kaydırma ile işaret genişletme |
| `read_blockpos`, `write_blockpos` | Tek big-endian u64; eksik veride tüketim yok |
| `read_long_array`, `write_long_array` | VarInt uzunluk, negatif/aşırı uzunlukta gövde tüketilmeden `decoder_error`; önek hatalarında wire tüketimi |
| `BitSet`, `read_bitset`, `write_bitset` | Kırpılmış little-endian kelimeler; aynı bit kümesi aynı sıralı listeyi verir |
| Tüketim kuralı | Her `readLong` atomik; UUID ikinci yarıda eksikse ilk 8 bayt tüketilmiş olur |

Yazma bu dilimde her zaman başarılıdır; ayrı uzunluk sınırı yoktur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 579 senaryoluk kayıtlı Java referansı iki yeni JVM çalıştırmasıyla doğrulandı.
- Debug ve release Rust adaptörleri **579/579** eşleşti: 180 math + 22 timer + 144 wire + 141 utf8 + 92 binary.
- Karşılaştırma kimlik bitleri, koordinatlar, diziler, bit listeleri, tüketim, kalan/yazılan baytlar ve hata kategorisini kapsar.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği: `6e38fbb2e12abbb1ef3efcb8889bf9b709b769375509d6ddb41a3827e44e2b59`. Binary ailesi kimliği: `72017ab2c353dd2343a27966d37bb45eb419baa4894c1dc891321f5c5a38c42b`.

Yerel raporlar: `compare-eb7mz513` (debug 579), `compare-_e8wf5_y` (release 579) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
```

Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. Release için derlemeye `--release` eklenir ve `target/release/examples/behavior_adapter` seçilir.

P03.b kapsamında sabit BitSet varyantları da uygulandı. Sonraki dilim bağımlılık sırasına göre seçilir.
