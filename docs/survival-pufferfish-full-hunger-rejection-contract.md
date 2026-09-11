# P12 — Survival pufferfish full-hunger rejection sözleşmesi

Bu dilim, default pufferfish'in survival oyuncu tam açlıktayken
kullanılmamasını sınar. `use_item` sequence ACK'i alınır; consumable kullanım
durumu başlamaz, stack tüketilmez ve pufferfish effect'leri uygulanmaz.

## Java referansı

`Items.java:1321-1325` pufferfish'i `Foods.PUFFERFISH` ile kaydeder.
`Foods.java:33` food'un `can_always_eat` değerini false bırakır. Player'ın
`canEat` yolu `Player.java:1600-1606` içinde yalnızca invulnerable, always-eat
veya `FoodData.needsFood()` koşullarından biri true olduğunda başarılıdır;
`FoodData.java:88-98` tam food seviyesinin `20` olduğunu tanımlar.

Server listener `ServerGamePacketListenerImpl.java:1417-1443` sequence'i item
davranışından önce ACK kuyruğuna alır. `Item.java:180-194` consumable başlangıcı
ancak item kullanımı kabul edildiğinde çalışır; bu senaryoda `canEat` false
olduğu için tüketim tamamlanma yolu açılmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`, yeni oyuncunun başlangıç food seviyesi `20`
- Sunucu console hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:pufferfish`
- Hazırlık sonrası serverbound `SET_CARRIED_ITEM`, hotbar slot `0`
- Kullanım: serverbound `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `38`
- Kullanım sonrası drain: 2.5 saniye; reddin ve başlangıç paketlerinin gözlenmesi beklenir

Beklenen invariant'lar carried/use packet sayılarının `1/1` olması, hazırlık
slotunun state `2`, slot `36`, count `1`, item ID `1089`, component `[]`
taşıması, başlangıç content'inin state `1` ve 46 slot olması, ACK listesinin
`[38]` olmasıdır. Kullanım sonrasında ek slot update'i, effect update'i ve
effect remove'u beklenmez; default stack tüketilmeden kalır.

## Sınır

Bu sözleşme yalnızca full-hunger eligibility reddini kapsar. Food seviyesi
düşük pufferfish tüketimi [food component override sözleşmesinde](survival-pufferfish-use-contract.md),
diğer food/effect varyantları ve inventory click sonraki dilimlerdir.
