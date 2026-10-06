# T44b1 — Oyuncunun eyerli atı sürmesi: `travelRidden`, `AbstractHorse.tickRidden`, atlama yükü, `PlayerCommand` — [client-parity]

Bu kayıt, yerel oyuncunun oturduğu ve denetimini taşıdığı eyerli at ailesinin (at, eşek, katır, iskelet atı,
zombi atı) istemci tikini ve oyuncunun binici olarak tikini kapsar. Kanıt sınıfı `client-parity`'dir: gerçek
Java `ClientLevel` içine gerçek `LocalPlayer` ve gerçek bir evcil, eyerli at konur, `level.tickNonPassenger(at)`
koşturulur ve her tikte oyuncunun ve atın durumu ile gönderilen paketler (bayt bayt) Rust'ınkiyle karşılaştırılır.
Tekne için bkz. [T44a](client-boat-ride-parity-conformance.md).

## Java kaynak incelemesi

- `LivingEntity.aiStep`/`travelRidden`/`travel` (`travelInAir`, `travelInFluid`: su, lav, `floatInWaterWhileRidden`),
  `handleRelativeFrictionAndCalculateMovement`, `getFrictionInfluencedSpeed` (havada atın hızının onda biri, çünkü
  denetleyen bir oyuncu), `Entity.move` (basamak yüksekliği özniteliği, çarpışma, sekme, blok hız çarpanı),
  `LivingEntity.checkFallDamage`, `Entity.applyEffectsFromBlocks` (balçık, bal, ağ, tatlı çilek, toz kar, kabarcık
  sütunu; canlı için).
- `AbstractHorse`: `getRiddenInput` (yan yarım, geri çeyrek; kalkıkken sıfır), `tickRidden` (binicinin bakışına
  dönmek, yerdeyken bekleyen atlayışı yapmak), `executeRidersJump`, `onPlayerJump`/`standIfPossible` (atlayışta
  kalkma, yirmi tik, `allowStandSliding`), `onClimbable` (hiç tırmanmaz), `getPassengerAttachmentPoint` (koltuk ve
  kalkarken yukarı/geri), `positionRider` (binicinin gövdesi atın gövdesiyle), `addPassenger` (binici atın baktığı yöne
  bakar), `canSprint` (yalnız oyuncu ve deve).
- `LocalPlayer.aiStep`'in `jumpableVehicle` dalı (atlama tuşunun yükü, bırakınca `onPlayerJump` ve
  `START_RIDING_JUMP` komutu), yolcu kapıları (T44a), `ServerboundPlayerCommandPacket`'in yük alanı.

## Kanıt

- `movement/ride-horse`: 33 satır (hem Java hem Rust tarafında gerçek sınıflar, bit bit eşit): boşta, ileri, geri, yan,
  çapraz, sprint/eğilme tuşları, dönüş, bakış (yukarı/aşağı), küçük/tam/ileri/çift atlayış, basamak, çit, merdiven,
  duvar, sığ ve derin su, suda atlayış, buz, balçık, bal, ruh kumu, ağ, düşüş, kabarcık, merdiven; eşek, katır, iskelet
  atı, zombi atı. Her tikte oyuncunun konumu, hareketi, bakışı, bayrakları ve gönderilen paketler (`player_input`,
  `player_command` yüklü atlayış, `move_player_rot`, `move_vehicle`), atın konumu, hareketi, dönüşü, çarpışma
  bayrakları, düşüş mesafesi, bekleyen atlayış ve baş dönüşü.
- `entity/pick-table`'a (`movement/entity-pick-table`) yolcu koltukları eklendi (`EntityAttachment.PASSENGER`, her
  tür için, `tools/generate/entity_picks.py`); her koşuda baştan sona karşılaştırılır.
- Canlı (`server-conformance`): sunucuda çağrılan eyerli ata binildi, ileri, atlayış ve dönüşle sürüldü.

## Sapmalar (açık)

- Atın başka varlıklarla çarpışması ve itilmesi yok; at bebek/ölçekli olunca koltuk ve kutu ölçeği ( `scale` ) koltuk
  ofsetinde yok.
