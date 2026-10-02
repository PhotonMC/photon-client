# T41 — Elde eşya: `SetEquipment` ve `ItemInHandLayer` — [client-parity]

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

## Veri

`ClientboundSetEquipmentPacket` (kimlik 102): varint entity, her yuva için bayt (üst bit
"devam"), eşya yığını; `EquipmentSlot` sırası ana el, yan el, ayak, bacak, göğüs,
baş, gövde, eyer. Yığınlar `ClientEntity.equipment` içinde. `Avatar.DATA_PLAYER_MAIN_HAND`
(15, `HUMANOID_ARM` serileştiricisi 42) veri indeksi `entity/variant-fields` satırıyla her
koşuda Java'nın erişimcisinden karşılaştırılır.

## Kapsam ve dal kararı

`entity/held-item-pose` 42 satır: oyuncu (geniş ve ince) için on bir poz (boş, sağ/sol
eşya, iki el, bakış, çömelme, sallama, yüzme, solak, solak yay çekme, yan elde iki elli
poz); zombi ve piglin yetişkin/bebek için üçer poz; zombi saldırgan ve solak; iskelet
için yay, yaylı saldırgan (sağ ve solak), kılıçlı saldırgan. Aile içi türler tablo
satırıdır; eşyanın görüntü dönüşümü ayrıca `model/display-*` ailesiyle kapalıdır.

## Sapmalar (açık)

- Kullanım pozları yok (`isUsingItem`: blok, yay çekme, arbalet, mızrak, dürbün,
  boru, fırça) ve `ItemInHandLayer`'ın mızrak sallaması/kullanımı
  (`SpearAnimations`, `animateUseItem`): kullanım durumu okunmuyor.
- Piglinin silah/arbalet/hayranlık pozları (`PiglinArmPose`), mızrak etiketli eşyalarda
  `SPEAR` pozu yok; mobda `EMPTY` kalır. Saldırı kolu/sallaması `Animate` ile (T43).
- Yerel oyuncunun ana kolu seçenek olarak okunmuyor (sağ sayılır); üçüncü şahıs
  yerel oyuncu eşyası envanterden alınır, sunucunun yankısından değil.
- Eşya dokuları blok atlasından, ışık entity konumundan alınır; kafa eşyası, zırh,
  pelerin, elytra ve armor stand/diğer kollu modeller (köylü, illager, allay…) T41b/T41c
  ve ilgili aile görevlerinde.
