# P06–P08 — Vanilla 26.2 oyun oturumu doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`
**Sonuç:** Rust probe iki ardışık koşuda login, configuration, play ve ilk chunk akışını tamamladı.

JAR, yerel 26.2 sürüm manifestindeki resmi server nesnesiyle boyut ve SHA-1 bakımından eşleşti. Sunucu geçici `/tmp` dizininde, `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5` ve creative/peaceful ayarlarıyla başlatıldı. JAR, kaynak veya dünya dosyası depoya taşınmadı.

## Rust probe koşuları

```text
cargo run --offline --example connection_probe -- 127.0.0.1:25566 VanillaProbe2
cargo run --offline --example connection_probe -- 127.0.0.1:25566 VanillaProbe3
```

Her iki koşuda vanilla server logu oyuncunun `joined the game` ve probe kapanırken `left the game` kayıtlarını verdi; sunucu tarafında timeout oluşmadı.

| Gözlem | VanillaProbe2 | VanillaProbe3 |
| --- | ---: | ---: |
| compression threshold | 256 | 256 |
| login finished | `true` | `true` |
| configuration finished | `true` | `true` |
| enabled feature | 1 | 1 |
| known pack | 1 | 1 |
| registry packet | 29 | 29 |
| registry entry | 397 | 397 |
| update tags packet | 1 | 1 |
| play login | `true` | `true` |
| chunk packet | 24 | 24 |
| chunk batch finished | 6 | 6 |
| received packet | 175 | 180 |

Toplam paket sayısı, probe'un 250 ms drain penceresinde sunucunun paketleri gönderme zamanlamasına bağlıdır. Phase geçişleri ve yukarıdaki içerik sayaçları başarı ölçütleridir.

## Yakalanan istemci hatası

İlk vanilla probe koşusu (`VanillaProbe1`) eski kodla `UnexpectedEof` ile kapandı; vanilla server oyuncuyu önce kabul etti, sonra geçersiz keep-alive yanıtı nedeniyle timeout kaydetti. Java 26.2 packet dispatch sırası yeniden kontrol edildiğinde `43` numarasının `initialize_border`, `44` numarasının gerçek play `keep_alive` olduğu görüldü. Rust dispatch sabiti `44` olarak düzeltildi ve aynı test iki kez tekrarlandı. Bu hata [connection modülündeki sabit testi](../src/connection.rs) ile korunuyor.

## Java referans istemcisi

Aynı vanilla sunucuya Java Edition 26.2 referans istemcisi bağlandı. Logda `VanillaReference joined the game`, OpenGL cihazı (`AMD Radeon RX 9060 XT`) ve chunk section UBO büyütmeleri görüldü. Böylece vanilla sunucunun grafik Java istemcisini de oyun oturumuna taşıdığı doğrulandı.

Bu koşu Rust parity karşılaştırması değildir. Sahte offline kimlik bilgileri nedeniyle profile-key/Realms için `401` uyarıları ve yerel narrator kütüphanesi bulunmadığı için bir narrator uyarısı görüldü; bunlar bağlantı ve chunk teslimi sonucunu etkilemedi.

## Açık kapsam

Bu sonuç P06/P07'nin offline login, compression, phase transition, keep-alive/ping ve registry/tag teslimi dilimlerini; P08'in ise ilk chunk teslimi ön koşulunu kapsar. Encryption, parçalı ağ, disconnect/reconnect, chunk section/palette/light decoder'ları, entity state, client world cache, input/pencere, hareket, eylem ve Rust tarafında oynanabilir render akışı açık kalır. Başarılı probe tamamlanmış istemci veya Java ile tam oyun eşitliği anlamına gelmez.

Orijinal Java server/client dosyaları, assetler, dünya ve ham loglar yerel referans veya geçici dizinlerde kaldı; depoya eklenmedi.