- Atın kendi hareketinin sesleri (nal, atlama, soluma) ve parçacıkları yok.
- Atın gövde dönüşü (`yBodyRot`) `LivingEntity.tick` sonu tarafından değil `tickRidden` ile kurulur; yürüme animasyonu
  genel mob yolundan gelir (Java'ya karşı karşılaştırılmadı).
- Atın zıplama boost etkisi (`JUMP_BOOST`) ve yavaş düşme/levitasyon etkileri okunmuyor; bu etkiler yerel oyuncuda da yok.
- Atın veri bayrakları (kalkma, otlama) sunucudan gelir; istemcinin kendi kalkması yirmi tik sayılır.
- Diğer binekler (eyerli domuz, strider, deve, minecart…): [T44c](tasks/T44c-diger-bineklar.md).
- `LocalPlayer` ile `MountBody` ayrı `LivingEntity` hareket portlarıdır (oyuncu ve mob); ortaklaştırılması açık.

## Eyerli domuz ve yönetilmeyen yolcu (T44c1)

`movement/ride-pig`: 16 satır gerçek `Pig` ile bit bit eşit (havuçlu sopa elde, eyerli): düz ileri gider ve binicinin
bakışıyla döner (`getRiddenInput` hep ileri, `tickRidden`, hız `özellik·0,225`), tuşlar yok sayılır, sopasız veya başka
eşyayla ya da eyersiz domuzda binici yalnız oturur; düşüş, merdiven (domuz tırmanır), basamak/duvar, su, buz, bal, ağ.
`movement/ride-horse` eyersiz at satırıyla genişledi. Yönetilmeyen araçta (eyersiz at, sopasız domuz, teknenin ikinci
koltuğu) binici `rideTick` ile koltuğa oturur, `Rot` paketi yollar, `MoveVehicle` yollamaz ve araç sunucudan gelen
konumla ilerler (`controlled` bayrağı; koltuk noktası tür tablosundan, yolcu sırasına göre). Sapma: havuçlu sopanın
hızlandırması (`boost`, rastgele süre) istemcide yok; sopa yalnız ana ele bakılır, sol el (off hand) envanter yuvası 45
okunur ama oracle'da yalnız ana el sınandı.

## Strider (T44c2)

`movement/ride-strider`: 17 satır gerçek `Strider` ile bit bit eşit (savaş sopası yerine çarpık mantarlı sopa elde, eyerli):
düz ileri gider ve binicinin bakışıyla döner (`getRiddenInput`), soğukken (`isSuffocating`) yavaş, sıcak blokta (magma),
lavda ve lav üstünde hızlı (`getRiddenSpeed`: özellik × 0,35 / 0,55; soğukluk ayrıca hız özelliğine `-0,34` çarpan
ekler, `Strider.setSuffocating`); lavın kaynak bloğunun üstünde durur (`canStandOnFluid`, `getLiquidCollisionShape`: yarım
yüksek sütun; çarpışma, dayanak bloğu ve `shouldTravelInFluid` buna göre), lava düşünce ya da lavın içindeyken yüzeye
çıkar (`floatStrider`), lavda düşme affedilir (`checkFallDamage`). Kapsanan: kara, duvar, basamak, lav havuzu (yürüme,
girme, derin, çatı altı, kıyı), içine batma, düşme, magma zemini, su, buz, ağ; sopasız ve eyersiz. Koltuk yürüyüşle
oynar (istemci `getPassengerAttachmentPoint`). Sapma: çarpık mantarlı sopanın hızlandırması (`boost`) yok; strider bineği
`Strider`a binen strider (sıcak binek) durumu yok.

## Deve (T44c3)

`movement/ride-camel`: 24 satır gerçek `Camel` ve `CamelHusk` ile bit bit eşit. Eyerli deve binicinin bakışıyla gider
(`getRiddenInput`), koşarken (`canSprint`) hız özelliğine `+0,1` ekler; zıplama tuşu
yükü koşudur (`dash`): bakış yönünün yatay izdüşümü `22,2222 × güç × hız × blok hız çarpanı`, düşeyde `1,4285 × güç ×
zıplama momentumu` eklenir, soğukluk 55 tick ve soğukluk bitene dek koşu bonusu yok. Oturan deve
ileri itilince kalkar (`standUp`), oturma/kalkma geçişinde ve oturmuşken yerinde kalır; sudaki oturan deve hemen kalkar
(`Camel.tick`). Koltuk: oturmada gövde çapası (`getBodyAnchorAnimationOffset`) ve yolcu sırasına göre ileri/geri.
Kapsanan: düz, koşu, yön, kısa/çift/yukarı bakarak/koşarken koşu, oturan, oturuyor/kalkıyor geçişi, basamak, çit, su,
derin su, suda oturan, buz, balçık, düşme, `CamelHusk`.
Sapmalar (kayıtlı): çit sınırı ve basamak dışında entity itişi/çarpışması yok; bebek ve ölçekli deve koltuğu sınanmadı;
ses yok; ikinci yolcu koltuğu (`index != 0`) oracle'da yok; minecart yolcusu ve binme/inme akışı T44c4'e bırakıldı.
