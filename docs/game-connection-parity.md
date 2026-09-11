# P06–P08 — Pumpkin uyumluluk oturumu doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** yerel, offline Pumpkin 26.2 adayı
**Sonuç:** bağlantı ve ilk dünya teslimi iki Rust probe koşusunda geçti. Bu kayıt vanilla baseline'dan ayrı tutulur; güncel resmi server sonucu [vanilla doğrulamasındadır](vanilla-connection-parity.md).

## Çalıştırma

Pumpkin izole bir geçici dünya/config kopyasında çalıştırıldı. Rust probe şu komutla iki ayrı koşuda, `RustProbe3` ve `RustProbe4` kullanıcı adlarıyla çalıştırıldı:

```text
cargo run --offline --example connection_probe -- 127.0.0.1:25565 RustProbe3
```

Komut gerçek TCP oturumu açar; elle oyun oynama veya ekran değerlendirmesi istemez. Probe, ilk world chunk akışından sonra kısa bir drain penceresi kullandığı için Pumpkin logunda erken `connection reset` uyarısı oluşmadı. Her iki oturumda da Pumpkin `joined` ve `left` kaydı üretti.

## Rust probe gözlemleri

| Gözlem | RustProbe3 | RustProbe4 |
| --- | ---: | ---: |
| compression threshold | 256 | 256 |
| login finished | `true` | `true` |
| configuration finished | `true` | `true` |
| enabled feature | 1 | 1 |
| known pack | 1 | 1 |
| registry packet | 29 | 29 |
| registry entry | 398 | 398 |
| update tags packet | 1 | 1 |
| play login | `true` | `true` |
| chunk packet | 22 | 22 |
| chunk batch finished | 6 | 6 |

Alınan toplam paket sayısı iki koşuda sırasıyla 973 ve 1010 oldu. Bu sayı, sunucunun chunk/entity paketlerini aynı sessizlik penceresinde ne kadar hızlı gönderdiğine bağlıdır; yukarıdaki phase ve içerik sayaçlarının yerine geçmez.

## Java referans gözlemi

Aynı Pumpkin oturum ortamına yerel Java 26.2 istemcisi bağlandı. Java logunda `RustPlayer joined the game` görüldü; LWJGL/OpenGL başlatıldı ve chunk/UBO yüklemesi gerçekleşti. Bu koşu Rust parity karşılaştırması değildir; yalnızca referans istemcinin aynı protokol/sunucu ortamında dünya oturumuna ulaşabildiğini gösterir.

Java koşusunda sahte offline kimlik bilgileri nedeniyle profile-key/Realms kimlik doğrulama uyarıları, sistemde narrator kütüphanesi bulunmadığı için narrator uyarısı ve kopyalanan test dünyasındaki duplicate entity UUID uyarıları görüldü. Bunlar bu bağlantı probe'unun başarı koşulu değildir ve mekanik eşitliği olarak raporlanmaz.

## Açık kapsam

Bu sonuç P06/P07'nin ilk gerçek bağlantı, phase transition, compression, keep-alive/ping ve registry/tag teslimi dilimlerini; P08'in ise yalnızca ilk chunk teslimi ön koşulunu kapsar. Encryption, disconnect/reconnect, parçalı ağ harness'i, chunk section/palette/light decoder'ları, entity state, client world cache, input/pencere ve oynanabilir render akışı açık kalır. Başarılı bağlantı, tamamlanmış Rust istemcisi veya Java ile tam davranış eşitliği anlamına gelmez.

Orijinal Java kaynakları, assetler, dünya kopyası ve ham client/server logları `.data/` veya geçici test dizinlerinde tutuldu; depoya eklenmedi.
