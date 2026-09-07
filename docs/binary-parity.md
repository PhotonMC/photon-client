# P03.b — UUID ve BlockPos Rust davranış dilimi

`src/binary.rs`, sabit boyutlu UUID ve BlockPos kodlamalarını içerir. `examples/behavior_adapter.rs` üretim kütüphanesini mevcut JSONL senaryolarına bağlar; beklenen değer üretmez. Long array, BitSet ve diğer P03 biçimleri bu dilime dahil değildir.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `FriendlyByteBuf` içindeki UUID/BlockPos yolları ve `BlockPos` paketleme kaynak incelemesinde belirlendi. `BinaryOracle.java` doğrudan orijinal sınıfları çağırır; senaryo ve beklenen çıktılar değiştirilmedi. Ayrıntılı sözleşme ve 43 senaryoluk kapsam [binary referans belgesindedir](binary-contract.md).

| Rust API | Korunan davranış |
| --- | --- |
| `Uuid::new`, `read_uuid`, `write_uuid` | İki big-endian u64; her 16 bayt geçerli, tek hata kesik veri |
| `BlockPos::new`, `pack`, `unpack` | X/Z 26 bit ve Y 12 bit maskeleme ile sarma; aritmetik kaydırma ile işaret genişletme |
| `read_blockpos`, `write_blockpos` | Tek big-endian u64; eksik veride tüketim yok |
| Tüketim kuralı | Her `readLong` atomik; UUID ikinci yarıda eksikse ilk 8 bayt tüketilmiş olur |

Yazma bu dilimde her zaman başarılıdır; ayrı uzunluk sınırı yoktur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 530 senaryoluk kayıtlı Java referansı iki yeni JVM çalıştırmasıyla doğrulandı.
- Debug ve release Rust adaptörleri **530/530** eşleşti: 180 math + 22 timer + 144 wire + 141 utf8 + 43 binary.
- Karşılaştırma kimlik bitleri, koordinatlar, tüketim, kalan/yazılan baytlar ve hata kategorisini kapsar.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği: `88117b64f9b5f22bcca4fa95b542409ee781a26606da3861746e9673fd4fac3a`. Binary ailesi kimliği: `f320872a5c6e2a905ccea0ceb896ff62c7ce4ffeac34433b21563287d815d749`.

Yerel raporlar: `compare-1iv2wlbr` (debug 530), `compare-ct2_db7d` (release 530) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
```

Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. Release için derlemeye `--release` eklenir ve `target/release/examples/behavior_adapter` seçilir.

P03.b kapsamında long array ve bitset henüz uygulanmadı. Sonraki dilim bağımlılık sırasına göre seçilir.
