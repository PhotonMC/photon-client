# P03.b — Binary Rust davranış dilimi

`src/binary.rs`, Vec3 stream codec'leri ile sabit ve değişken boyutlu binary kodlamaları içerir. `examples/behavior_adapter.rs` üretim kütüphanesini mevcut JSONL senaryolarına bağlar; beklenen değer üretmez. NBT ve diğer P03 biçimleri bu dilime dahil değildir.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `Vec3.STREAM_CODEC` içindeki üç `ByteBuf` double yolu, `Vec3.LP_STREAM_CODEC`/`LpVec3` packed yolu ve `FriendlyByteBuf` UUID/BlockPos/long array yolları, `BlockPos`/`SectionPos` paketleme kaynak incelemesinde belirlendi. `BinaryOracle.java` doğrudan orijinal sınıfları çağırır; senaryo ve beklenen çıktılar değiştirilmedi. Ayrıntılı sözleşme ve 144 senaryoluk kapsam [binary referans belgesindedir](binary-contract.md); LP Vec3 alt kapsamı [ayrı parity belgesindedir](lp-vec3-parity.md).

| Rust API | Korunan davranış |
| --- | --- |
| `read_vec3`, `write_vec3` | Üç ardışık big-endian f64; bileşen başına atomik okuma ve raw bit koruması |
| `read_lp_vec3`, `write_lp_vec3` | 15 bit x/y/z quantization, 2 bit/continuation ölçek, zero marker, sanitize/clamp ve packed byte sırası |
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

- 144 senaryoluk kayıtlı binary Java referansı iki yeni JVM çalıştırmasıyla doğrulandı.
- LP Vec3 alt kümesindeki 6 yazma ve 9 okuma senaryosu Java/Rust byte ve sonuç eşitliğiyle doğrulandı.
- Debug ve release Rust adaptörleri **144/144** binary senaryosunda eşleşti.
- Karşılaştırma kimlik bitleri, koordinatlar, diziler, bit listeleri, tüketim, kalan/yazılan baytlar ve hata kategorisini kapsar.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği: `7c9538198afb189201c05520d1d9beb9d8b8ed1ecf818a5751ba7c9cf2774c30`. Binary ailesi kimliği: `5324fd28d97aff4539ad52a38d5e137805c4f43aca2081ac1b9fa8b1b221123b`.

Yerel raporlar: `compare-qqvsym4q` (binary debug 144), `compare-7tecixjf` (binary release 144), `compare-wdb79blv` (birleşik debug 1375) ve `compare-_hlw5kud` (birleşik release 1375) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
```

Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. Release için derlemeye `--release` eklenir ve `target/release/examples/behavior_adapter` seçilir.

P03.b binary kapsamı tamamlandı. Sonraki dilim bağımlılık sırasına göre seçilir.
