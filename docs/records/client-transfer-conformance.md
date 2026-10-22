# Transfer (`ClientboundTransferPacket`) — [server-conformance değil, sahte sunucu testi]

Kapsam: yapılandırma (`11`) ve oyun (`129`) evresinde `Transfer`: bağlantı bırakılır, verilen sunucu adı
çözülür, `TRANSFER` niyetiyle (el sıkışmada yazılan sunucu adıyla) yeni bir bağlantı açılır; çerezler
(T100) yeni bağlantıya taşınır. Art arda en çok beş aktarım izlenir. Yeni oturum yeniden yapılandırılmış
gibi başlar (`level_reset`, `reconfigured`).

Kanıt: `cookie` birim testi (paket çözümü) ve sahte sunucu testi (oyun evresinde aktarım paketi; hedef
sunucu el sıkışmasında `TRANSFER` niyetini, adı ve portu görür). Gerçek bir aktarımla (Velocity, vanilla
`/transfer`) denenmedi: aktarılacak ikinci sunucu ve girişin tamamı sahte sunucuda yok. Java oracle'ı yok.

## Sapmalar (açık)

- Bağlantı ayrı bir iş parçacığında kurulur ve `ConnectScreen` ("Transferring to new server...")
  görünür; İptal başlık ekranına döner, hata `connect.failed.transfer` başlıklı
  `DisconnectedScreen`'e gider. Gerçek vanilla `/transfer` ile (kendi kendine ve ulaşılamayan adrese)
  denendi. Java eski seviyeyi hemen siler; burada başarısız aktarımda eski dünya ekranın arkasında kalır.
- Ad çözümü yalnız sistemin adres çözümlemesidir (`SRV` kaydı aranmaz).
- Çerezlerin saklandığı yer bellektir; diske yazılmaz (Java'da da yok).
- Sohbet durumu ve görülen oyuncular taşınmaz (`TransferState` bunları taşır).
