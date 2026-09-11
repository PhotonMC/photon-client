# P12 — Creative snowball use sözleşmesi

Bu dilim, creative oyuncunun hotbar'ına bir snowball stack'i yerleştirip ana elle `use_item` göndermesini sınar. Amaç dolu stack item kullanımının sequence ACK'ini, projectile entity spawn'ını ve spawn packet'indeki owner verisini gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

`ServerboundUseItemPacket` alanları hand, sequence, yRot ve xRot sırasındadır; packet ID'si `67`'dir. Snowball item'i server level'da oyuncunun rotasyonundan bir snowball projectile üretir. `ClientboundAddEntityPacket` entity registry ID'sini `120` olarak taşır ve packet data alanı projectile owner entity ID'sisini taşır. Pinned vanilla item registry'de snowball ID'si `1044`'tür.

## Probe girdisi ve akışı

- Oyun modu: `CREATIVE`
- Creative slot: `36`, count `1`, item registry ID `1044`
- Carried hotbar slotu: `0`
- El: `MAIN_HAND`, ordinal `0`
- Use sequence: `11`
- yRot/xRot: `0.0f` / `0.0f`
- Ön koşul: configuration tamamlanmış, ilk chunk alınmış ve `player_loaded` gönderilmiş oturum

Rust probe slot stack'ini, carried slotunu ve tek `use_item` paketini yollar; gelen block ACK ve `add_entity` gövdesini tam tüketerek kaydeder. Beklenen sonuç bir slot paketi, bir carried paketi, bir use paketi, ACK `[11]` ve entity type ID `120` olan tek spawn kaydıdır. Spawn kaydının `data` alanı server'ın oyuncu entity ID'siyle eşleşir.

## Sınır

Bu dilim projectile'ın spawn packet'ini doğrular. Creative stack tüketiminin inventory packet'lerini, projectile'ın tick hareketini, block/entity hit sonucunu, impact event'ini, ses/partikül ve client prediction'ı kapsamaz.
