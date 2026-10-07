# T32d — Hoglin ve zoglin — [client-parity]

Bu kayıt, hoglin ve zoglinin (yetişkin ve bebek) modelinin Java 26.2 ile Rust'ta aynı köşe konum, doku
koordinatı ve normali üretmesini; saldırı sallanmasının istemci durumunun aynı ilerlemesini kapsar. Kanıt
sınıfı `client-parity`'dir (model ve tik) ve canlı gözlemdir: gerçek `HoglinModel` ve `BabyHoglinModel`
katman tablosundan pişirilip gerçek `setupAnim` ile pozlanır; sayaç gerçek `Hoglin` ve `Zoglin`
varlığıyla ve gerçek varlık olayıyla karşılaştırılır.

## Java kaynak incelemesi

- `HoglinModel.setupAnim`: kulaklar yürüyüşle `hız·sin(konum)` kadar çırpınır (sağ ve sol zıt işaretli);
  baş yalnız yatay döner, pitch okunmaz; baş eğimi saldırı kesriyle `0,8727` ile `-π/9` arasında
  ara değer alır; kesir `1 - |10 - 2·kalan|/10` (tam sayı aritmetiği). Ön sağ ve arka sol bacak aynı
  açıyı, ön sol ve arka sağ bacak karşıt açıyı alır (`1,2` çarpanı). Bebekte baş ayrıca kesir·`2,5`
  alçalır.
- `Hoglin` ve `Zoglin` (istemci): olay 4 saldırı sallanmasını 10 tike kurar, her `aiStep` bir azaltır.
  `getAttackAnimationRemainingTicks` çizim durumuna gider.
- Doku: hoglin ve zoglin için tek ve bebek dokusu; çeşit yoktur.

## Kapsam

`entity/hoglin-model` 28 satır (dört katman × duruş, bakış, yürüyüş, sallanmanın beş noktası);
`living-entity-animations`'a üç satır (boşta, sallanan hoglin, iki kez sallanan zoglin).

## Titreme (T32i) — [client-parity]

`LivingEntityRenderer.setupRotations` donmuş (`isFullyFrozen`, 140 tik) her mobun, `Hoglin.isConverting`
olan hoglinin (bağışık değil, `NoAI` değil, boyutun `gameplay/piglins_zombify` değeri doğru) ve boğulan
striderin gövde açısına `cos(⌊yaş⌋·3,25)·π·0,4` derece ekler. `entity/shaking-rotations` (55 satır: inek,
hoglin ve strider × beş tik/kesir) gerçek `setupRotations`ın matrisini, gerçek `HoglinRenderer` ve
`StriderRenderer` `isShaking` geçersiz kılmalarıyla karşılaştırır; Rust kararı istemci dünyasından
(veri, `NoAI` bayrağı, boyut ortamı, tik sayacı) verir. Boyutun özniteliği oracle'da kayıt defterinin
kendi boyut türünden okunur (`overworld` doğru, `the_nether` yanlış); kayıt verisindeki `attributes`
alanı (düz değer veya `BooleanModifier`) birim testle çözülür. Hoglinin bağışıklık verisinin
kimliği `entity/data-ids` tablosuna girer. Zoglin `ZoglinRenderer` ile titremez.

## Sapmalar (açık)

- Dalın kaynağı boyut türünün kendi özniteliğidir; biyom ve hava katmanları `piglins_zombify`ı
  değiştirmez (kaynakta yalnız boyut türü ayarlıyor), veri paketi biyomu ayarlarsa istemci bunu
  okumaz.
- Sallanma sesi ve hoglin sesleri yok.
