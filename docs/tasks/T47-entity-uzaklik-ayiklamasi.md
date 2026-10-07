# T47 — Entity uzaklık ve bölüm görünürlüğü ayıklaması

**Amaç:** Java'nın çizmediği entity'leri çizmemek. Bugün yalnız frustum'un tamamen dışındakiler atlanıyor (`crates/voxel/src/play/cull.rs`, kutu Java'nınkinden geniş); `Entity.shouldRenderAtSqrDistance` (kutu boyu × 64 × `viewScale`) ve `LevelExtractor.isEntityVisible`'ın bölüm koşulu (`isSectionCompiledAndVisible`) yok, bu yüzden uzak küçük entity'ler (yerdeki eşya, ok) Java'dan fazla çiziliyor.
**Bağımlılık:** T24. **Boy:** S. **Türler:** bütün entity'ler.

## Oku
- Java: `client/renderer/extract/LevelExtractor.java` (`extractVisibleEntities`, `isEntityVisible`), `client/renderer/entity/EntityRenderer.java` (`shouldRender`, `getBoundingBoxForCulling`, `affectedByCulling`) ve onu ezen çiziciler (`grep -rn "getBoundingBoxForCulling\|affectedByCulling\|shouldRenderAtSqrDistance"`), `world/entity/Entity.java` (`shouldRender`, `viewScale`).
- Rust: `crates/voxel/src/play/cull.rs`, `crates/voxel-world/src/world/entities/store.rs` (`drawn`), `crates/voxel-world/src/pick.rs` (`bounding_box`).

## Yapılacaklar
1. `viewScale = clamp(görüş mesafesi / 8, 1, 2.5) × entityDistanceScaling`; türlerin `shouldRenderAtSqrDistance` ezmeleri (ör. ok, eşya, display) tablo satırı olarak.
2. Kutu: `getBoundingBoxForCulling` ve ezmeleri (+0.5; illusioner, sniffer, trident, mutlu ghast, minecart, ejderha başı); `affectedByCulling` false olanlar hiç atlanmaz; tasma (`Leashable`) kolu.
3. Bölüm koşulu: entity'nin bloğu derlenmiş ve görünür bölümde değilse (dünya yüksekliği dışı hariç) atlanır.
4. Bugünkü geniş kutu kaldırılır; `VOXEL_CULLING=0` hepsini kapatmaya devam eder.

## Oracle
`living_ticks` ailesinde gerçek `EntityRenderDispatcher.shouldRender` ve `LevelExtractor.isEntityVisible`: tür × uzaklık × kamera açısı tablosu, sonuç bit eşit.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`; `tools/voxel.py bench` p50 gerilemesi yok.

## Kapsam dışı
Gölge, ad etiketi ve tasma çizgisi geçişleri (Java'da çizilen entity'ye bağlı; ayrı inceleme).

## Belgeler
`docs/live-play-session.md` ayıklama paragrafı; yeni `client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Devir notu
