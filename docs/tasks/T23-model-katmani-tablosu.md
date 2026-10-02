# T23 — Entity model katmanı tablosu (oracle'dan)

**Amaç:** `LayerDefinitions.createRoots()` ModelPart ağaçlarını (küp boyutları, kaynak noktaları, UV ofsetleri, texture boyutu, parça adları) Java'dan yakalayıp `voxel-data` içinde üretilmiş tablo olarak tut; parity'de baştan sona karşılaştır (AGENTS.md "formülü yeniden üretemiyorsan tabloyu veri olarak al").
**Bağımlılık:** yok. **Boy:** L.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/model/geom/LayerDefinitions.java` (565; `createRoots`), `.data/minecraft/source/net/minecraft/client/model/geom/{ModelPart,PartPose,ModelLayerLocation,ModelLayers}.java`, `.data/minecraft/source/net/minecraft/client/model/geom/builders/{LayerDefinition,MeshDefinition,PartDefinition,CubeDefinition,CubeListBuilder,UVPair}.java`, `.data/minecraft/source/net/minecraft/client/model/geom/builders/MeshTransformer.java` (bebek dönüşümü).
- Kalıp: `tools/generate/` (üretici örnekleri: `ls tools/generate; sed -n 1,40p tools/generate/all.py`), `crates/voxel-data/src/generated/` (biçim kuralı: `docs/coding-standards.md` "Referans verisi"), mevcut oracle tablo örneği: blok tablosu `tests/behavior/java/BlockOracle.java` + adaptör kolu.
- Rust: `crates/voxel-render/src/player_model.rs`, `special_models.rs` (el yazımı modeller; tablo gelince birleştirilecek, bu görevde dokunma).

## Yapılacaklar
1. `tests/behavior/java/EntityModelOracle.java` (veya `EntityOracle`'a operation): `LayerDefinitions.createRoots()` → her `ModelLayerLocation` için ModelPart ağacı (ad, `PartPose` konum/dönüş, küpler: origin, boyut, `dilation`, UV, mirror; texture genişlik/yükseklik) JSON; `Oracle.vanillaRegistries()` gerekli mi kontrol et (headless sorunu varsa AGENTS.md uyarılarına bak).
2. `cases.py`: `entity_model` ailesi/operation (`entity` ailesine ek satır, ayrı aile **açma**); tek satır = tüm tablo (büyük çıktı) veya katman başına satır (tercih: katman başına, böylece fark hangi modelde belli olur).
3. Üretici: `tools/generate/entity_models.py` → `crates/voxel-data/src/generated/entity_models.rs` (okunur: katman başına bir `Layer { ... }` zinciri; **ham bayt/indeks dizisi yok**). Üretim adımı `docs/reference-data.md`'ye eklenir.
4. Rust tarafı yalnız tabloyu tipli arama ile sunar (`voxel-data`: `entity_model(layer: &str) -> &Layer`); adaptör aynı tabloyu JSON'a çevirip karşılaştırır.

## Oracle
Bu görevin kendisi oracle: capture + tüm parity karşılaştırması.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`; `python3 tools/generate/all.py` yeniden üretimi aynı dosyayı verir.

## Kapsam dışı
Modelleri çizmek (T24), animasyon (`KeyframeAnimation`), zırh/eşya katmanları dışındaki katmanlar da tabloda **tam** (tüm roots).

## Belgeler
`docs/reference-data.md` (üretim adımı), `docs/coding-standards.md` gerekmez.

## Devir notu
- Yapıldı: `EntityOracle` `model_layers` (gerçek `LayerDefinitions.createRoots`, tanım düzeyinde: poz, küp başına origin/boyut/büyütme/ayna/uv/uv ölçeği/yüzler, çocuklar ada göre sıralı), 358 katman · 3359 parça · 2494 küp; `tools/generate/entity_models.py` → `voxel-data/src/generated/entity_models.rs` (484 KB, `ModelLayer::of` zinciri), tipler ve `layer(id)`/`layers()` `voxel-data/src/entity_model.rs`; adaptör aynı tabloyu JSON'a çevirip karşılaştırır (parity geçti).
- Java'dan sapma: yok; Java bunu `HashMap` sırasıyla verir, tabloda çocuklar ada göre sıralı (sıra sözleşmenin parçası değil).
- Üretim adımı `docs/reference-data.md` "Üretilen tablolar". `approx_constant` Clippy uyarısı dosya başında gerekçeyle kapalı (Java'nın `1.5708F` yazımı).
- Açık: tabloyu çizim için bakılmış `ModelPart`'a çevirmek (T24); `player_model.rs`/`special_models.rs` el yazımı olarak duruyor.
