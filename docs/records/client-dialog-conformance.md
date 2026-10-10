# Sunucu diyalogları: veri ve paketler — [paketler: sahte sunucu testi; ekran: T105b]

Kapsam: oyun evresinde `ClientboundShowDialog` (`140`), `ClientboundClearDialog` (`139`) ve
`ServerboundCustomClickAction` (`68`). Diyalog `Holder` olarak gelir: satır içi etiket ya da `minecraft:dialog`
kayıt defterinin sıra numarası. Bildirim ve onay diyaloğu, ortak veri (başlık, Esc, duraklatma, işlem sonrası,
gövde) ve düğme eylemleri okunur; `ClickEvent` artık `show_dialog` ve yüklü `custom` taşır.

Kanıt: `photon-game` ve `photon-net` birim testleri (ayrıştırma varsayılanları, `pause` + `after_action: none`
reddi, tutucu ve `CustomClickAction` gövdesi). Bu bir `server-conformance` kaydıdır; ekran eşitliği
(`client-parity`) T105b'dedir.

## Sapmalar (açık)

- Yapılandırma evresindeki diyalog (`17`/`18`) yok sayılır; ekranı yok.
- Bilinen paketin kayıt girdisi veri taşımaz (`None`); ona gönderme yapan `ShowDialog` sessizce düşer.
- Gövde türleri yalnız düz mesaj ve eşya; girdiler okunmaz (T106). `dynamic/*` eylemleri `Dynamic` olarak
  işaretlenir, çalıştırılmaz.
