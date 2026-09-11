# P12 — Menzil dışı blok kırma reddi sözleşmesi

Bu dilim, oyuncu yüklendikten sonra gerçek vanilla server'a menzil dışındaki bir blok için `START_DESTROY_BLOCK` gönderir. Amaç, server'ın hedef menzili kontrol edip dünya güncellemesi üretmeden sequence ACK vermesini gözlemlemektir.

## Java referansı

`ServerGamePacketListenerImpl.handlePlayerAction` action packet'ini loaded oyuncuya yönlendirir ve `ackBlockChangesUpTo` çağırır. `ServerPlayerGameMode.handleBlockBreakAction` önce `isWithinBlockInteractionRange` kontrolünü yapar; hedef uzaksa block-break state'i değiştirmez ve bu action için clientbound block update üretmez.

## Probe girdisi ve akışı

- Action: `START_DESTROY_BLOCK`, enum ordinal `0`
- Hedef: `(1_000_000, 64, 1_000_000)`; spawn oyuncusundan kesinlikle menzil dışı
- Direction: `UP`, ID `1`
- Sequence: `3`

İlk `player_loaded` bildirimi sonrasında Rust tek packet gönderir. Beklenen gözlem action sayısı `1`, ACK listesi `[3]` ve block update listesinin boş olmasıdır. Bu, ACK'in action kabul kuyruğuna alındığını; hedefin kırıldığı veya client world state'in değiştiğini göstermez.

## Sınır

Bu negatif probe ray/hit hesabını, survival destroy progress'ini, spawn korumasını, placement'ı, item use'u ve prediction/rollback'u uygulamaz.
