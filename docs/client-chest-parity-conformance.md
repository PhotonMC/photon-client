# T79, T80 — Sandık, shulker kutusu ve çan blok entity çizimi — [client-parity]

Gerçek `ChestRenderer` ve sandık modelleri (Java) ile Rust `voxel_render::chest` aynı girdide karşılaştırılır: `entity/chest-model` (üç model, dört açıklık) ve `entity/chest-pose` (tür × yön × malzeme × açıklık; kapak, kilit ve gövdenin kaydedilen köşe, UV ve ışığı).

## Java kaynak incelemesi

`ChestRenderer.submit`: durum sandığın yönüne göre `modelTransformation` ile dikey ortası etrafında döner; açıklık `1 − (1 − açıklık)³` ile yumuşatılır ve kapağa verilir; tek, sol ve sağ yarı kendi modelini ve dokusunu seçer (`normal`, `trapped`, `ender`, dört bakır aşaması). Çift sandıkta açıklık iki yarının büyüğü, ışık `LightCoordsUtil.max`'tır. Kapak `ChestLidController`'dır: `BlockEvent` 1 ve veri > 0 iken tikte +0,1, değilse −0,1. `PoseStack.Pose.mulPose` bir dönüşün hiçbir şey döndürmediğini ilk kullanımda saptar ve o matrisi birim sayar; JOML `mulTranslationAffine`/`mulAffine` yolları bit bit aynı seçilir.

## Yapılan

- `voxel_render::chest` (tür, malzeme, doku adı, `eased_open`, `model_transformation`, `chest_pose`), `mob_model::setup_chest`, JOML özellik bayrakları ve `mulTranslationAffine`.
- `voxel_world::chest`: istemci sandık bloğu durumundan sandığı kendisi kurar (`ChestBlock`, `EnderChestBlock` sınıfları), çift sandık eşini bulur, kapak denetleyicisi `BlockEvent` ile süren tikle ilerler; `ClientboundBlockEventPacket` ayrıştırılır.
- `voxel::chests`: görünür parçalardaki sandıkları, yakın ışığı ve dokuları çizime verir.

## Sapmalar ve açık kalanlar

- Noel dokusu (`CHRISTMAS`) modelde ve oracle'da var, oyunda tarih denetimi yok: yalnız normal doku kullanılır.
- Sandık sesleri (açılış/kapanış) [T78](tasks/T78-seviye-olaylari.md)'dedir.
- Canlıda tek, çift, tuzaklı, bakır ve ender sandığı görüldü; kapak hareketi canlıda ayrıca denenmedi (kapak denetleyicisi birim testle sınandı).

## Shulker kutusu (T79b)

Gerçek `ShulkerBoxRenderer.submit` ile Rust `voxel_render::shulker_box` aynı girdide karşılaştırılır (`entity/shulker-box-pose`, 34 satır: altı yön × üç ilerleme, on altı renk): matris, ilerleme, ışık, doku ve kapağı o ilerlemeye kurulmuş modelin dörtgenleri.

- `ShulkerBoxRenderer`: kutu "yukarı"dan yönüne `Direction.getRotation` ile döner, blokdan bir tık küçüktür (`0,9995`); kapak `24 − ilerleme·8` yüksekliğe çıkar ve `270°·ilerleme` döner. Doku rengin kendisi, boyasız kutuda `entity/shulker/shulker`.
- Kapak `ShulkerBoxBlockEntity`'dir: `BlockEvent` 1 ve veri 1 açar, 0 kapatır; tikte onda bir ilerler. Yön özelliği yoksa kutu yukarı bakar.
- Sapma: kapağın açılırken yaptığı komşu güncellemeleri ve üstündeki varlıkları itmesi (`moveCollidedEntities`) istemcide yok; kapalı/açık durumdaki çarpışma şekli değişmez. Kapak hareketi canlıda denenmedi.
- Yatak ayrı bir blok entity çizicisi gerektirmez: 26.2'de normal blok modelidir (`BedRenderState` kullanılmıyor).

## Çan (T80)

Gerçek `BellRenderer.submit` ile Rust `MobModel::setup_bell` aynı girdide karşılaştırılır (`entity/bell-pose`, 36 satır: durağan ve beş yön × altı tik): ilerleme, ışık, doku ve sallanmış modelin dörtgenleri.

- `BellModel.setupAnim`: çalındığı yönün tersine `sin(tik/π)/(4 + tik/3)` kadar sallanır (kuzey/güney `x`, doğu/batı `z`); yukarı/aşağıdan çalınan çan sallanmaz. Çizici durumu `tik + kare kesri` ve çalındığı yönü (`shaking` iken) alır.
- `BellBlockEntity`: `BlockEvent` 1 yönü (`from3DDataValue`) alır, sayacı sıfırlar ve sallanmayı başlatır; her tik sayaç artar, elli tikte durur. Gövde blok modelinde yoktur, yalnız bu çizici çizer.
- Sapma: baskıncı yakındayken yankılanma (ses ve parçacık, `EntityTypeTags.RAIDERS`) yok. Canlıda yer ve tavan çanı görüldü; sallanma canlıda denenmedi.
