# P13 — Survival inventory partial changed-slot prediction sözleşmesi

Bu dilim, inventory pickup packet'inde değişen slotun `HashedStack`
haritasından tamamen çıkarıldığı durumda vanilla server'ın slotu düzeltmesini,
doğru carried prediction'ı ise korumasını sınar.

## Java referansı

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
packet içindeki changed-slot map girdilerini tek tek remote slot beklentisi
olarak kaydeder. Haritada bir slot yoksa o slot için yeni bir remote beklenti
alınmaz; önceki beklenti korunur.

`AbstractContainerMenu.broadcastChanges`, pickup sonrası gerçek slot durumunu
korunan remote beklentiyle karşılaştırır. Kaynak slot gerçek durumda empty
olduğu için slot `36` için state-id taşıyan `container_set_slot` düzeltmesi
gönderilir. Carried hash'i gerçek stone stack ile eşleştiği için cursor
düzeltmesi gönderilmez. State-id doğru olduğundan full resync yolu çalışmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: boş map; slot `36` için hash girdisi yok
- Carried prediction: item ID `1`, count `8`, boş added/removed component
  map/set içeren non-empty hash

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1` ve count `8`
  gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot map'i slot `36` içermediği için server'ın önceki remote slot
  beklentisi korunur ve gerçek empty slot için state `3` correction gelir.
- Carried prediction doğru olduğu için `set_cursor_item` correction gelmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim değişen slot hash girdisinin eksik olmasını ve doğru carried hash'iyle
bağımsız slot/cursor düzeltmesini kapsar. Birden fazla eksik slot, yanlış veya
kısmi carried component prediction, diğer click input'ları ve özel container'lar
sonraki P13 dilimleridir.
