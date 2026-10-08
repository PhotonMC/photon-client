# T24 — Entity çizim iskeleti + ilk mob

**Amaç:** `EntityRenderer`-benzeri soyutlama: ModelPart ağacı + entity dokusu + ışık ile çizim; ilk hedef mob domuz (`PigModel`) ve inek.
**Bağımlılık:** T22, T23. **Boy:** L.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/renderer/entity/{EntityRenderer,LivingEntityRenderer,AgeableMobRenderer,PigRenderer,CowRenderer,EntityRenderDispatcher}.java`, `.data/minecraft/source/net/minecraft/client/model/animal/pig/PigModel.java`, `.data/minecraft/source/net/minecraft/client/model/EntityModel.java`, `.data/minecraft/source/net/minecraft/client/model/geom/ModelPart.java` (`render`, `translateAndRotate`), `.data/minecraft/source/net/minecraft/client/renderer/entity/state/LivingEntityRenderState.java`, doku yolları (`assets/minecraft/textures/entity/pig/pig_temperate.png`; `grep -n "TEXTURE" PigRenderer.java`).
- Rust: T22 oyuncu çizim yolu (`player_renderer.rs`), T23 `photon-data` tablosu, `crates/photon-render/src/{atlas,model,vulkan,scene}.rs` (entity dokuları ayrı doku yolu mu atlas mı — `docs/atlas-layout-and-mip-upload.md` kısa bak), `entity` türü → renderer eşleme için `photon-data` entity türü kaydı (`grep -rn "pig" crates/photon-data/src/generated | head -3`).

## Yapılacaklar
1. `ModelPart` çizici: ağaç yürütme, `PartPose` + dönüş sırası (Z, Y, X) Java'daki gibi, küp → vertex (UV, mirror), gölgeleme `CardinalLighting` (yüz başına parlaklık; mevcut blok kodunu yeniden kullan), ışık (entity konumundaki blok+gökyüzü ışığı, `LightTexture`).
2. Entity renderer kaydı: tür kimliği → (model katmanı, doku, ölçek, `setupAnim`). `EntityRenderState` çıkarımı (yaş, `walkAnimation`, `hurt`) — yalnız bu mob için gerekenler.
3. Domuz: `PigModel.setupAnim` (kafa dönüşü, 4 bacak yürüme salınımı), bebek ölçeği (`AgeableMobRenderer`), eyer/derileri **yok**.
4. Çizim sırası: opak entity'ler blok geçişinden sonra, yarı saydam öncesi (Java `LevelRenderer` sırasına bak: `submitEntities`).

## Oracle
`setupAnim` eklem dönüşümleri: `entity_pose` operation'ı (Java `PigModel` + `PigRenderState` kurup `setupAnim`, sonra her parça için `xRot/yRot/zRot/x/y/z`); Rust `setupAnim` aynı sayıları üretmeli (yalnız bu domuz/inek; yeni aile açma — `entity` ailesine ek). Mantık eşitliği; görsel P15 kare karşılaştırması ayrı.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı (`shot --rcon "summon pig ~2 ~ ~"`): ekran görüntüsünde domuz; bir kez oku.

## Kapsam dışı
Diğer mob'lar (PLAN-M4), gölge, hasar kırmızısı/ölüm devrilmesi, binme.

## Belgeler
`docs/remaining-work.md` M4 madde 2–3.

## Devir notu
- Yapıldı: `EntityTextures` (tek paketlenmiş entity dokusu: 18 deri + mob dokuları, `Stitcher`; her model kendi 0..1 uv'sini dikdörtgene taşır), `Part::baked` (tablodan ModelPart), `MobModel` + `QuadrupedModel.setupAnim` (başın dönüşü, çapraz bacak salınımı), `LivingEntityRenderer` yerleşimi (`mob_model::placement_pose`), `crates/photon/src/mobs.rs` (tür → model katmanı/doku; domuz ve inek, yetişkin/bebek), `ClientEntity.baby` artık bütün hayvanlarda (indeks 16). Oracle: `quadruped_model` (gerçek `PigModel`/`CowModel.setupAnim` sonrası bütün dörtgenler; 5 katman × 7 poz) bit bit geçti. Canlı: domuz, bebek domuz, inek, bebek inek ekran görüntüsünde doğru doku ve yönle.
- Sapma: çeşit hep `temperate` (T24b), eyer/yün/boynuz katmanı yok, gölge yok, ışık entity'nin 0,5 yukarısından tek örnek (Java göz konumu), `scale` 1 (öznitelik uzak entity için okunmuyor), çizim sırası: oyuncu ve mob'lar tek entity ara belleğinde, kamera dışı ayıklama yok.
- Açık: T24b, PLAN-M4.
