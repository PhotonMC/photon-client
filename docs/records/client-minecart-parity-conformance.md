# T38b, T38c — Minecart ailesi (eski ve yeni davranış) — [client-parity]

Bu kayıt, minecart'ın (ve sandık, ocak, huni, TNT, spawner, komut bloğu türevlerinin) modelinin yerleşimini
(`AbstractMinecartRenderer`, eski davranışın çıkardığı durumdan), taşıdığı bloğun yerleşimini, rayın verdiği konumu
(`OldMinecartBehavior.getPos`, `getPosOffs`), vuruş sallanmasını ve konum-dönüş yaklaşmasını Java 26.2 ile aynı sonucu
vermesi için kapsar. Kanıt sınıfı `client-parity`'dir: yerleşim gerçek `MinecartRenderer.submit`'e, ray sorguları ve tik
gerçek `Minecart` varlığına karşı karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- Hangi davranışın kullanıldığı özellik bayrağına bağlıdır: `minecart_improvements` varsayılan (`vanilla`) kümede
  değildir, yani sunucu açmadıkça **eski** davranış kullanılır. Yeni davranış T38c bölümündedir.
- `AbstractMinecartRenderer.submit`: kimlikten türeyen küçük bir kayma (`offsetSeed`: `(id·493286711)` ve ikinci dereceden bir
  karışım; her eksen için üç bitlik bir dilim, 0.004 ölçeğinde); eski çizim: ray üzerindeyse konum rayın verdiği yere kayar,
  yön önündeki ve arkasındaki ray noktalarından (`getPosOffs(±0.3)`) çıkarılır (`atan2` ve `atan·73`), değilse gönderilen dönüş;
  `(0, 0.375, 0)`, `Y(180 − yaw)`, `Z(−eğim)`; vuruş varsa `X(sin(t)·t·hasar/10·yön)`; taşıdığı blok için `0.75` ölçek,
  `(−0.5, (kayma − 8)/16, 0.5)`, `Y(90)`; en sonda model için `(−1, −1, 1)`.
- `OldMinecartBehavior` (istemci): `AbstractMinecart.tick` önce vuruş süresi ve hasarı birer azaltır; konum ve dönüş son paketin
  hedefine üç tikte yaklaşır (aradeğerleme sürüyorsa), sürmüyorsa dönüşler 360'a bölümünden kalana alınır. `getPos`: rayın iki
  çıkışı arasında, eğimli rayda yüksekliğiyle; `getPosOffs`: rayın yönünde verilen kadar ileri/geri.
- Veri: `VehicleEntity` (8 vuruş süresi, 9 yön, 10 hasar), `AbstractMinecart` (11 özel blok durumu, 12 blok kayması), ocağın 13
  yakıtı. Taşınan blok: sunucunun adlandırdığı, yoksa türünün kendisi (ocak yakıt varken yanık; sandık ve ocak kuzeye bakar;
  düz minecart yok); kayma: sunucunun dediği, yoksa türünün (6; sandık 8; huni 1).

## Kapsam

`entity/minecart-pose` 16 satır (rayda ve rayda değil, düz ve iki eksende, yokuş yukarı ve aşağı, köşe, uçlar çakışan, vuruşlu ve
vuruş bitmiş, başka kimlik, bloklu üç kayma; her satırda modelin ve bloğun pozu); `living-entity-animations`'a 14 satır (boşta,
vuruş, her tür, ocak yakıtı, bloğu adlandırma ve geri alma, taşıma, hareket, hareket ve dönüş, ışınlanma ve 19 yerde ray sorgusu:
düz, yokuş, köşe, hepsi dört ray türünde).

## TNT minecart (T38d)

