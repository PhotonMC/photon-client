# T40g, T40g2 — Eşya çerçevesi ve haritası — [client-parity]

Bu kayıt `item_frame` ve `glow_item_frame` entity'lerinin (`ItemFrame`, `ItemFrameRenderer`) çizimini kapsar. Kanıt sınıfı `client-parity`'dir: çerçeve modelinin ve eşyanın yerleşimi gerçek `ItemFrameRenderer.submit`'e karşı karşılaştırılır. Konum türetimi, ışık kuralları ve çerçeve modelinin kendisi oracle'sızdır (aşağıda). Canlı gözlem `server-conformance`'tır.

## Java kaynak incelemesi

- `ItemFrameRenderer.submit`: dağıtıcı çizimi `getRenderOffset` kadar kaydırır (`(adım·0,3f, −0,25, adım·0,3f)`); `submit` önce bunu geri alır, sonra duvara doğru `0,46875` kadar çıkar. Yatay yönde `Y(180 − yön.toYRot)`, dikeyde `X(−90·adım)` ve `Y(180)`. Çerçeve modeli (görünmez değilse) `(−0,5, −0,5, −0,5)` kayması ile; eşya `Z(0,4375)` (görünmezde `0,5`) ileride, `Z(dönüş·360/8)` ve `0,5` ölçekle çizilir. Harita varsa (`mapId`) eşya yerine harita çizilir (bu kayıtta yok).
- Çerçeve modeli: `BlockStateDefinitions.getItemFrameFakeState(parlak, harita)` — adı bir bloğa bağlı olmayan, yalnız `map` özelliği olan sahte durum; `blockstates/item_frame.json` ve `glow_item_frame.json` `map=false/true` ile `block/item_frame[_map]` modellerini seçer. `BlockRenderer` bu iki dosyayı kayıt durumlarının ardına dört sahte durum olarak ekler (`item_frame_state`).
- Veri: `HangingEntity` yönü 8 almıştır; `ItemFrame` eşya (9, `ItemStack`) ve dönüş (10, `int`, sekize göre). Yön `AddEntity` verisinde (`get3DDataValue`), konum sunucunun yolladığı bloktur; entity `AABB.ofSize(blok merkezi, duvara 0,46875)`'in merkezindedir. Gözü yüksekliği 0 olduğundan ışık yoklaması entity'nin kendi konumudur.
- Işık: parlak çerçevenin blok ışığı en az 5 (`getBlockLightLevel`); parlak çerçevede eşya tam ışıkla (`15728880`) çizilir.

## Kapsam

- `entity/item-frame-pose` 18 satır (altı yön, sekiz dönüş, boş, görünmez, görünmez ve dikey, uzak konum; her satırda çerçeve ve eşya pozu). Eşya pozu, katmanın kendi `NO_TRANSFORM`'unun `(−0,5)` kaymasıyla karşılaştırılır (oracle'da katmanın başka dönüşümü yoktur).
- Rust: `photon-render::item_frame` (`item_frame_position`, `render_offset`, `item_frame_poses`), `BlockRenderer::item_frame_state`, `photon-world` veri (dönüş, eşya), oyunda `item_frames.rs` (eşya `Fixed` bağlamında).

## Çerçevedeki harita (T40g2)

- `ClientboundMapItemDataPacket` (kimlik 51): harita kimliği (var int), ölçek (bayt), kilit (bool), süsler (bool, sayı, her biri: tür numarası, `x`, `y`, `rot & 15`, isteğe bağlı ad bileşeni) ve renk yaması (genişlik sıfırsa yok; yükseklik, `başX`, `başY`, renk dizisi). İstemci haritayı ilk pakette yapar (renkler boş), süsleri paketteki ile değiştirir (`addClientSideDecorations`), yama renkleri yazar; sürüm sayacı her pakette artar. Dünya yenilenince (boyut değişimi) haritalar silinir.
- `MapColor.getColorFromPackedId`: 256 paketli rengin ARGB'si ve `MapDecorationType` kayıt tablosu (ad, sprite, çerçevede gösterilir mi, harita rengi, sayılır mı) oracle'dan alınır (`entity/map-data`, `tools/generate/map_data.py`) ve her koşuda karşılaştırılır.
- `ItemFrameRenderer`: eşya `map_id` bileşeni taşıyorsa ve haritanın verisi gelmişse harita dalı: çerçeve modeli `map=true`, poz `Z(dönüş % 4 · 2 sekizde)`, `Z(180)`, `0,0078125` ölçek, `(−64, −64, 0)` ve `(0, 0, −1)` ötelemesi; ışık parlak çerçevede `15728850`. `MapRenderer.render`: `(0..128)` kare `z = −0,01`, sonra her süs (çerçevede yalnız `showOnItemFrame` olanlar): `(x/2 + 64, y/2 + 64, −0,02)` ötele, `Z(rot·360/16)`, `(4, 4, 3)` ölçek, `(−0,125, 0,125, 0)` ötele, kare `z = sıra · −0,001`.
- Kapsam: `entity/item-frame-pose`'a 7 harita satırı (dönüşler, parlak ve dikey, güney; poz ve haritanın köşeleri, doku koordinatları ve ışığı), `entity/map-render` 5 satır gerçek `MapRenderer.render`'a karşı (süssüz, süslü, yalnız çerçevede olanlar, hiçbiri, uzak), `entity/map-data` tablosu; paket ayrıştırıcı için birim testi.
- Oyunda `item_frames::MapTextures`: haritalar 128 satırlık yuvalara (en çok 16) yazılan tek görüntüde (`VulkanBackend::set_map_texture`, tanımlayıcı küme aynı kalıp yeni görüntüye çevrilir), süs sprite'ları entity dokularına paketlenir; haritalar metin hattıyla, süsler yarı saydam metin hattıyla çizilir. Canlı: `filled_map` taşıyan iki çerçevede (biri parlak) harita görüldü.
- Açık: süs adları (`decoration.name`: yazı ölçeği ve konumu) çizilmez ve oracle'sızdır; 16'dan çok harita aynı anda görünürse yuvalar sırayla yeniden kullanılır; eldeki/envanterdeki harita çizimi ayrı iştir.

## Sapmalar ve açık kalanlar

- Verisi gelmemiş haritanın çerçevesi Java'nın `mapData == null` koluyla `map=false` modeliyle ve harita eşyası olarak çizilir.
- Entity'nin kendi konumu ve vuruş kutusu hâlâ blok köşesidir (seçim/hedefleme çerçeve için doğru değil); çizim konumu blok köşesinden türetilir. Konum ve ışık kuralı (yoklama noktası, parlak çerçeve alt sınırı) kaynaktan yazıldı, oracle'sız.
- Çerçeve modelinin geometrisi oracle'sızdır (sahte durumun kendisi karşılaştırılmaz; blok model hattı başka kayıtlarla doğrulanmıştır). `entitySolidZOffsetForward` derinlik kaydırması yok; eşya adı etiketi (bakılan çerçevede özel adlı eşya) yok; ses, partikül, dış çizgi yok.
