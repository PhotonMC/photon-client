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

## Sapmalar (açık)

- Uzaktaki oyuncunun kol pozları (`AvatarRenderer.getArmPose`: blok, yay, trident, arbalet doldurma/tutma, dürbün, boru, fırça) ve `HumanoidModel.setupAnim`'in kullanım dalı (kullanılan elin kolu önce), yolcu, süzülme (`speedValue` dahil) ve dürbünde soluk almama gerçek modele karşı karşılaştırıldı (`entity/held-item-pose`). Kullanım süresi istemcide sayılır (`ticks_using_item`); arbalet doldurma süresi hızlı doldurma büyüsünden. Pozu eşyadan çıkarma (kullanım animasyonu, yüklü arbalet) birim testli, Java'ya karşı karşılaştırılmadı.
- Mızrak (`SPEAR` pozu, `SpearAnimations`, saldırı sallaması), yerel oyuncunun kendi kullanımı (istemci izlemiyor, model tutmada kalır), piglinin silah/arbalet/hayranlık pozları (`PiglinArmPose`) ve bir yama `blocks_attacks`'ı kaldırırsa kalkan sayılması yok ([T41d2](tasks/T41d2-mizrak-piglin-yerel-kullanim.md)).
- Yerel oyuncunun ana kolu seçenek olarak okunmuyor (sağ sayılır); üçüncü şahıs
  yerel oyuncu eşyası envanterden alınır, sunucunun yankısından değil.
- Eşya dokuları blok atlasından, ışık entity konumundan alınır; kafatası modelleri
  (`SkullBlockRenderer`), pelerin, elytra ve diğer kollu/başlı modeller (köylü, illager, allay…)
  T41c ve ilgili aile görevlerinde.
