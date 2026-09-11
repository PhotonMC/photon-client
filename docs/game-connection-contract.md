# P06–P08 — İlk gerçek oyun oturumu sözleşmesi

Bu dilim, Rust tarafının sabit bir Pumpkin sunucusuna bağlanıp login, configuration ve play'e geçişte ilk dünya paketlerini almasını ölçer. Bu bir oynanabilir istemci veya Java eşitliği kanıtı değildir; sonraki dünya, hareket ve görüntü dilimleri için otomatik bir bağlantı ön koşuludur.

## Sabit referans

- Java Edition 26.2
- Protokol 776
- Java 25
- Offline-authenticated, yerel Pumpkin 26.2 sunucusu
- TCP adresi test koşusunda `127.0.0.1:25565`
- Kullanıcı adları test koşusunda `RustProbe3` ve `RustProbe4`

Java kaynak incelemesinde `ServerLoginPacketListenerImpl`, `ClientHandshakePacketListenerImpl`, `ServerConfigurationPacketListenerImpl`, `CompressionEncoder`, `CompressionDecoder`, `ClientPacketListener` içindeki chunk batch akışı ve `GameProtocols` paket sıraları kullanıldı. Kaynak kodu bu depoya kopyalanmadı.

## Rust davranış sınırı

`src/connection.rs` aşağıdaki akışı yürütür:

1. Protocol 776 login intention ve offline login hello gönderir.
2. Login compression paketini okuyup zlib çerçevesini Java'nın eşik kuralıyla çözer; eşik altındaki paketleri `uncompressed length = 0` ile, eşik ve üstündekileri sıkıştırılmış olarak yazar.
3. Login finished sonrasında login acknowledged gönderir ve configuration'a geçer.
4. Brand, client information ve known-pack seçimini gönderir; registry data, enabled features ve tags paketlerini çözüp sayar; keep-alive, ping ve code-of-conduct yanıtlarını verir.
5. Configuration finished sonrasında play login, chunk ve chunk batch finished paketlerini gözlemler. Batch tamamlanınca Java başlangıç hesaplayıcısındaki `3.5f` hedef değerini gönderir.
6. İlk chunk alındıktan sonra kısa bir sessizlik penceresinde gelen paketleri tüketir. Bu, probe'un sunucu paketlerini yazarken bağlantıyı erken bırakmasını önler.

Sözleşmenin temel gözlemleri şunlardır:

- compression threshold
- login ve configuration phase geçişleri
- enabled feature, known pack, registry ve tag sayıları
- play login ve ilk chunk teslimi
- chunk batch geri bildirim sayısı
- alınan paketlerin phase, id ve gövde uzunluğu

## Kapsam dışı

Bu dilim chunk gövdesini dünya state'ine dönüştürmez; palette, section, ışık, biome, block entity ve entity decoder'ları P08 alt dilimleridir. P05 pencere/input, oyuncu tick'i, hareket, eylem, envanter, kamera, renderer, ses, GUI ve yeniden bağlanma da bu sözleşmede yoktur. Sunucu davranışı istemciye uyarlamak için değiştirilmez; Pumpkin yalnızca sabit entegrasyon karşılığıdır.
