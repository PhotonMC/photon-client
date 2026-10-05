# T38b — Minecart ailesi (eski davranış) — [client-parity]

Bu kayıt, minecart'ın (ve sandık, ocak, huni, TNT, spawner, komut bloğu türevlerinin) modelinin yerleşimini
(`AbstractMinecartRenderer`, eski davranışın çıkardığı durumdan), taşıdığı bloğun yerleşimini, rayın verdiği konumu
(`OldMinecartBehavior.getPos`, `getPosOffs`), vuruş sallanmasını ve konum-dönüş yaklaşmasını Java 26.2 ile aynı sonucu
vermesi için kapsar. Kanıt sınıfı `client-parity`'dir: yerleşim gerçek `MinecartRenderer.submit`'e, ray sorguları ve tik
gerçek `Minecart` varlığına karşı karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- Hangi davranışın kullanıldığı özellik bayrağına bağlıdır: `minecart_improvements` varsayılan (`vanilla`) kümede
  değildir, yani sunucu açmadıkça **eski** davranış kullanılır. Yeni davranış bu görevde yok (aşağıda).
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

## Sapmalar (açık)

- Yeni (deneysel) davranış, TNT minecart'ın fitil beyazı ve spawner minecart'ın içindeki yaratık yok
  ([T38c](tasks/T38c-minecart-yeni-davranis.md), [T38d](tasks/T38d-minecart-tnt-spawner.md)). Sandık minecart'ın sandığı
  blok varlık (özel) modeliyle çizildiğinden henüz görünmez; diğer bloklar normal blok modelidir.
- Taşınan bloğun ışığı ve ışığın çizimi oracle'sızdır; gölge ve ses yok. Doku yolu elle yazıldı.
