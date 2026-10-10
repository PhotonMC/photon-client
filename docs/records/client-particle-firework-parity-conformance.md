# T74c — Havai fişek parçacıkları — [client-parity]

Bu kayıt `FireworkParticles`'ı kapsar: başlatıcı (`Starter`), kıvılcım (`firework`) ve parlama
(`flash`); roketin `EntityEvent` 17'si (`ClientLevel.createFireworks`) ve `fireworks` bileşeninin
patlama listesi. Kanıt sınıfı `client-parity`'dir: `movement/particle-engine` satırları gerçek
motoru gerçek sağlayıcılarla koşturur (başlatıcı sıradaki tohumla kurulur, kıvılcımlar motorun
tohum kuyruğundan alır) ve parçacık sayıları, sesler ve her alanın özeti bit bit eşittir.

## Java kaynak incelemesi

- Başlatıcı `NoRenderParticle`dir (`NO_RENDER` grubu). Ömrü `2·patlama − 1`, bir patlama
  kıpırdıyorsa +15. İlk tikte patlama sesi (üç ve üstü patlama ya da büyük top: büyük; 16 blok ve
  üstü uzaktaysa `_far`; perde `0.95 + r·0.1`), çift tiklerde sıradaki patlamanın kıvılcımları ve
  bir parlama, sondaki tikte (kıpırdama varsa) kıpırdama sesi (perde `0.9 + r·0.15`).
- Kıvılcımlar `engine.createParticle` ile doğrudan yapılır (seviyenin sınırlayıcısından
  geçmez); renk ve solma rengi başlatıcının rastgelesinden seçilir. Renksiz patlama
  `DyeColor.BLACK.getFireworkColor()` (1973019) alır. Top, yıldız, creeper ve patlama biçimleri
  kaynaktaki sırayla çekilir.
- Kıvılcım: `SimpleAnimatedParticle` (sürtünme 0.91, yerçekimi 0.1), boy ×0.75, ömür
  `48 + nextInt(12)`, tam parlak, ikinci yarısında alfa söner ve solma rengine beşte bir
  yaklaşır. İz bırakan kıvılcım, ilk yarısında `(yaş + ömür)` çiftken, yarı yaşta, kendi anlık
  renginde izsiz bir kıvılcım bırakır. Kıpırdayan kıvılcım ömrünün üçte birinden sonra yanıp söner.
- Parlama (`OverlayParticle`): ömür 4, boy `7.1·sin((yaş + a − 1)·π/4)`, alfa
  `0.6 − (yaş + a − 1)·0.125`, rengi seçenekten.
- Patlamasız roket seviyeden 2–4 `poof` bırakır (sınır her turda yeniden çekilir).

## Kanıt

`movement/particle-engine`: küçük ve büyük top (solmalı), yıldız, creeper, patlama, renksiz,
iki ve üç patlama, kıpırdama beklemesi, yakın ve uzak ses, patlamasız roket. Oracle'da kıvılcımın
bıraktığı iz kıvılcımı tohumsuz yapıldığı için yoktur; iz, kıpırdama ve solma Rust birim
testleriyle kanıtlıdır. `fireworks` bileşeninin ağ okuması birim testlidir.

## Sapmalar (açık)

- Uzaktaki seslerin mesafe gecikmesi (`playDelayed`) yok: ses motorunda ertelenmiş çalma yok, ses
  hemen çalar. Oracle da erteli sesi hemen çalmış sayar.
- Canlıda (sunucu `summon firework_rocket`, `shot`) iki patlamalı roketin kıvılcımları gözlendi; kıpırdayan yıldızın yanıp sönmesi ve izler gözlenmedi. Parçacık seçeneği (azalt/en az) henüz tutulmadığı için doğrudan
  yapım `override_limiter` ile temsil edilir (seçenek `All` iken eşdeğer).
