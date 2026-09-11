# P06.a — Status socket ve ping yaşam döngüsü sözleşmesi

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
