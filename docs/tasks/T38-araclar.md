# T38 — Tekneler

**Amaç:** Tekne ve sandıklı tekne (ahşap türleri, bambu salı): model, çizim duruşu, vuruş sallanması, kabarcık sütunu, kürek çekme, su altı durumu. Minecart ailesi T38b'ye ayrıldı.
**Bağımlılık:** T24. **Boy:** M. **Türler:** *_boat, *_chest_boat, bamboo_raft, bamboo_chest_raft.

## Oku
- Java: `client/model/object/boat/BoatModel.java`, `client/renderer/entity/BoatRenderer.java`, `client/model/object/cart/MinecartModel.java`, `client/renderer/entity/AbstractMinecartRenderer.java`, `client/renderer/entity/MinecartRenderer.java`.
- Rust: `crates/photon-render/src/entity/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/photon/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/photon-render/src/entity/textures.rs` (`rect`), `crates/photon-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Tekne çizimi (`BoatRenderState`, kürek açısı `rowingTime`, balçık/su yarığı `WaterPatch`); ahşap türü doku satırı.
2. Minecart: yönelme ve ray eğimi (`NewMinecartBehavior` istemci iç-konumlaması); içerik bloğu (`getDisplayBlockState`).
3. Oyuncunun bindiği araç kontrolü T44'te.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Sürüş (T44), ray fiziği sunucu tarafında, sesler; minecart (T38b).

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Tekne dalı + minecart dalı; türevler tablo satırı. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı: `setup_boat`, `photon-render/boat.rs` (`boat_pose`, `boat_vertices`, `Quaternion::angle_axis`), `photon-world/boat.rs` (vuruş sönmesi, kabarcık, kürek konumları, su altı durumu), `crates/photon/src/objects/boats.rs`; oracle sahnesi yaşayan olmayan varlıkları da tikler. Kayıt: `docs/records/client-boat-parity-conformance.md`.
- Bulgu: `Quaternionf.setAngleAxis` ekseni normalleştirmez (`(1,0,1)` ekseninde kabarcık dönüşü).
- Bölündü: minecart T38b'de. Sapma: su yaması çizilmez (derinlik-yalnız çizim yok), yolcu ile kürek çekme oracle'sız. Canlı gözlem yapılmadı.
