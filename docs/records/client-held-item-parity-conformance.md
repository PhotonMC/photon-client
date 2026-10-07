# T41 — Elde eşya ve kafa eşyası: `SetEquipment`, `ItemInHandLayer`, `CustomHeadLayer` — [client-parity]

Bu kayıt, insansı canlıların (oyuncu, zombi, iskelet, piglin ve aileleri,
yetişkin ve bebek) ellerinde tuttuğu eşyanın Java 26.2 ile aynı yere çizilmesini
ve tutmanın kollara verdiği pozu kapsar. Kanıt sınıfı `client-parity`'dir:
gerçek model sınıfları gerçek `setupAnim` ile pozlanır, gerçek
`ItemInHandLayer`/`PlayerItemInHandLayer` her eli bir `submitItem` çağrısına
çevirir ve eşyanın o andaki poz matrisi ile modelin her dörtgeni bit bit
karşılaştırılır. `SetEquipment` paketinin ayrıştırılması `server-conformance`
(birim test) kanıtıdır; canlı gözlem (zombi, iskelet, piglin, oyuncu) ikincil.

## Java kaynak incelemesi

- `ItemInHandLayer.submitArmWithItem`: eli `translateToHand` ile bulur, x ekseninde
  −90° ve y ekseninde 180° döndürür, sonra `(±1, 2, −10)/16` öteler (bebekte
  `(0, 1, −4.5)/16`; zırh askısında bebek ofseti yok). Eşyanın kendi görüntü
  dönüşümü bundan sonra uygulanır.
- `HumanoidModel.translateToHand`: kök, sonra kol. `PlayerModel` ince kolda eli
  yarım birim dışarı alır; `SkeletonModel` bir birim dışarı alır.
- `HumanoidModel.setupAnim`: kol pozları (`EMPTY`, `ITEM`, `BOW_AND_ARROW`…) kolları
  sağ/sol el ve iki elli sırasına göre pozlar. Oyuncuda dolu el `ITEM`
  (`AvatarRenderer.getArmPose`); mobda varsayılan `EMPTY`, iskelette ana kolla
  yay çeken saldırgan iskelet `BOW_AND_ARROW` (`AbstractSkeletonRenderer`).
- `ArmedEntityRenderState`: sağ el `getItemHeldByArm(RIGHT)`, `THIRD_PERSON_RIGHT_HAND`
  bağlamıyla; ana kol oyuncuda `Avatar.DATA_PLAYER_MAIN_HAND`, mobda sol elli bayrağı.

- `CustomHeadLayer.submit`: yatay ölçek (`Transforms.horizontalScale`), kök, kafa parçası,
  `translate(0, −0,25 + yOffset, 0)`, y'de 180°, `scale(0,625, −0,625, −0,625)`; eşya `HEAD`
  görüntü bağlamıyla çizilir. Zırh olarak çizilen kafa parçası ve kafatasları (`AbstractSkullBlock`,
  kafatası modeli) bu yoldan geçmez. Dönüşümler: piglin (0, 0, 1,0019531), köylü
  (−0,1171875, −0,07421875, 1), ötekiler varsayılan.

## Veri

`ClientboundSetEquipmentPacket` (kimlik 102): varint entity, her yuva için bayt (üst bit
"devam"), eşya yığını; `EquipmentSlot` sırası ana el, yan el, ayak, bacak, göğüs,
baş, gövde, eyer. Yığınlar `ClientEntity.equipment` içinde. `Avatar.DATA_PLAYER_MAIN_HAND`
(15, `HUMANOID_ARM` serileştiricisi 42) veri indeksi `entity/variant-fields` satırıyla her
koşuda Java'nın erişimcisinden karşılaştırılır.

## Kapsam ve dal kararı

`entity/held-item-pose` 65 satır (42 el + 23 kafa eşyası satırı: oyuncu, ince oyuncu, zombi, iskelet, piglin, bebekler, bakış, çömelme, köylü dönüşümü): oyuncu (geniş ve ince) için on bir poz (boş, sağ/sol
eşya, iki el, bakış, çömelme, sallama, yüzme, solak, solak yay çekme, yan elde iki elli
poz); zombi ve piglin yetişkin/bebek için üçer poz; zombi saldırgan ve solak; iskelet
için yay, yaylı saldırgan (sağ ve solak), kılıçlı saldırgan. Aile içi türler tablo
satırıdır; eşyanın görüntü dönüşümü ayrıca `model/display-*` ailesiyle kapalıdır.

