# P12 — Survival block-break başlatma ve tamamlama sözleşmesi

Bu dilim, survival oyuncunun bir dirt bloğu için `START_DESTROY_BLOCK` gönderip kontrollü bir tick aralığından sonra aynı hedefe `STOP_DESTROY_BLOCK` göndermesini gerçek vanilla server üzerinde sınar. Amaç stop eşiğini geçen kırma sonucunu, sequence ACK'lerini ve otoriter air update'ini gözlemlemektir.

## Java referansı

`ServerboundPlayerActionPacket.Action` içinde `START_DESTROY_BLOCK` ordinal `0`, `STOP_DESTROY_BLOCK` ordinal `2`'dir. Packet ID `41` ve Direction `UP` ID `1` olarak kodlanır. `ServerPlayerGameMode.handleBlockBreakAction`, aynı hedefte geçen tick sayısını ve block destroy progress'ini çarpar; sonuç `0.7F` eşiğini geçerse destruction progress'i `-1` ile temizler ve `destroyAndAck` çağırır. Kırılan block için `ClientboundBlockUpdatePacket` state ID `0` döner.

## Probe girdisi ve akışı

- Game mode: `SURVIVAL`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Hedef materyali: kontrollü y=`100` dirt platformu
- Direction: `UP`, ID `1`
- Start sequence: `8`
- Stop sequence: `9`
- Start ile stop arasında bekleme: `650 ms`

İlk `player_loaded` sonrasında Rust start packet'ini yollar, 650 ms bekler ve stop packet'ini yollar. Bu süre dirt'in boş elle stop eşiğini geçmesi için seçilmiştir. Vanilla server ACK kuyruğunda önce `[8]`, sonra `[9]` gözlenir; hedef state ID `0` block update ile air olur.

## Sınır

Bu dilim tüm hardness/tool kombinasyonlarını, her tick'teki progress değerlerini, delayed destroy yolunu, client-side break prediction'ı, ray hit üretimini ve başka oyuncuya gönderilen destruction progress paketlerini kapsamaz. Sabit bekleme, genel istemci tick eşitliği yerine bu tek kontrollü server senaryosunun girdisidir.
