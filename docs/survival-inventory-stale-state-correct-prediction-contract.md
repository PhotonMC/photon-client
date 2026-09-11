# P13 — Survival inventory stale state correct prediction sözleşmesi

Bu dilim, inventory pickup packet'inde post-click slot ve carried hash'leri
doğru olsa bile state-id eski olduğunda vanilla server'ın full resync seçmesini
sınar.

## Java referansı

`ServerGamePacketListenerImpl.handleContainerClick`, packet state-id'sini menu
state-id'siyle karşılaştırır. Değerler eşit değilse `fullResyncNeeded` true olur.
Server click'i yine `clicked` ile uygular, packet'teki remote hash'leri kaydeder
ve sonunda `broadcastFullState` çağırır. Bu dal hash eşleşmelerine bakarak
artımlı slot/cursor yayınını bastırmaz.

`AbstractContainerMenu.broadcastFullState`, pickup sonrası bütün slotları ve
carried item'ı yeni state-id ile birlikte yollar. Bu nedenle doğru post-click
prediction gönderilmiş olsa bile stale state-id tam content yayınının yerini
değiştirmez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Server menu state-id: `2`
- Gönderilen stale state-id: `0`
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `8`, boş added/removed component
  map/set içeren doğru non-empty hash

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1` ve count `8`
  gözlemi üretir.
- Server click'i uygular, fakat state-id mismatch nedeniyle full resync dalını
  seçer.
- Pickup sonrası full content state `3` ile slot `36` empty, carried item `1`
  count `8` olur.
- Doğru hash'lere rağmen artımlı slot veya cursor düzeltmesi gönderilmez;
  full content gözlemi oluşur.

## Sınır

Bu dilim stale state-id ile doğru slot/carried prediction'ın birlikte
gönderilmesini ve full resync önceliğini kapsar. Ahead state-id'nin doğru hash
varyantı, stale state ile component'li item, diğer click input'ları ve özel
container'lar sonraki P13 dilimleridir.
