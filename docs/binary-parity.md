# P03.b — Binary Rust davranış dilimi

`src/binary.rs`, Vec3 stream codec'i ile sabit ve değişken boyutlu binary kodlamaları içerir. `examples/behavior_adapter.rs` üretim kütüphanesini mevcut JSONL senaryolarına bağlar; beklenen değer üretmez. NBT ve diğer P03 biçimleri bu dilime dahil değildir.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `Vec3.STREAM_CODEC` içindeki üç `ByteBuf` double yolu ile `FriendlyByteBuf` UUID/BlockPos/long array yolları, `BlockPos`/`SectionPos` paketleme kaynak incelemesinde belirlendi. `BinaryOracle.java` doğrudan orijinal sınıfları çağırır; senaryo ve beklenen çıktılar değiştirilmedi. Ayrıntılı sözleşme ve 129 senaryoluk kapsam [binary referans belgesindedir](binary-contract.md).

| Rust API | Korunan davranış |
| --- | --- |
| `read_vec3`, `write_vec3` | Üç ardışık big-endian f64; bileşen başına atomik okuma ve raw bit koruması |
| `Uuid::new`, `read_uuid`, `write_uuid` | İki big-endian u64; her 16 bayt geçerli, tek hata kesik veri |
| `BlockPos::new`, `pack`, `unpack` | X/Z 26 bit ve Y 12 bit maskeleme ile sarma; aritmetik kaydırma ile işaret genişletme |
| `read_blockpos`, `write_blockpos` | Tek big-endian u64; eksik veride tüketim yok |
| `SectionPos::new`, `pack`, `unpack` | X/Z 22 bit ve Y 20 bit maskeleme ile sarma; aritmetik kaydırma ile işaret genişletme |
| `read_sectionpos`, `write_sectionpos` | Tek big-endian u64; eksik veride tüketim yok |
| `read_long_array`, `write_long_array` | VarInt uzunluk, negatif/aşırı uzunlukta gövde tüketilmeden `decoder_error`; önek hatalarında wire tüketimi |
| `BitSet`, `read_bitset`, `write_bitset` | Kırpılmış little-endian kelimeler; aynı bit kümesi aynı sıralı listeyi verir |
| Tüketim kuralı | Her `readLong` atomik; UUID ikinci yarıda eksikse ilk 8 bayt tüketilmiş olur |

Yazma bu dilimde her zaman başarılıdır; ayrı uzunluk sınırı yoktur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 129 senaryoluk kayıtlı binary Java referansı iki yeni JVM çalıştırmasıyla doğrulandı.
- Debug ve release Rust adaptörleri **129/129** binary senaryosunda eşleşti.
- Karşılaştırma kimlik bitleri, koordinatlar, diziler, bit listeleri, tüketim, kalan/yazılan baytlar ve hata kategorisini kapsar.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği: `560602ef9264eceeefea1f7e04180a821c7e67a9bd644f85964cbe1229e4e713`. Binary ailesi kimliği: `40e3640839f4783a6a3f1b08072a8e7679461e101d4f72b5133ddfe58e06e3d0`.

Yerel raporlar: `compare-q8wuyafn` (binary debug 129), `compare-kj2w1vpa` (binary release 129), `compare-khi93grl` (birleşik debug 1360) ve `compare-qhum5wxw` (birleşik release 1360) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
```

Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. Release için derlemeye `--release` eklenir ve `target/release/examples/behavior_adapter` seçilir.

P03.b binary kapsamı tamamlandı. Sonraki dilim bağımlılık sırasına göre seçilir.
