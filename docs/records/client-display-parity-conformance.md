# T40i, T40i2, T40i3 — Display entity'leri — [client-parity]

Bu kayıt `block_display` ve `item_display` entity'lerinin (`Display`, `DisplayRenderer`) istemci durumunu ve çizim yerleşimini kapsar. Kanıt sınıfı `client-parity`'dir: veri, render durumu ve aradeğerleme gerçek `Display` varlıklarının gerçek istemci dinleyicisiyle tiklenmesine, yerleşim gerçek `DisplayRenderer.submit`'e karşı karşılaştırılır. Işık geçersiz kılma, görüş menzili ve çizim entegrasyonu oracle'sızdır (aşağıda). Canlı gözlem `server-conformance`'tır. Metin display'i, gölge, çerçeve ve parıltı dışı bırakıldı.

## Java kaynak incelemesi

- Veri (`Entity` sekiz alır): 8 başlangıç gecikmesi, 9 dönüşüm aradeğerleme süresi, 10 konum/dönüş süresi, 11 öteleme, 12 ölçek (vektör), 13 sol ve 14 sağ dönüş (dördey), 15 billboard, 16 parlaklık, 17 görüş menzili, 18 gölge yarıçapı, 19 gölge gücü, 20 genişlik, 21 yükseklik, 22 parıltı rengi; block display 23 blok durumu; item display 23 eşya, 24 `ItemDisplayContext` (bayt). Paketteki her alan `onSyncedDataUpdated` çağırır (değişmese de).
- `Display.tick` (istemci): bekleyen notlar tikte işlenir: başlangıç tiki `tickCount + gecikme`, süre güncellenir, render durumu yenilenir; süre sıfır değilse ve bir önceki durum varsa yeni durum son ilerlemeden `Transformation.slerp` ile (öteleme ve ölçek doğrusal, dönüşler küresel) aradeğerlenir, yoksa sabit durum kurulur. Kısmi tik sorulunca ilerleme `(tickCount − başlangıç + kısmi) / süre` (0–1) ve son ilerleme olarak hatırlanır. Hiç veri gelmemişse durum yoktur (çizilmez).
- Konum ve dönüş aradeğerlemesi `InterpolationHandler(entity, 0)`: uzunluk konum/dönüş süresidir; sıfırsa `snapTo`. Bu görevde aradeğerleme uzunluğu entity başına parametrikleşti (öteki varlıklarda üç).
- `DisplayRenderer.submit`: yönelim billboard'a göre `rotationYXZ(−yaw, pitch, 0)`: sabit entity'nin, dikey yaw kameranın (−180), yatay pitch kameranın (eksi), merkez ikisi de kameranın; sonra dönüşümün matrisi (`öteleme·sol·ölçek·sağ`) `mulPose` ile; block display modeli, item display `Y(π)` sonra eşyayı çizer. `mulPose(Matrix4fc)` normal matrisini matris dik-birimse çarparak, değilse tersinin devriği olarak yeniden hesaplar.
- Işık: `brightnessOverride` ≠ −1 ise blok ve gökyüzü ışığı (paketli parlaklıktan) dünyanınkinin yerine geçer; ışık yoklaması entity'nin konumudur (göz yüksekliği 0).

## Kapsam

- `movement/living-entity-animations`'a 6 satır (sessiz, tam veri, aradeğerleme ve araya giren güncelleme ve gecikme, ters yönde dönüş, konum/dönüş süresi, item display bağlamı; her tikte konum/dönüş, durum, blok/eşya ve iki kısmi tikte ilerleme, matris, gölge).
- `entity/display-pose` 11 satır (dört billboard, tek biçimli ve ezilmiş ve aynalı ölçek, item; her satırda pozun matrisi ve normal matrisi).
- Rust: `photon-world::display` (veri, durum, aradeğerleyiciler), `photon-render::display` (matris, yönelim, poz), `Pose::mul_matrix` genelleşti (normal matrisini yeniden hesaplar, `Matrix3::inverse_transpose`), `DisplayContext::None`, oyunda `displays.rs`.

## Gölge, görüş menzili ve ışık (T40i3)

