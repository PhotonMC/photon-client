# P06–P08 — İlk gerçek oyun oturumu sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Rust tarafının sabit bir Java Edition 26.2 protokol sunucusuna bağlanıp login, configuration ve play'e geçişte ilk dünya paketlerini almasını ölçer. Bu bir oynanabilir istemci veya Java eşitliği kanıtı değildir; sonraki dünya, hareket ve görüntü dilimleri için otomatik bir bağlantı ön koşuludur.

## Sabit referans

- Java Edition 26.2
- Protokol 776
- Java 25
- Offline-authenticated, yerel vanilla 26.2 dedicated server
- Güncel baseline TCP adresi test koşusunda `127.0.0.1:25566`
- Probe kullanıcı adları koşuya göre sabitlenir; güncel vanilla kaydı `VanillaProbe2` ve `VanillaProbe3` adlarını kullanır
- Pumpkin koşusu aynı kaydın [Pumpkin server-conformance alt bölümünde](#pumpkin-server-conformance-karşılaştırması) tutulur

Java kaynak incelemesinde `ServerLoginPacketListenerImpl`, `ClientHandshakePacketListenerImpl`, `ServerConfigurationPacketListenerImpl`, `CompressionEncoder`, `CompressionDecoder`, `ClientPacketListener` içindeki chunk batch akışı ve `GameProtocols` paket sıraları kullanıldı. Kaynak kodu bu depoya kopyalanmadı.

## Rust davranış sınırı

`crates/voxel-net/src/connection/session.rs` ve `crates/voxel-net/src/connection/protocol.rs` aşağıdaki akışı yürütür:

1. Protocol 776 login intention ve offline login hello gönderir.
2. Login compression paketini okuyup zlib çerçevesini Java'nın eşik kuralıyla çözer; eşik altındaki paketleri `uncompressed length = 0` ile, eşik ve üstündekileri sıkıştırılmış olarak yazar.
3. Login finished sonrasında login acknowledged gönderir ve configuration'a geçer.
4. Brand, client information ve known-pack seçimini gönderir; registry data, enabled features ve tags paketlerini çözüp sayar; keep-alive, ping ve code-of-conduct yanıtlarını verir.
5. Configuration finished sonrasında play login, chunk ve chunk batch finished paketlerini gözlemler. Batch tamamlanınca Java başlangıç hesaplayıcısındaki `3.5f` hedef değerini gönderir.
6. İlk chunk alındıktan sonra kısa bir sessizlik penceresinde gelen paketleri tüketir. Bu, probe'un sunucu paketlerini yazarken bağlantıyı erken bırakmasını önler.
7. İlk clientbound oyuncu konumu düzeltmesinde teleport ID ve relative flag'lerini çözer; Java akışındaki teleport kabulü ile mutlak `move_player_pos_rot` cevabını gönderir ve ilk chunk batch'i tamamlanınca `player_loaded` bildirir.

Sözleşmenin temel gözlemleri şunlardır:

- compression threshold
- login ve configuration phase geçişleri
- enabled feature, known pack, registry ve tag sayıları
- play login ve ilk chunk teslimi
- chunk batch geri bildirim sayısı
- ilk oyuncu konumu düzeltmesi ve `player_loaded` gönderimi
- alınan paketlerin phase, id ve gövde uzunluğu

## Kapsam dışı

Bu dilim chunk gövdesini dünya state'ine dönüştürmez; packet'in x/z, heightmap type/word alanları, section byte uzunluğu, section palette storage başlıkları, block entity listesi ve ışık update byte uzunlukları [ayrı P08 sözleşmesinde](chunk-packet-conformance.md) yapısal olarak tüketilir. Palette ID'lerinin block state/biome kayıtlarına eşlenmesi, section world state'i, ışık değeri, registry, block entity world state ve entity decoder'ları hâlâ P08 alt dilimleridir. P05 pencere/input, oyuncu tick'i, hareket, eylem, envanter, kamera, renderer, ses, GUI ve yeniden bağlanma da bu sözleşmede yoktur. Vanilla baseline'ı istemci davranışının bağımsız referansıdır; Pumpkin'a özgü eksikleri telafi eden istemci dalları bu sözleşmeye eklenmez.


## Vanilla 26.2 server-conformance doğrulaması

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

İlk vanilla probe koşusu (`VanillaProbe1`) eski kodla `UnexpectedEof` ile kapandı; vanilla server oyuncuyu önce kabul etti, sonra geçersiz keep-alive yanıtı nedeniyle timeout kaydetti. Java 26.2 packet dispatch sırası yeniden kontrol edildiğinde `43` numarasının `initialize_border`, `44` numarasının gerçek play `keep_alive` olduğu görüldü. Rust dispatch sabiti `44` olarak düzeltildi ve aynı test iki kez tekrarlandı. Bu hata [protocol modülündeki sabit testi](../src/connection/protocol.rs:2121) ile korunuyor.

## Java referans istemcisi

Aynı vanilla sunucuya Java Edition 26.2 referans istemcisi bağlandı. Logda `VanillaReference joined the game`, OpenGL cihazı (`AMD Radeon RX 9060 XT`) ve chunk section UBO büyütmeleri görüldü. Böylece vanilla sunucunun grafik Java istemcisini de oyun oturumuna taşıdığı doğrulandı.

Bu koşu Rust parity karşılaştırması değildir. Sahte offline kimlik bilgileri nedeniyle profile-key/Realms için `401` uyarıları ve yerel narrator kütüphanesi bulunmadığı için bir narrator uyarısı görüldü; bunlar bağlantı ve chunk teslimi sonucunu etkilemedi.

## Açık kapsam

Bu sonuç P06/P07'nin offline login, compression, phase transition, keep-alive/ping ve registry/tag teslimi dilimlerini; P08'in ilk chunk teslimi ön koşulunu kapsar. Chunk packet gövdesinin yapısal doğrulaması [ayrı P08 kaydında](chunk-packet-conformance.md) tutulur. Encryption, parçalı ağ, disconnect/reconnect, chunk section/palette/light decoder'ları, entity state, client world cache, input/pencere, hareket, eylem ve Rust tarafında oynanabilir render akışı açık kalır. Başarılı probe tamamlanmış istemci veya Java ile tam oyun eşitliği anlamına gelmez.

Orijinal Java server/client dosyaları, assetler, dünya ve ham loglar yerel referans veya geçici dizinlerde kaldı; depoya eklenmedi.


## Pumpkin server-conformance karşılaştırması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** yerel, offline Pumpkin 26.2 adayı
**Sonuç:** bağlantı ve ilk dünya teslimi iki Rust probe koşusunda geçti. Bu kayıt vanilla baseline'dan ayrı tutulur; güncel resmi server sonucu [Vanilla 26.2 server-conformance alt bölümündedir](#vanilla-26.2-server-conformance-doğrulaması).

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
