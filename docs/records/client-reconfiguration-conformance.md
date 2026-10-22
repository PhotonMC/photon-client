# Yeniden yapılandırma (`StartConfiguration`) — [server-conformance değil, sahte sunucu testi]

Kapsam: oyun evresinde `ClientboundStartConfigurationPacket` (terminal paket): okuyucu iş parçacığı bu
pakette durur, istemci `ServerboundConfigurationAcknowledged` gönderir, yapılandırma aşaması
(`configure_connection`: kayıt defterleri, etiketler, özellikler, bilinen paketler, bitiş) yeniden
çalışır, sonra yeni bir oyun evresi başlar: dünya boştur, `level_reset` ve `reconfigured` bayrakları kalkar,
açık ekran ve sohbet kapanır. Yapılandırma döngüsü açılışla ortaktır (`probe/open.rs`).

Kanıt: `connection::play::reconfigure` testi, loopback üzerinde sahte sunucuyla: terminal paket, onay
(`16`), bitiş (`3`) ve bitiş yanıtını doğrular. İkinci girişin başka bir boyut türü kimliği ve yüksekliğiyle gelmesi de sahte sunucu testiyle
sınanır. Vanilla `/transfer` ile gerçek sunucuda denendi (dünya yeniden kuruldu, çökme yok); Velocity ile
denenmedi; Java oracle'ı yok. `client-parity` değildir.

## Sapmalar (açık)

- Yapılandırma ayrı bir iş parçacığında çalışır (en çok 30 sn okuma beklemesi); pencere çizilmeye ve
  girdi almaya devam eder ve `ServerReconfigScreen` görünür (yerleşimi Java oracle'ıyla sınanır,
  `gui/connect-screen`). Bu sürede oyun tikleri durur. Bağlantıyı kesen sunucu ya da hata
  `DisconnectedScreen`'e gider ("Aborted" düğmeden, 600 tik sonra etkin). Yapılandırma evresindeki
  `Transfer` izlenir. Sunucunun yapılandırmadaki kopma nedeni (`Disconnect` paketinin metni) okunmaz;
  ekran "Disconnected" (`disconnect.endOfStream`) der.
- Sohbet durumu (`storeState`) ve görülen oyuncular korunmaz; sohbet kayıtları yeni oturumla boşalır.
- Yapılandırmadaki özel yük, çerez, kaynak paketi ve diyalog paketleri yok sayılır (T100, T103, T105).
- Yeniden yapılandırmada marka ve istemci bilgisi yeniden gönderilmez (Java'da da yok).
