# T42 — Entity gölgesi (`submitShadow`)

**Amaç:** Canlıların ve bazı entity'lerin altına yarı saydam yuvarlak gölge çiz (`EntityRenderDispatcher` gölge parçaları: `shadowRadius`, `shadowStrength`, zemin bloğunun ışığı ve mesafe solması).
**Bağımlılık:** T24. **Boy:** M. **Türler:** tüm çizilen entity'ler (`shadowRadius > 0`).

## Oku
- Java: `client/renderer/entity/EntityRenderDispatcher.java`, `client/renderer/entity/EntityRenderer.java`.
- Rust: `crates/voxel/src/mobs.rs`, `overlays.rs` (çizim listeleri), `crates/voxel-render/src/vulkan.rs` (`TextTranslucent` boru hattı, T25b), `crates/voxel-render/src/objects/xp_orb.rs` (yarı saydam dörtgen kalıbı), `crates/voxel-world/src/collision.rs` (zemin taraması için blok kutuları).

## Yapılacaklar
1. `EntityRenderDispatcher.extractShadowPieces` kuralları: yarıçap tablosu (tür başına `shadowRadius`, bebek ölçeği), zemin taraması (en çok kaç blok aşağı), kutu kesişimi ve alfa.
2. Çizim: kare dörtgenler `shadow` dokusuyla (`textures/misc/shadow.png`), zemin yüzeyine oturan, yarı saydam (T25b'nin `TextTranslucent` boru hattı kalıbı).
3. Oracle: gerçek `ClientLevel` sahnesinde gölge parçaları (konum/alfa/uv) — `ShadowPiece` kayıtları.

## Oracle
`EntityOracle.java`'da `experienceOrb`/`itemEntityPose` kalıbı: gerçek `EntityRenderDispatcher` gölge çıkarımı gerçek `ClientLevel` sahnesiyle (`MovementOracle.buildScene`) koşturulur; her `ShadowPiece` (konum, boyut, alfa, uv) bit bit karşılaştırılır.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Blok gölgesi/ışık haritası (var), bulut gölgesi.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Bağımsız ve her çizilen mobu görsel olarak iyileştirir; erken yapılabilir. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı: `voxel-world/shadow.rs` (`extractShadow`: parça seçimi, ışık, alfa), `voxel-render/shadow.rs` (dörtgen köşeleri ve doku sınırına kırpma), `PipelineKind::Shadow` (ışıksız, yarı saydam, derinliğe yazmayan), `overlays::entity_shadows` (domuz/inek/oyuncu/eşya/küre yarıçapları, üçüncü şahıs yerel oyuncu). Kayıt: `docs/records/client-entity-shadow-parity-conformance.md`.
- Oracle: `movement/entity-shadows` 51 satır (gerçek ClientLevel + gerçek ışık motoru + gerçek `extractShadow`/`prepare`). Canlı: inek, domuz ve eşya altında gölge görüldü.
- Sapma: `getSkyDarken` yok (gece gölgeleri koyu kalır); yalnız çizilen türlerin yarıçapı; yerel oyuncu gölgesi oracle'sız.
- Açık: `gameplay/sky_light_level` ortam özniteliği (gece/yağmur) ayrı iş; tür başına `getShadowRadius` geçersiz kılmaları aile görevlerinde.
