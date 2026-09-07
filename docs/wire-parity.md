# P03.a — Rust VarInt/VarLong davranış dilimi

İlk Rust davranış dilimi `src/wire.rs` içinde. `examples/behavior_adapter.rs`, üretim kütüphanesini mevcut JSONL senaryolarına bağlar; kendi codec algoritmasını veya beklenen sonuçlarını içermez. `src/main.rs` hâlâ boş uygulama iskeletidir. Ağ bağlantısı, paket framing'i ve diğer P03 veri biçimleri henüz uygulanmadı.

## Java sözleşmesi

Referans: Java Edition 26.2, protokol 776; orijinal client SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`. Kaynak köküne göre `net/minecraft/network/VarInt.java` ve `VarLong.java` içindeki `getByteSize`, `read`, `write` metotları yeniden incelendi. Mevcut Java adaptörleri doğrudan bu metotları çağırır.

- İşaretli i32/i64 bit desenleri kodlanır; ZigZag dönüşümü yok. Negatif i32 beş, negatif i64 on bayt kullanır.
- Decoder Java'nın kabul ettiği kısa olmayan temsilleri kabul eder. Son veri grubunda tür genişliğini aşan bitlerin bulunması tek başına hata değildir; sonuç referanstaki tür genişliğine daralır.
- VarInt'in beş, VarLong'un on devam baytı ardından veri biterse `BufferUnderflow` oluşur. Sonraki bayt varsa o bayt da tüketilir ve `TooBig` oluşur; sonraki baytın devam bitinin sıfır olması bunu değiştirmez.
- Başarılı veya hatalı okuma sonrasında tüketilen baytlar geri alınmaz. Kalan veri aynı sırayla korunur. Gelecekteki ağ frame okuyucusunun eksik veri için yeniden deneme/rollback kararı bu düşük seviyeli decoder'ın dışındadır.

## Rust API

| API | Davranış |
| --- | --- |
| `read_varint(&mut &[u8])` | `Result<i32, ReadError>`; verilen slice okunmamış veriyi gösterecek şekilde ilerler |
| `read_varlong(&mut &[u8])` | Aynı tüketim sözleşmesi, i64 sonuç |
| `write_varint(i32, &mut Vec<u8>)` | VarInt baytlarını mevcut buffer sonuna ekler |
| `write_varlong(i64, &mut Vec<u8>)` | VarLong baytlarını mevcut buffer sonuna ekler |
| `varint_size(i32)`, `varlong_size(i64)` | Kodlanmış bayt sayısı |

Üretim kütüphanesinin haricî bağımlılığı yok. `serde_json` yalnızca örnek test adaptörü için `dev-dependencies` altında; sürüm Cargo.lock ile sabit. JSONL alanları, hex/ondalık dönüşümleri ve hata kategorisi eşlemesi test adaptöründe kalır. Math/timer aileleri adaptör tarafından açık hata ile reddedilir.

## Doğrulama

Mevcut 144 wire senaryosunun girdileri ve Java beklentileri Rust'a uydurulmadan kullanıldı. Rust kodu yazılmadan önce aileye ait referans açıkça yakalandı; iki JVM çalıştırması eşleşti. Referans kimliği:

`b92f6e6ffca85f499f3d28192c96d88f23547046fc137dcfc983f187dc2f0b2b`

Rust adaptörüyle debug ve release profillerinde 144/144 senaryo eşleşti. Karşılaştırma baytları, değerleri, boyutu, hata kategorisini, tüketilen bayt sayısını ve kalan veriyi kapsıyor. Bu sonuç yalnızca mevcut corpus ve Linux x86_64 ortamı için P03.a kanıtıdır; bütün protokol veya istemci eşitliği değildir.

Ek doğrulama: desteklenmeyen aile/şema, i32 dışındaki yazma girdisi ve bozuk/yarım hex girdileri adaptörden başarısız çıktı kodu aldı. `cargo fmt --all --check` ve `cargo clippy --offline --all-targets -- -D warnings` geçti. Paket listesi orijinal kaynakları, `.data/` kayıtlarını ve test adaptörünü içermiyor.

Ham raporlar `.data/runs/behavior/compare-*/report.json` içinde; seçilen corpus, referans kimliği ve derlenen adaptör SHA-256'sı rapora yazılır. Java exception sınıfı yalnızca tanısal alan; Rust ile karşılaştırılan hata kategorisi `result.error` içindedir.

## Yeniden çalıştırma

Depo kökünde, referans dosyaları yerelde mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py capture --family wire
python3 tools/behavior/run.py compare --family wire --adapter target/debug/examples/behavior_adapter
```

`capture` mevcut kimliğin beklentilerini değiştirmez; yeni checkout veya yeni kimlikte açık yakalama sağlar. Yalnızca mevcut Java kaydını doğrulamak için `verify --family wire` kullanılabilir. Release denemesi için `cargo build --locked --offline --release --example behavior_adapter` ve `--adapter target/release/examples/behavior_adapter` kullanılır. Offline derleme için Cargo.lock'taki geliştirme bağımlılıkları Cargo önbelleğinde bulunmalıdır.

Sıradaki bağımsız küçük dilim P02.a zamanlayıcısıdır; mevcut 22 Java senaryosuna Rust adaptörü bağlanacak. Math ve timer ailelerindeki toplam 202 senaryo şu an yalnızca Java referansı olarak hazırdır.
