# Yeniden yapılandırma (`StartConfiguration`) — [server-conformance değil, sahte sunucu testi]

Kapsam: oyun evresinde `ClientboundStartConfigurationPacket` (terminal paket): okuyucu iş parçacığı bu
pakette durur, istemci `ServerboundConfigurationAcknowledged` gönderir, yapılandırma aşaması
(`configure_connection`: kayıt defterleri, etiketler, özellikler, bilinen paketler, bitiş) yeniden
çalışır, sonra yeni bir oyun evresi başlar: dünya boştur, `level_reset` ve `reconfigured` bayrakları kalkar,
açık ekran ve sohbet kapanır. Yapılandırma döngüsü açılışla ortaktır (`probe/open.rs`).

Kanıt: `connection::play::reconfigure` testi, loopback üzerinde sahte sunucuyla: terminal paket, onay
(`16`), bitiş (`3`) ve bitiş yanıtını doğrular. Gerçek bir sunucuyla (Velocity gibi) denenmedi; Java
oracle'ı yok. `client-parity` değildir.

## Sapmalar (açık)

- Yapılandırma ana iş parçacığında bloklayarak çalışır (en çok 30 sn); Java eşzamansızdır ve
  `ServerReconfigScreen` gösterir. Bu sürede kare durur.
- Sohbet durumu (`storeState`) ve görülen oyuncular korunmaz; sohbet kayıtları yeni oturumla boşalır.
- Yapılandırmadaki özel yük, çerez, kaynak paketi ve diyalog paketleri yok sayılır (T100, T103, T105).
- Yeniden yapılandırmada marka ve istemci bilgisi yeniden gönderilmez (Java'da da yok).
