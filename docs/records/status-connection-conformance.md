# P06.a — Status socket ve ping yaşam döngüsü sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim Rust tarafının Java Edition 26.2 status protokolüne bağlanmasını ölçer. Login veya oyun dünyası açılmaz; bağlantı handshake içindeki status intention ile seçilir, status yanıtı okunur, ping echo'su doğrulanır ve vanilla sunucunun bu isteği tamamladıktan sonra bağlantıyı kapatması gözlenir.

## Sabit referans

- Java Edition 26.2
- Protokol 776
- Java 25
- Offline-authenticated, yerel vanilla 26.2 dedicated server
- Güncel baseline TCP adresi test koşusunda `127.0.0.1:25566`
- Ping değeri `0x0102030405060708`

Java kaynak incelemesinde `ClientIntentionPacket`, `StatusProtocols`, `ClientboundStatusResponsePacket`, `ServerStatusPacketListenerImpl`, `ServerboundPingRequestPacket` ve `ClientboundPongResponsePacket` yolları kullanıldı. Kaynak kodu bu depoya kopyalanmadı.

## Rust davranış sınırı

`query_status_and_ping` aşağıdaki akışı yürütür:

1. Protocol 776 ile status intention handshake paketi gönderir.
2. Payload'sız status request gönderir ve clientbound status response packet'ini okur.
3. Status response içindeki Java uzunluklu UTF-8 JSON metnini sınırıyla birlikte çözer ve ham metni gözlem için saklar.
4. Sabit ping değerini gönderir, pong packet'ini okur ve değerin aynen geri geldiğini doğrular.
5. Status oturumundaki iki packet'i phase, id ve gövde uzunluğuyla kaydeder.

Bu dilim JSON içeriğinin bütün `ServerStatus` alanlarını Rust veri modeline dönüştürmez. JSON şema codec'i, favicon, server list UI'si, login ve play state geçişleri ayrı kapsamdır.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** resmi vanilla 26.2 dedicated server, offline yerel koşu
**Sonuç:** Rust status probe iki ardışık koşuda status response ve ping echo akışını tamamladı.

## Çalıştırma

```text
cargo run --offline --example status_probe -- 127.0.0.1:25566
```

Komut iki bağımsız TCP oturumunda çalıştırıldı. İki sonuç da aynıydı:

```text
status_connection=ok
status_response_length=115
status_response={"description":"Vanilla 26.2 parity test","players":{"max":20,"online":0},"version":{"name":"26.2","protocol":776}}
pong_time=72623859790382856
received_packet_count=2
```

`72623859790382856`, `0x0102030405060708` değerinin signed 64-bit gösterimidir. Status response içindeki sürüm ve protocol alanları sabit hedefle eşleşti; server ping yanıtından sonra bağlantıyı kapattı.

## Açık kapsam

Bu sonuç status intention, status request/response framing, UTF-8 response payload'ı, ping/pong echo'su ve status oturumunun kapanışını kapsar. `ServerStatus` JSON şemasının Rust'a ayrıştırılması, favicon, server list ekranı, login/play bağlantısı ve online authentication açık kalır.

Bu kayıt vanilla baseline içindir; Pumpkin koşusu [ayrı oyun oturumu kaydında](game-connection-conformance.md) tutulur. Orijinal Java dosyaları ve ham server logları yerel referans/geçici dizinlerde kaldı.
