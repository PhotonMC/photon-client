# Sunucu diyalogları — [paketler: server-conformance (sahte sunucu testi); ekran (T105b): client-parity]

Kapsam: oyun evresinde `ClientboundShowDialog` (`140`), `ClientboundClearDialog` (`139`) ve
`ServerboundCustomClickAction` (`68`). Diyalog `Holder` olarak gelir: satır içi etiket ya da `minecraft:dialog`
kayıt defterinin sıra numarası. Bildirim ve onay diyaloğu, ortak veri (başlık, Esc, duraklatma, işlem sonrası,
gövde) ve düğme eylemleri okunur; `ClickEvent` artık `show_dialog` ve yüklü `custom` taşır.

Kanıt: `photon-game` ve `photon-net` birim testleri (ayrıştırma varsayılanları, `pause` + `after_action: none`
reddi, tutucu ve `CustomClickAction` gövdesi). Paket kısmı bir `server-conformance` kaydıdır.

## Ekran (T105b, `client-parity`)

Bildirim ve onay diyaloğunun ekranı (`SimpleDialogScreen`) ve bekleme ekranı (`WaitingForResponseScreen`) Java'nın
gerçek sınıflarıyla `dialog_screen` GUI oracle'ında karşılaştırılır ([tablo](../gui-oracle.md)): başlık ve uyarı
düğmesi, kaydırılan gövde (düz mesaj, eşya ve açıklaması), alt düğmeler, Tab/Enter/Space/Esc, tıklama, tekerlek ve
çubuk sürükleme, Tab ile görünüme kaydırma, uyarı düğmesinin ipucu. Düğmenin ve Esc'in verdiği karar
(`DialogScreen.runAction`: sonraki ekran + `run_command`/`custom`/`show_dialog`) `photon_game::dialog::run_action`
ile aynı kayıtla karşılaştırılır. Oyuna bağlama (`controls/dialogs.rs`) bunun üstündeki ince kısımdır:
`ShowDialog`/`ClearDialog`, uyarı ekranı (`DialogScreen.WarningScreen`), bekleme ekranı ve sohbetteki
`show_dialog`/`custom` tıklamaları.

## Sapmalar (açık)

- Düğme eylemi `run_command` doğrudan komut paketi olarak gider; Java'nın `sendUnattendedCommand` denetimi
  (ayrıştırma hatası, imza gerekli, izin yok için onay penceresi) yoktur (sohbet tıklamasında da yok).
- Gövdedeki metnin tıklanabilir stili (`runActionOnParent`) ve oklarla odak gezintisi yok; Tab sırası uyarı düğmesi,
  gövde parçaları, alt düğmeler. Eşya gövdesinin ipucu (`show_tooltip`) çizilmez; eşya simgesi kutu dışına taşan
  durumda çizilmez (Java kesiyor). Java oracle'ı eşya çizimini kaydetmediği için simge parity kapsamı dışıdır.
- `WarningScreen` Java'da bir `ConfirmScreen`dir; burada aynı ekran, tek oyunculu "Return to Menu" etiketi yok.
- Ekran yığını yerine her ekranın kendi alanı vardır: diyalog açıkken alttaki ekran (duraklatma, sohbet) olduğu
  gibi kalır, diyalog kapanınca yeniden görünür.

- Yapılandırma evresindeki diyalog (`17`/`18`) yok sayılır; ekranı yok.
- Bilinen paketin kayıt girdisi veri taşımaz (`None`); ona gönderme yapan `ShowDialog` sessizce düşer.
- Gövde türleri yalnız düz mesaj ve eşya; girdiler okunmaz (T106). `dynamic/*` eylemleri `Dynamic` olarak
  işaretlenir, çalıştırılmaz.
