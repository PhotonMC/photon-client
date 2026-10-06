# T40g — Eşya çerçevesi — [client-parity]

Bu kayıt `item_frame` ve `glow_item_frame` entity'lerinin (`ItemFrame`, `ItemFrameRenderer`) çizimini kapsar. Kanıt sınıfı `client-parity`'dir: çerçeve modelinin ve eşyanın yerleşimi gerçek `ItemFrameRenderer.submit`'e karşı karşılaştırılır. Konum türetimi, ışık kuralları ve çerçeve modelinin kendisi oracle'sızdır (aşağıda). Canlı gözlem `server-conformance`'tır.

## Java kaynak incelemesi

- `ItemFrameRenderer.submit`: dağıtıcı çizimi `getRenderOffset` kadar kaydırır (`(adım·0,3f, −0,25, adım·0,3f)`); `submit` önce bunu geri alır, sonra duvara doğru `0,46875` kadar çıkar. Yatay yönde `Y(180 − yön.toYRot)`, dikeyde `X(−90·adım)` ve `Y(180)`. Çerçeve modeli (görünmez değilse) `(−0,5, −0,5, −0,5)` kayması ile; eşya `Z(0,4375)` (görünmezde `0,5`) ileride, `Z(dönüş·360/8)` ve `0,5` ölçekle çizilir. Harita varsa (`mapId`) eşya yerine harita çizilir (bu kayıtta yok).
- Çerçeve modeli: `BlockStateDefinitions.getItemFrameFakeState(parlak, harita)` — adı bir bloğa bağlı olmayan, yalnız `map` özelliği olan sahte durum; `blockstates/item_frame.json` ve `glow_item_frame.json` `map=false/true` ile `block/item_frame[_map]` modellerini seçer. `BlockRenderer` bu iki dosyayı kayıt durumlarının ardına dört sahte durum olarak ekler (`item_frame_state`).
- Veri: `HangingEntity` yönü 8 almıştır; `ItemFrame` eşya (9, `ItemStack`) ve dönüş (10, `int`, sekize göre). Yön `AddEntity` verisinde (`get3DDataValue`), konum sunucunun yolladığı bloktur; entity `AABB.ofSize(blok merkezi, duvara 0,46875)`'in merkezindedir. Gözü yüksekliği 0 olduğundan ışık yoklaması entity'nin kendi konumudur.
- Işık: parlak çerçevenin blok ışığı en az 5 (`getBlockLightLevel`); parlak çerçevede eşya tam ışıkla (`15728880`) çizilir.

## Kapsam

- `entity/item-frame-pose` 18 satır (altı yön, sekiz dönüş, boş, görünmez, görünmez ve dikey, uzak konum; her satırda çerçeve ve eşya pozu). Eşya pozu, katmanın kendi `NO_TRANSFORM`'unun `(−0,5)` kaymasıyla karşılaştırılır (oracle'da katmanın başka dönüşümü yoktur).
- Rust: `voxel-render::item_frame` (`item_frame_position`, `render_offset`, `item_frame_poses`), `BlockRenderer::item_frame_state`, `voxel-world` veri (dönüş, eşya), oyunda `item_frames.rs` (eşya `Fixed` bağlamında).

## Sapmalar ve açık kalanlar

- Harita içeriği yok: istemcide harita verisi paketi ve `MapRenderer` yoktur; harita taşıyan çerçeve, haritanın verisi gelmemiş gibi (Java'nın `mapData == null` kolu) `map=false` modeliyle ve eşya olarak çizilir ([T40g2](tasks/T40g2-cerceve-haritasi.md)).
- Entity'nin kendi konumu ve vuruş kutusu hâlâ blok köşesidir (seçim/hedefleme çerçeve için doğru değil); çizim konumu blok köşesinden türetilir. Konum ve ışık kuralı (yoklama noktası, parlak çerçeve alt sınırı) kaynaktan yazıldı, oracle'sız.
- Çerçeve modelinin geometrisi oracle'sızdır (sahte durumun kendisi karşılaştırılmaz; blok model hattı başka kayıtlarla doğrulanmıştır). `entitySolidZOffsetForward` derinlik kaydırması yok; eşya adı etiketi (bakılan çerçevede özel adlı eşya) yok; ses, partikül, dış çizgi yok.
