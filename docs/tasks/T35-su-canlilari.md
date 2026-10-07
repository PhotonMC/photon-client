# T35 — Su canlıları: kalamar, balıklar, yunus, aksolotl, kaplumbağa, kurbağa, nautilus, gardiyan

**Amaç:** Suda yaşayan mobların modelleri ve su içi animasyonları: kalamar/ışıklı kalamar (dönüş durumu), morina/somon/tropikal balık/kirpi balığı (üç boy), iribaş, yunus, aksolotl (oyun ölü taklidi), kaplumbağa, kurbağa (`KeyframeAnimation`), nautilus ve zombi nautilus, gardiyan ve elder gardiyan (ışın).
**Bağımlılık:** T24, T43b. **Boy:** L. **Türler:** squid, glow_squid, cod, salmon, tropical_fish, pufferfish, tadpole, dolphin, axolotl, turtle, frog, nautilus, zombie_nautilus, guardian, elder_guardian.

## Oku
- Java: `client/model/animal/squid/SquidModel.java`, `client/model/animal/fish/CodModel.java`, `client/model/animal/dolphin/DolphinModel.java`, `client/model/animal/turtle/TurtleModel.java`, `client/model/animal/frog/FrogModel.java`, `client/model/monster/guardian/GuardianModel.java`, `client/renderer/entity/SquidRenderer.java`, `client/renderer/entity/TropicalFishRenderer.java`, `client/renderer/entity/PufferfishRenderer.java`, `client/renderer/entity/GuardianRenderer.java`.
- Rust: `crates/voxel-render/src/entity/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity/textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Suda dönüş/yüzme durumları (`Squid.xBodyRot` vb.) entity tikinde (yalnız oracle'lanmış kollar).
2. Balık aileleri: aynı model, farklı desen katmanı (tropikal balık deseni/renk verisi).
3. `KeyframeAnimation` kullananlar (kurbağa, aksolotl) T43b sonrası.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Balık tutma, su altı ses ve parçacıkları.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Çok aile var: ilk oturumda kalamar+balıklar, ikincisinde geri kalan; gerekirse böl. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
(boş)
