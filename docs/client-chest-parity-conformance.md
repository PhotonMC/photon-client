# T79 — Sandık blok entity çizimi — [client-parity]

Gerçek `ChestRenderer` ve sandık modelleri (Java) ile Rust `voxel_render::chest` aynı girdide karşılaştırılır: `entity/chest-model` (üç model, dört açıklık) ve `entity/chest-pose` (tür × yön × malzeme × açıklık; kapak, kilit ve gövdenin kaydedilen köşe, UV ve ışığı).

## Java kaynak incelemesi

`ChestRenderer.submit`: durum sandığın yönüne göre `modelTransformation` ile dikey ortası etrafında döner; açıklık `1 − (1 − açıklık)³` ile yumuşatılır ve kapağa verilir; tek, sol ve sağ yarı kendi modelini ve dokusunu seçer (`normal`, `trapped`, `ender`, dört bakır aşaması). Çift sandıkta açıklık iki yarının büyüğü, ışık `LightCoordsUtil.max`'tır. Kapak `ChestLidController`'dır: `BlockEvent` 1 ve veri > 0 iken tikte +0,1, değilse −0,1. `PoseStack.Pose.mulPose` bir dönüşün hiçbir şey döndürmediğini ilk kullanımda saptar ve o matrisi birim sayar; JOML `mulTranslationAffine`/`mulAffine` yolları bit bit aynı seçilir.

## Yapılan

- `voxel_render::chest` (tür, malzeme, doku adı, `eased_open`, `model_transformation`, `chest_pose`), `mob_model::setup_chest`, JOML özellik bayrakları ve `mulTranslationAffine`.
- `voxel_world::chest`: istemci sandık bloğu durumundan sandığı kendisi kurar (`ChestBlock`, `EnderChestBlock` sınıfları), çift sandık eşini bulur, kapak denetleyicisi `BlockEvent` ile süren tikle ilerler; `ClientboundBlockEventPacket` ayrıştırılır.
- `voxel::chests`: görünür parçalardaki sandıkları, yakın ışığı ve dokuları çizime verir.

## Sapmalar ve açık kalanlar

- Noel dokusu (`CHRISTMAS`) modelde ve oracle'da var, oyunda tarih denetimi yok: yalnız normal doku kullanılır.
- Sandık sesleri (açılış/kapanış) [T78](tasks/T78-seviye-olaylari.md)'dedir; shulker kutusu ve yatak çizimi bu görevin kalanı olan [T79b](tasks/T79b-shulker-yatak.md)'dedir.
- Canlıda tek, çift, tuzaklı, bakır ve ender sandığı görüldü; kapak hareketi canlıda ayrıca denenmedi (kapak denetleyicisi birim testle sınandı).