## Kafatası ve gözde dürbün (T41f4)

`entity/held-item-pose` 64 satır kafadaki kafatası (yedi tür, piglinin kulakları ve ejderin çenesi yürüyüşle, bakış, çömelme, mob dönüşümleri, köylü dönüşümü) gerçek `CustomHeadLayer` ve gerçek kafatası modelinin dörtgenlerine karşı; 20 satır gözde dürbün (iki oyuncu yapısı, iki ana kol, iki el, dört bakış; salladığında, kullanılmıyorken ve çömelirken) gerçek `PlayerItemInHandLayer`'ın poz matrisine karşı: bit bit eşit. Oyunda `Hands::skull`/`Armor::skull_vertices` kafatasını mobun/oyuncunun/zırh askısının dokusuyla (varsayılan oyuncu derisi) çizer, `Hands::eye` kullanılan dürbünü göze taşır. Açık: profili olan oyuncu kafası (PLAN-M9), köylü/cadı kafa dönüşümünün kafatası ofseti (yalnız varsayılan sıfır), yerel oyuncunun dürbünü (kullanımı izlenmiyor).

## Mızrak (T41d3)

`entity/held-item-pose` 138 satır: gerçek `HumanoidModel` (`SPEAR` kol pozu, `SpearAnimations.thirdPersonHandUse` ve `thirdPersonAttackHand`) ile gerçek `ItemInHandLayer` (`thirdPersonUseItem`, `thirdPersonAttackItem`) gerçek mızrak eşya yığınlarıyla: yedi mızrağın kendi süreleriyle sekiz kullanım anı (kaldırma, sallanma, indirme, geri kaldırma), her iki el ve ana kol, yüzme/süzülme/yolcu/çömelme, vuruş geri tepmesi, iki elde mızrak, saldırı sallamasında dürtme (oyuncu, zombi, bebek piglin; yedi hız; iki kol). `entity/spear-table`: gerçek kayıt defterinden mızrak olan her eşyanın sallanma süresi, `KineticWeapon` gecikmesi ve üç koşulun süreleri, ileri hareket (`voxel-data::spear`). Hepsi bit bit eşit. Aynı koşuda `ZombieModel`'in `animateZombieArms`'ı da düzeldi: dürtmede kollar oynatılmaz, bebek zombi elinde eşya varken kollarını kaldırmaz (`main_hand_empty`); ve `Matrix4f.rotateAroundAffine` ötelemesi için gerçek yazım sırası (döndürülmüş üçüncü sütun) taklit edilir.

Oyunda: uzaktaki oyuncu ve insansı moblar (`arm_poses::{spear_view, mob_spear_pose}`): `getArmPose` (kinetik silahlı eşya kullanılırken `SPEAR`, mızrak tutarken `SPEAR`, zombide öteki elin mızrağı da), `ItemInHandLayer`'ın dürtme ve kullanım dönüşleri, `EntityEvent` 2 ile vuruş geri tepmesi (`ClientEntity::ticks_since_kinetic_hit`), ve sallama süresi elde tutulan mızrağın `SwingAnimation.duration`'ı (`ClientEntity::swing_durations`; oyuncular ve canavarlar). Çıkarım (`getArmPose`, hangi elin ne tuttuğu) Java'ya karşı karşılaştırılmadı; `spears` eşya etiketi yedi mızrak sayılır.

Açık: uzaktaki varlıkların hız/yorgunluk etkileri (okunmuyor), vuruş sesi (`makeLocalHitSound`), `SwingAnimationType.NONE`, bir yamanın `KINETIC_WEAPON`/`SWING_ANIMATION` bileşenini değiştirmesi (yalnız varsayılan bileşenler).

## Yerel oyuncunun sallaması ve kullanımı (T41d4)

