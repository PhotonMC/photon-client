# T39 — Ok ve spektral ok — [client-parity]

Bu kayıt, ok ve spektral okun modelinin (saplanınca titreme), yerleşimini ve istemci durumunu (titreme süresi, yerde olma ve
iksir rengi verisi) Java 26.2 ile aynı sonucu vermesi için kapsar. Kanıt sınıfı `client-parity`'dir: model gerçek `ArrowModel`'e,
yerleşim gerçek `SpectralArrowRenderer.submit`'e (`ArrowRenderer`'ın ortak kodu), durum gerçek `AbstractArrow` tikine karşı
karşılaştırılır. Yüz ayıklama ve ışık oracle ile karşılaştırılmaz. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `ArrowModel.setupAnim`: kök, `shake > 0` iken `−sin(shake·3)·shake` derece kadar z ekseninde döner. Model iki çapraz düzlem ve
  bir kuyruk düzlemidir (kalınlıksız küpler); katman 0.9 ile ölçeklenir.
- `ArrowRenderer.submit`: `Y(yaw − 90)`, `Z(eğim)`; yaw ve eğim `getYRot(kısmiTik)`/`getXRot(kısmiTik)` (düz aradeğer); titreme
  `shakeTime − kısmiTik`. Boru hattı `entityCutoutCull`: arka yüzler çizilmez. İki yüzü aynı yerde olan düzlemlerde yalnız göze bakan
  çizilir; Rust'ta entity boru hattı yüzleri ayıklamadığından, bu model için üçgenler CPU'da normal ve göz konumuyla ayıklanır.
- `AbstractArrow.shakeTime`: her tik 0'a kadar bir azalır; `IN_GROUND` verisi (10) doğru olunca, ilk tik geçmişse ve titreme sürmüyorsa 7
  olur (`onSyncedDataUpdated`). `Arrow.ID_EFFECT_COLOR` (11): `0`'dan büyükse ok iksirlidir (`arrow_tipped`), değilse `arrow`;
  spektral ok kendi dokusunu (`arrow_spectral`) taşır. Renderer iksir rengiyle boyamaz.

## Kapsam

`entity/arrow-model` 8 satır (durgun, titreme süresinin ucları ve ortası, kısmi tik, negatif); `entity/arrow-pose` 7 satır (eksenler,
çapraz, yukarı/aşağı, sarılmış yaw, kaydırılmış konum); `living-entity-animations`'a 6 satır (uçan, saplanan, yeniden saplanan, ilk
tikte saplanan, iksirli, spektral). Hareket (hava sürtünmesi ve yer çekimi) önceki arrow-tick kaydındadır.

## Sapmalar (açık)

- Saplanma çarpışması, kritik atış parçacıkları, ok isabeti ve sesler yok; yerdeki okun konumu sunucunun gönderdiğidir.
  Trident ve diğer mermiler ayrı görevlerde ([T39b](../tasks/T39b-trident.md) ve sonrası). Işık: gözden okunur.
