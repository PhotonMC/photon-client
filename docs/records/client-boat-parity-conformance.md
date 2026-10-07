# T38 — Tekneler, sandıklı tekneler ve sallar — [client-parity]

Bu kayıt, teknenin (tekne, sandıklı tekne, bambu salı) modelinin, çizim duruşunun ve istemci tikinin (vuruşun sönmesi,
kabarcık sütununun sallaması, su altı durumu) Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı
`client-parity`'dir: modeller gerçek `BoatModel` ve `RaftModel`'e, duruş gerçek `AbstractBoatRenderer.submit`'e,
tik gerçek `AbstractBoat`'a karşı karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `AbstractBoatModel.setupAnim`: her kürek, kürek çekme süresinin `sin(−t)` ve `sin(−t + 1)` ile sarılıp `clampedLerp`
  edilmesiyle sallanır (`xRot` `−π/3` ile `−π/12` arası, `yRot` `−π/4` ile `π/4` arası; sağ kürek `π − yRot`).
  Tekne, sandıklı tekne ve sal ayrı katmanlardır (sal: `RaftModel`, bambu).
- `AbstractBoatRenderer.submit`: `(0, 0,375, 0)` kayma, `180 − yRot` dönüş, vuruş varsa `sin(t) × t × hasar / 10 × yön`
  derece x ekseninde sallanma, suyun altında değilken ve kabarcık açısı sıfırdan farklıyken `(1, 0, 1)` ekseninde
  kabarcık dönüşü, `(−1, −1, 1)` ölçek ve çeyrek dönüş. **Not:** `Quaternionf.setAngleAxis` ekseni normalleştirmez
  (ilk karşılaştırma bunu yakaladı: normalleştirilmiş hâl yarı değeri veriyordu).
- `AbstractBoat.tick` (istemci): durum kontrolü (başlangıçta), vuruş süresi ve hasar tikte bir azalır, kabarcık sütunu
  çarpanı `+0,05` ya da `−0,1` ile 0 ile 1 arasında, açı `10 sin(0,5 yaş) × çarpan`, küreklerin konumu çekilirken
  `π/8` artar, yoksa sıfırlanır. Kürek çekme durumu veri **ve** bir yolcu ister.
- Suyun altında olma durumu: kutunun üstünden bir binde bir yukarıya kadar bir su bloğunun yüksekliği o noktayı aşıyorsa.
- Veriler: vuruş süresi 8, vuruş yönü 9, hasar 10 (`VehicleEntity`), sol ve sağ kürek 11 ve 12, kabarcık süresi 13.
- Oracle sahnesi yaşayan olmayan varlıkları da tikleyebilir hale geldi (sahneye bir iskor tablosu verildi: tekne
  itilebilir varlıklar için takım sorar).

## Kapsam

`entity/boat-model` 20 satır (tekne, sandıklı tekne, bambu salı, sandıklı sal ve kiraz teknesi: durgun, kürek çekerken, uzun
çekmiş, tek yanlı); `entity/boat-pose` 10 satır (durgun, dönmüş, vuruş iki yönde ve süresi dolmuş, kabarcık ve su
altında, bir parçacık kadar kabarcık, hepsi); `entity/variant-fields`'e altı satır; `living-entity-animations`'a 7 satır
(yüzer, vurulur, kabarcıkla sallanır, su altı, sığ su, suyun üstünde, karada).

## Sapmalar (açık)

- Su yaması (`waterMask`, teknenin içine su girmesini önleyen yalnız derinlik yazan model) çizilmez: yalnız derinlik
  yazan bir çizim aşaması yok (modeli 0×0 dokulu, oracle'da NaN üretir, satırı çıkarıldı).
- Yolcusu olan teknenin kürek çekmesi (kürek konumunun ilerlemesi) oracle'da yok: sahnede yolcu kurulmuyor. Mantık
  yazıldı (veri ve yolcu birlikte), karşılaştırılmadı.
- Teknenin kendi yüzme, itme ve yolcu denetimi (`floatBoat`, `controlBoat`) yerel kullanıcı dışında yok. Gölge, ses,
  parçacık yok. Canlı gözlem yapılmadı.