- `MinecartTNT`: olay 70 fitili 80 tike kurar (`primeFuse`), istemcide her tik bir azalır (0'da kalır; patlama sunucunun).
  `TntMinecartRenderer`: kalan fitil `fitil − kısmiTik + 1` (yanık değilse −1); `−1 < f < 10` iken blok `getSwellAmount(f)`
  kadar şişer (`(−0.5·şişme, 0, −0.5·şişme)` kayması ve `1 + şişme` ölçeği, blok yerleşiminden sonra); `isLit(f)` (`(int)(f/5)` çift)
  iken blok tam beyaz bindirmeyle çizilir (`OverlayTexture.pack(15, 10)`, yoksa `NO_OVERLAY`). Kayıt, `entity/minecart-pose`'a 11
  satırla (fitilin her aralığı, kısmi değerler, rayda ve vuruşlu) ve bloğa verilen bindirmenin karşılaştırılmasıyla genişledi;
  `living-entity-animations`'a 2 satır (olay 70, yeniden kurma). Spawner minecart istemcide yaratık göstermez (sunucu doğurma
  verisini göndermez): düz spawner bloğudur.

## Yeni (deneysel) davranış (T38c)

- Sunucu `minecart_improvements`'ı açtıysa (`ClientboundUpdateEnabledFeatures`) minecart'lar bundan sonra `NewMinecartBehavior` ile
  yapılır (bayrak yapım anında okunur). Bu davranışın `getInterpolation`'ı yoktur: hareket, dönüş ve ışınlanma paketleri konumu ve
  dönüşü yaklaştırmadan **hemen** koyar (`setXRot` yine ±90'a sıkıştırır); `lerpMotion` hareketi doğrudan atar.
- `ClientboundMoveMinecartPacket` (kimlik 55): minecart ve adımları (konum ve hareket çift, iki dönüş birer bayt, ağırlık float). Adımlar
  bir sonraki tikte oynatılmak üzere biriktirilir.
- `lerpClientPositionAndRotation` her tik: gecikme bir azalır; sıfıra inince eski değerler (konum, hareket, dönüşler) saklanır,
  oynanan adımlar atılır ve biriken adımlar alınır (ağırlıklar toplanır; toplam sıfırsa gecikme 0, değilse 3). Oynuyorsa konum, hareket
  ve dönüşler `(3 − gecikme + kısmiTik) / 3` noktasındaki adımdan alınır (ağırlığa göre; dönüşler `rotLerp` ile kısa yoldan).
  `getCurrentLerpStep` tek girişlik bir bellek tutar (kısmi tik ve gecikme aynıysa eski cevabı verir; adımlar değişse de silinmez):
  ağırlığı sıfır adımlarda (`adjustToRails` anında) bu, bir önceki paketin cevabını döndürür; Rust bellek giriş değerini aynen taşır.
- Çizim (`newExtractState`, `getRenderOffset`, `newRender`): oynuyorsa konum, `x/y rot` oynatmadan; yoksa varlığın kendi dönüşü
  (kısmi tiksiz). Çizim konumu `x + (oynatma konumu − x)`; poz `Y(yRot)`, `Z(−xRot)`, `(0, 0.375, 0)`; ray bakılmaz.

Kapsam: `entity/minecart-pose`'a 4 satır (`new_render`: düz, dönük ve eğik, rayı yok sayan, vuruş ve bloklu); `living-entity-animations`'a
5 satır (boşta, adımlar, dört paket ve bekleyen, ağırlıksız adımlar, hareket/dönüş/ışınlanma paketlerinin anında koyması; her tikte
konum, hareket, dönüş ve iki kısmi tikte oynatma yeri).

## Sapmalar (açık)

- Sandık minecart'ın sandığı blok varlık (özel) modeliyle çizildiğinden henüz görünmez; diğer bloklar normal blok modelidir.
- Yeni davranışta istemcinin `onRails` bayrağı (yalnız ses ve adım olayları okur) tutulmaz; çarpışma/itme ve sunucu kolu sunucunundur.
- Taşınan bloğun ışığı ve ışığın çizimi oracle'sızdır; gölge ve ses yok. Doku yolu elle yazıldı.
