# P12 — Survival block-break başlatma ve iptal sözleşmesi

Bu dilim, survival oyuncunun aynı hedef için `START_DESTROY_BLOCK` gönderip hemen ardından `ABORT_DESTROY_BLOCK` ile kazmayı iptal etmesini gerçek vanilla server üzerinde sınar. Amaç hedefin kırılmadığını, sequence ACK sırasını ve server'ın actor dışına gönderdiği break-progress davranışını ayırmaktır.

## Java referansı

`ServerboundPlayerActionPacket.Action` sırasına göre `START_DESTROY_BLOCK` ordinal `0`, `ABORT_DESTROY_BLOCK` ordinal `1` ve `STOP_DESTROY_BLOCK` ordinal `2`'dir. Serverbound packet ID'si `41`'dir. `ServerGamePacketListenerImpl.handlePlayerAction` start/abort/stop action'larını `ServerPlayerGameMode.handleBlockBreakAction`'a aktarır ve sequence'i ACK kuyruğuna alır.

Survival start, hedefin destroy progress değerini hesaplayıp `ServerLevel.destroyBlockProgress` ile progress başlangıcını yayınlar. Abort aynı hedefte progress'i `-1` ile temizler ve dünya block state'ini değiştirmez. `ServerLevel.destroyBlockProgress` kendi action'ını yapan oyuncuyu hedef listesinden çıkardığı için actor bağlantısı `ClientboundBlockDestructionPacket` almaz; yerel client'ın kendi break animasyonu ayrı prediction kapsamıdır.

## Probe girdisi ve akışı

- Game mode: `SURVIVAL`
- Hedef: oyuncunun spawn olduğu konumun bir blok altındaki Java floor hedefi
- Direction: `UP`, ID `1`
- Start sequence: `6`
- Abort sequence: `7`
- Ana el: boş

İlk `player_loaded` sonrasında Rust start ve abort packet'lerini aynı oturumda yollar. Vanilla server ACK kuyruğunda en yüksek sequence'i `[7]` olarak döndürür; hedef için block update üretmez ve hedef taş olarak kalır.

## Sınır

Bu dilim survival kazma süresini, tick aralıklarını, `STOP_DESTROY_BLOCK` ile tamamlamayı, delayed destroy yolunu, ray hit üretimini, client-side break prediction'ı ve başka oyuncuya gönderilen destruction progress akışını kapsamaz.