- `LivingEntity.getCurrentSwingDuration`: sallayan elin eşyasının süresi (mızrakta kendi süresi, yoksa altı), hız (`haste`) ya da kanal gücü (`conduit_power`) varsa `süre − (1 + en yüksek seviye)`, yoksa kazı yorgunluğu varsa `süre + (1 + seviye)·2`. `swing(el)` yarısı geçmemiş bir sallamayı yeniden başlatmaz (yarı bu süreden); `updateSwingTime` ve saldırı animasyonu bu süreyle işler; sallayan kol (`swingingArm`) sallama bitince de kalır. Yerel oyuncunun sallaması artık el seçer: ikinci el eylem yapınca (`ServerboundSwing` zaten gidiyordu) sol kol sallanır.
- Kullanım: sunucunun canlılık bayrakları (kullanılıyor, ikinci el) ve sayaç (`getTicksUsingItem`, kullanıldığı her tikte bir artar). Üçüncü şahıs modeli kollarını bununla uzak oyuncularla aynı kodla (`arm_state_of`) kurar: yay, kalkan, arbalet, mızrak, dürbün, boru, fırça ve sallayan kol.
- Kapsam: `movement/entity-aim` ve öteki yerel oyuncu satırlarının her tik çıktısına sallama (`swinging`, `swingTime`, ikinci el, saldırı animasyonu) ve kullanım (kullanılıyor, el, sayaç) eklendi; satırlar gerçek `LocalPlayer`'a karşı: hız, kanal gücü ve hız birlikte, yorgunluk, ikinci elle koyma ve hızla ikinci el kullanımı, mevcut saldırı ve yerleştirme satırları.
- Açık: istemci kullanımı kendi başlatmaz (sunucunun bayrağı beklenir; `Item.use`'un ön tahmini yok); kullanırken yavaşlama (`isSlowDueToUsingItem`, `useEffects`) ve koşma kesilmesi; yerel mızrağın saldırı/dürtme akışı (oracle'da saldırı tuşu mızrakla paket göndermez) ve vuruş sesi; uzaktaki varlıkların etkileri; yerel mızrak sallaması oracle'sız (aynı `spear_of` yolu).

## Oyuncunun gövde dönüşü (T41f3)

`entity/avatar-pose` 26 satır: gerçek `AvatarRenderer.setupRotations` ile `LivingEntityRenderer.submit`'in sonu (ayaklar, çömelme kayması, gövde dönüşü, ölüm yatışı, riptide dönüşü, yüzme ve sürünme eğimi, süzülme eğimi ve yönü) gerçek yığında; model matrisi bit bit eşit. Süzülmede eğim `fallFlyingScale = clamp(t²/100)` ile büyür, süzülme yönü (`flyingYRot`) hareketin ve bakışın yatay yönünden. Oyunda `ClientEntity::{fall_flying_ticks, previous_motion}` sayılır, `arm_poses::fall_flying` durumu kurar; uzaktaki oyuncuda çizilir (yerel oyuncu süzülmez). Süzülme durumunun çıkarımı (kare, hareket, bakış) Java'ya karşı karşılaştırılmadı.

## Sapmalar (açık)

- Uzaktaki oyuncunun kol pozları (`AvatarRenderer.getArmPose`: blok, yay, trident, arbalet doldurma/tutma, dürbün, boru, fırça) ve `HumanoidModel.setupAnim`'in kullanım dalı (kullanılan elin kolu önce), yolcu, süzülme (`speedValue` dahil) ve dürbünde soluk almama gerçek modele karşı karşılaştırıldı (`entity/held-item-pose`). Kullanım süresi istemcide sayılır (`ticks_using_item`); arbalet doldurma süresi hızlı doldurma büyüsünden. Pozu eşyadan çıkarma (kullanım animasyonu, yüklü arbalet) birim testli, Java'ya karşı karşılaştırılmadı.
- Piglinin kol pozları (`PiglinArmPose`: silah yukarıda ve aşağı sallama, arbalet tutma/doldurma, hayranlık, dans) `entity/piglin-model`'de gerçek modele karşı karşılaştırıldı; pozun çıkarımı (`Piglin.getArmPose`) karşılaştırılmadı.
- Bir yama `blocks_attacks`'ı kaldırırsa kalkan sayılması yok ([T41d4](../tasks/T41d4-yerel-kullanim.md)).
- Yerel oyuncunun ana kolu seçenek olarak okunmuyor (sağ sayılır); üçüncü şahıs
  yerel oyuncu eşyası envanterden alınır, sunucunun yankısından değil.
- Eşya dokuları blok atlasından, ışık entity konumundan alınır; kafatası modelleri
  (`SkullBlockRenderer`), pelerin, elytra ve diğer kollu/başlı modeller (köylü, illager, allay…)
  T41c ve ilgili aile görevlerinde.
