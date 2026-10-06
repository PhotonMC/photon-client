# T44a — Oyuncunun tekneyi sürmesi: `AbstractBoat.tick`, `LocalPlayer.rideTick`, `MoveVehicle`, `PaddleBoat` — [client-parity]

Bu kayıt, yerel oyuncunun oturduğu ve denetimini taşıdığı teknenin (tekne, sandıklı tekne, bambu salı)
istemci tikini ve oyuncunun yolcu olarak tikini kapsar. Kanıt sınıfı `client-parity`'dir: gerçek Java
`ClientLevel` içine gerçek `LocalPlayer` ve gerçek tekne konur, `level.tickNonPassenger(tekne)` (tekne, sonra
yolcusu `rideTick` ile) koşturulur ve her tikte oyuncunun ve teknenin durumu ile gönderilen paketler (bayt bayt)
Rust'ınkiyle karşılaştırılır.

## Java kaynak incelemesi

- `AbstractBoat.tick`: durum (`getStatus`: su altı, akan su altı, suda, karada, havada; `isUnderwater`,
  `checkInWater`, `getGroundFriction`), `floatBoat` (durumuna göre sürtünme, kaldırma, havadan suya iniş:
  `getWaterLevelAbove`), `controlBoat`, `ServerboundPaddleBoatPacket`, `move(SELF)`, iki kez
  `applyEffectsFromBlocks`, sonra kürek konumları ve kürek sesi için durumun yeniden bulunması (`getPaddleSound`
  → `getStatus`, yan etkileri `waterLevel` ve `landFriction`).
- `Entity.move`: çarpışma (teknenin basamak yüksekliği yok), `setOnGroundWithMovement`, `checkFallDamage`
  (teknenin: `lastYd`), `restituteMovementAfterCollisions` (cansız varlıkta blok sekmesi 0,8 katı), blok hız
  çarpanı; `Entity.baseTick`'in akışkan itmesi.
- `LocalPlayer.tick`/`rideTick`: yolcu paketleri (`PlayerInput` değişince, `MovePlayerRot`, `MoveVehicle`,
  sprint komutu), `Player.tick` (yolcuda yerde değil), yolcunun akışkan kutusu
  (`AbstractBoat.modifyPassengerFluidInteractionBox`), `Player.travel` (yolcu doğrudan `LivingEntity.travel`),
  yürüme animasyonunun durması, çömelmemesi/yüzmemesi, sprint'in araç tarafından kapatılması,
  `AbstractBoat.positionRider` (koltuk, dönüş, `clampRotation`), `handleSetEntityPassengersPacket`'in oyuncuyu
  teknenin baktığı yöne çevirmesi.
- `ClientLevel.hasChunk` her yığın için doğru döner; Rust'ın oyuncu yolunda yüklü yığın sorgusu buna
  uydurulmuştu (yüklü olmayan yığında yerçekimi −0,1 sanılıyordu); düzeltildi.

## Kanıt

- `movement/ride-boat`: 40 satır (hem Java hem Rust tarafında gerçek sınıflar, bit bit eşit): suda boşta,
  ileri/geri, sola/sağa dönüş, iki yöne birden, girdi değişimi, hareketli başlangıç, havadan suya ve kara
  düşüş, sığ ve derin göl yüzeyi, su altı, akan su, kabarcık sütunu, kıyı, duvar; karada buz, mavi buz,
  balçık, bal, ruh kumu, nilüfer yaprağı, çit, merdiven; balçığa düşme, örümcek ağı, toz kar; sprint/eğilme/
  zıplama tuşları; sandıklı tekne ve bambu salı. Her tikte oyuncunun konumu, hareketi, bakışı, bayrakları,
  modeli besleyen değerleri, teknenin konumu, hareketi, dönüşü, durumu, `deltaRotation`, kürekleri, su
  seviyesi, kara sürtünmesi, `lastYd` ve gönderilen paketler (`paddle_boat`, `player_input`,
  `move_player_rot`, `move_vehicle`, `player_command`).
- Birim: `control_applies_java_input_delta_and_paddles` (altı satırlık denetim tablosu), yolcu dönüşü.
- Canlı (`server-conformance`): sunucuda çağrılan teknede oyuncu bindi, ileri ve dönüşle göletin kıyısına
  sürdü; sunucu istemcinin `MoveVehicle` konumunu kabul etti.

## Sapmalar (açık)

- Teknenin başka varlıkları itmesi (`push`, `EntitySelector.pushableBy`) ve varlık çarpışmaları yok:
  teknenin başka varlıkla çarpışması hesaba katılmaz.
- Kabarcık sütunundaki sunucu dalı (`isAboveBubbleColumn`) istemcide çalışmaz (Java'da da yalnız ses/parçacık).
- `isInterpolating`: yüklü yolcu teknesinde bekleyen sunucu konumu enterpolasyonu yok; `MoveVehicle` hep tekne
  konumunu yollar.
- Oyuncunun `yRotO` değeri izlenmiyor (yolcu bakışının kamerada enterpolasyonu); `clampRotation`'ın
  `yRotO += ...` kısmı uygulanmıyor.
- Yolcu sayısı birden fazla olan teknede ikinci yolcunun hareketi, binme/inme (`startRiding`, çıkış
  konumu `getDismountLocationForPassenger`), `mount.onboard` iletisi ve diğer araçlar ([T44b](tasks/T44b-binek-hayvanlar-minecart.md)).
