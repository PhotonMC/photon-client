# T36d — Vex ve allayın elindeki eşya — [client-parity]

Bu kayıt, vex ve allayın elinde tuttuğu eşyanın nerede çizildiğinin (`ItemInHandLayer` ve modelin
`translateToHand`'i) Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir: duruş,
gerçek `VexModel` ve `AllayModel` ile gerçek `ItemInHandLayer`'ın verdiği matristen karşılaştırılır. Canlı
gözlem `server-conformance`.

## Java kaynak incelemesi

- `VexModel.translateToHand`: kök, gövde ve kol dönüştürülür; eşya 0,55'e küçülür; ana kolda `+0,046875`, öbüründe
  `−0,046875` yatay, `−0,15625` dikey, `0,078125` ileri kayar. Kolların duruşu modelin kendi `setupAnim`'indedir
  (yüklenirken kollar uzanır).
- `AllayModel.translateToHand`: kök ve gövde dönüştürülür; `(0, 0,0625, 0,1875)` kayma; **sağ kolun** `xRot`'u kadar
  x ekseninde dönüş (iki el için de); 0,7 ölçek; `(0,0625, 0, 0)` kayma.
- İkisinden sonra `ItemInHandLayer`: x ekseninde −90°, y ekseninde 180°, `(±1, 2, −10) / 16` kayma; sağ el önce.
  Soldaki el tutulan eşyanın durumundan (`leftHandItemState`) gelir; ana kolun solak olması yalnız hangi elin hangi
  yuvadan okunacağını değiştirir (çizimde `main_arm_is_left`).
- Oracle matrisi, eşya modelinin köşeye çekilmesini (`−0,5`, `−0,5`, `−0,5`) içerir; adaptör aynısını uygular.

## Kapsam

`entity/flier-item-pose` 12 satır (vex'te sağ, sol, iki el, yüklenirken; allay'da sağ, sol, iki el, uçarken, kısmen
tutarken, boş el).

## Sapmalar (açık)

- Bebek kayması ve mızrak, eşya kullanımı duruşları uygulanmaz (vex ve allay bebek olmaz, kullanım okunmuyor).
  Eşyanın kendi çizimi mevcut el eşyası yoluyla yapılır. Canlı gözlem yapılmadı.
