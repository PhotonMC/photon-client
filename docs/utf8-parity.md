# P03.b — UTF-8 string Rust davranış dilimi

`src/utf8.rs`, `Utf8String` okuma/yazma davranışını kapsar. `examples/behavior_adapter.rs` üretim kütüphanesini mevcut JSONL senaryolarına bağlar; beklenen değer üretmez. `src/main.rs` hâlâ boştur. UUID, konum, bitset ve diğer P03 biçimleri bu dilime dahil değildir.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `Utf8String` okuma/yazma yolları kaynak incelemesinde belirlendi. `Utf8Oracle.java` doğrudan orijinal sınıfı çağırır; senaryo ve beklenen çıktılar değiştirilmedi. Ayrıntılı sözleşme ve 141 senaryoluk kapsam [UTF-8 referans belgesindedir](utf8-contract.md).

| Rust API | Korunan davranış |
| --- | --- |
| `utf8_max_bytes` | Kodlanmış sınır `3 × max_length` olarak hesaplanır |
| `read_utf8(&mut &[u8], usize)` | VarInt başlığı, kodlanmış/çözülmüş uzunluk ve kullanılabilir veri denetimleri; hata durumunda referansın tüketim/kalan veri davranışı |
| `write_utf8(&mut Vec<u8>, &[u16], usize)` | UTF-16 birim sınırı, Netty uyumlu kodlama ve kodlanmış sınır; hatada hedefe ekleme yapılmaz |
| Dahili Java uyumlu çözümleyici | Geçerli diziler, bozuk/overlong/kesik dizilerde replacement gruplaması, surrogate ve aralık dışı diziler |
| Dahili Netty uyumlu kodlayıcı | Eşleşmemiş surrogate sonrası birimin ham bayt davranışı dahil lone ve eşleşmemiş surrogate eşlemesi |

Yazma girdisi UTF-16 birimleri olarak alınır; yalnızca `&str` kabul etmek eşleşmemiş surrogate girdilerini temsil edemez. Okuma çıktısı UTF-16 birimleridir; başarılı çözümleme eşleşmemiş surrogate içermez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 487 senaryoluk kayıtlı Java referansı iki yeni JVM çalıştırmasıyla doğrulandı.
- Debug ve release Rust adaptörleri **487/487** eşleşti: 180 math + 22 timer + 144 wire + 141 utf8.
- Karşılaştırma metin birimleri, uzunluk, tüketim, kalan/yazılan baytlar ve hata kategorilerini kapsar; float toleransı yoktur.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği: `9b7f4d3c9c59cddd1ed96162a13c8f308f8d38df91a69d445ec5f31bc0278799`. UTF-8 ailesi kimliği: `f8d898f024e107b2f8029166e17d4b16282945620c06c5a01a1c4739a43e5935`.

Yerel raporlar: `verify-cwrm1vpb`, `compare-irkxihos` (debug 487), `compare-ynbbkpt5` (release 487) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
```

Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. Release için derlemeye `--release` eklenir ve `target/release/examples/behavior_adapter` seçilir.

P03.b kapsamında UUID, konum ve bitset henüz uygulanmadı. Sonraki dilim bağımlılık sırasına göre seçilir.
