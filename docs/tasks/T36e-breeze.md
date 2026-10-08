# T36e — Breeze

**Amaç:** `BreezeModel` (üç katman: gövde, rüzgâr, gözler; altı keyframe animasyonu: boşta, ateş, kayma, geri kayma, soluma, uzun zıplama), `BreezeWindLayer` (kaydırılan doku, `RenderTypes.breezeWind`), `BreezeEyesLayer` (`RenderTypes.breezeEyes`) ve `Breeze` istemci durumları (poz değişince başlayan animasyon durumları).
**Bağımlılık:** T36. **Boy:** M–L.

## Oku
- Java: `client/model/monster/breeze/BreezeModel.java`, `client/renderer/entity/BreezeRenderer.java`, `client/renderer/entity/layers/Breeze{Wind,Eyes}Layer.java`, `client/renderer/rendertype/RenderTypes.java` (`breezeWind`, `breezeEyes`), `world/entity/monster/breeze/Breeze.java` (`onSyncedDataUpdated`, `tick`).
- Rust: `crates/photon-world/src/mobs/frog.rs` (poza bağlı animasyon durumu kalıbı), `crates/photon-render/src/{mob_model,render_types}.rs` (`mob_vertices_swirl`: kaydırılan doku), `crates/photon/src/mobs/monsters.rs`.

## Oracle
`frog_model` kalıbı (gerçek model + animasyon durumları); üç katman ayrı satırlar; gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, rüzgâr topu (T39), davranışlar.

## Devir notu
- Yapıldı: `setup_breeze` (üç katman, altı animasyon), `photon-world/breeze.rs` (poz ve tik durum makinesi), `mob_vertices_scrolled` (kaydırılan ve tekrarlayan rüzgâr; sınırdan kesme) ve `mob_vertices_emissive`, entity gölgelendiricisine iki bant içi işaret (sıfır normal, negatif ışık), `monsters.rs` tablosuna breeze. Kayıt: `docs/records/client-breeze-parity-conformance.md`.
- Karar: yeni boru hattı yerine entity gölgelendiricisine iki işaret; gölgelendirici derlenmesi bir birim testle denetlenir.
- Sapma: gözlerin derinlik yazmaması ve rüzgâr sıralaması yok; doku yolları oracle'sız. Canlı gözlem yapılmadı.