- `Display.shouldRenderAtSqrDistance`: `mesafe² < (görüşMenzili·64·viewScale)²` (`viewScale` seçenek, bir). Çizilmeyen display gölge de vermez.
- Gölge: `DisplayRenderer.getShadowRadius/Strength` render durumunun yarıçap ve gücü, ilerlemede aradeğerlenmiş; yarıçap sıfır veya entity görünmezse gölge yoktur; gerisini mevcut gölge hattı (`extractShadow`) yürütür.
- Işık geçersiz kılma: parlaklık −1 değilse blok ve gökyüzü seviyeleri (`>> 4 & 15`, `>> 20 & 15`) yeniden paketlenir.
- Kapsam: `entity/display-light` 6 satır (gerçek `DisplayRenderer.getBlockLightLevel/getSkyLightLevel`: ikisi birden, karanlık, yalnız blok, yalnız gökyüzü, başıboş bitler, eksi); `living-entity-animations` display satırlarına her tikte altı mesafede `shouldRenderAtSqrDistance`.
- Frustum ayıklaması (`affectedByCulling`, genişlik/yükseklik) yapılmadı: istemcide hiçbir varlık için frustum ayıklaması yoktur, yalnız çizilen şeyi azaltırdı. Parıltı rengi ve dış çizgi kapsam dışı.

## Metin display'i (T40i2)

- Veri (item display'in ardından): 23 metin (bileşen, NBT), 24 satır genişliği, 25 arka plan rengi, 26 metin opaklığı (bayt), 27 biçim bayrakları (bayt; 1 gölge, 2 içinden, 4 varsayılan arka plan, 8 sola, 16 sağa). Herhangi birinin değişmesi render durumunu yeniden yapar; süre sıfır değilse opaklık tam sayıda (`lerpInt`), arka plan kanal kanal (`ARGB.srgbLerp`) son ilerlemeden yenisine kayar.
- `TextDisplayRenderer.submitInner`: pose üzerinde `rotate(π, y)`, `scale(−0,025)` ve `translate(1 − genişlik/2, −yükseklik)` (`yükseklik = satır·10 − 1`, genişlik en geniş satır); arka plan (rengi sıfır değilse) `(−1, −1, −0,01)`'den `(genişlik, yükseklik, −0,01)`'e kutu; satırlar `10` aralıkla, hizaya göre (sol 0, sağ `genişlik − satır`, orta `genişlik/2 − satır/2`); metin rengi `opaklık << 24 | 0xFFFFFF`; varsayılan arka plan `(int)(0,25·255) << 24`; içinden bayrağı arka planı ve metni içinden çizer.
- Kapsam: `entity/text-display-submit` 12 satır gerçek `TextDisplayRenderer.submit`'e karşı (pose matrisi, arka plan köşeleri ve rengi ve ışığı, her satırın yeri, gölge, kip, ışık, renk, çizim sırası: bir satır, üç satır ortalı/sola/sağa, iki hiza bayrağı, gölge ve içinden, varsayılan ve boş arka plan, yarım ve soluk opaklık, satırsız, uzak konum); `living-entity-animations`'a 2 satır gerçek `TextDisplay` varlığına karşı (veri, süre boyunca opaklık ve arka plan).
- Oyunda `displays::display_texts`: bileşen dilden sözcüklere (`Lang::runs`), `Font.split` ile satırlara, pozun köşeleri ve glifler isim etiketinin metin hatlarına (içinden / derinlikli) eklenir.
- Açık: satır bölme ve genişlikler oracle'sız (oyunun yazı tipiyle; satırlar oracle'da genişlikleriyle verilir), `getBackgroundOpacity` seçeneği varsayılan (0,25), kayan satır önbelleği yok (her karede yeniden bölünür), `POLYGON_OFFSET` kipinin derinlik kayması yok.

## Sapmalar ve açık kalanlar

- Oyun içi çizim (`displays.rs`, gölge dahil: `display_shadow`) ve gölgenin canlıda görülmesi oracle'sız (arazide gölge düşecek zemin bulunamadı; hat mob gölgeleriyle ortaktır). Çizim konumu entity'nin kısmi tikli konumudur.
- Dönüşümlü sağ/sol el bağlamları item display'inde `ItemLayerCache` başına kuruldur.
- `entitySolidZOffsetForward` yok.
