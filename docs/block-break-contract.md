# P12 — Yaratıcı modda blok kırma ve block update sözleşmesi

Bu dilim, ilk dünya yüklemesinden sonra oyuncunun spawn olduğu konumun altındaki bloğa vanilla block-break akışını gönderir. Amaç, gerçek hedef menzili, creative instant-break yolunu, sequence ACK'ini ve server'ın yeni block state'i clientbound `block_update` ile bildirmesini aynı oturumda gözlemlemektir.

## Java referansı

`ServerboundPlayerActionPacket` alan sırası ve action enum ordinal'ları; `ServerGamePacketListenerImpl.handlePlayerAction` içindeki loaded/ACK akışı; `ServerPlayerGameMode.handleBlockBreakAction` içindeki menzil, creative `destroyAndAck` ve block update davranışı incelendi. Hedef blok Java `BlockPos.containing` floor semantiğiyle oyuncu konumunun bir blok altından türetilir. `ClientboundBlockUpdatePacket` packed `BlockPos` ve block-state registry ID'si taşır.

## Probe girdisi ve akışı

- Action: `START_DESTROY_BLOCK`, enum ordinal `0`
- Hedef: ilk player-position paketindeki `(x, y - 1, z)` konumunun Java floor karşılığı
- Direction: `UP`, ID `1`
- Sequence: `2`
- Server ayarı: creative, spawn protection `0`

İlk `player_loaded` bildirimi gönderildikten sonra Rust tek bir start-destroy packet'i yollar. Vanilla server creative modda hedefi anında air state'ine geçirir, sequence'i ACK kuyruğuna alır ve hedef için state ID `0` olan `block_update` paketini gönderir. Rust trace hedef konumu, state ID'si ve ACK sequence'ini kaydeder.

## Sınır

Bu dilim survival kazma süresi, kazma ilerleme/iptal, ray/hit seçimi, placement, item use, prediction/rollback, client world cache ve görsel/ses feedback'ini uygulamaz. Hedefin spawn altındaki blok olarak seçilmesi kontrollü server probe'udur; genel oyuncu hedefleme algoritması değildir.
