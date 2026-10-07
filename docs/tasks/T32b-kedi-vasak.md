# T32b — Kedi, vaşak (ocelot)

**Amaç:** Kedi (çeşit kayıt defteri, tasma, oturma/uzanma) ve ocelot (`CatModel`, `OcelotModel`, `CatCollarLayer`).
**Bağımlılık:** T32. **Boy:** M.

## Oku
- Java: `client/model/animal/feline/{FelineModel,CatModel,OcelotModel,BabyCatModel}.java`, `client/renderer/entity/{CatRenderer,OcelotRenderer}.java`, `client/renderer/entity/layers/CatCollarLayer.java`, `world/entity/animal/feline/{Cat,Ocelot}.java`.
- Rust: `crates/voxel/src/wolves.rs` (kalıp: çeşit dokusu, tasma katmanı), `voxel-render/src/mob_model.rs` (`setup_wolf`), `voxel-world/src/wolf.rs`, `docs/records/client-wolf-parity-conformance.md`.

## Yapılacaklar
1. `FelineModel.setupAnim` (koşu, çömelme, oturma, uzanma).
2. Kedi çeşit kaydı ve tasma; bayraklar `TamableAnimal` verisinde (T32'de var).
3. Uzanma/ilgi eğimleri `Cat.tick` istemci durumu.

## Oracle
`wolf_model` / `living-entity-animations` kalıbı: gerçek model + `setupAnim`; tik durumu gerçek varlıkla.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır.

## Kapsam dışı
Evcil hayvan etkileşimi (sunucu), zırh/ekipman katmanları.

## Belgeler
`docs/index.md` satırı, yeni `docs/client-*-parity-conformance.md`.

## Devir notu
- Yapıldı: `setup_feline` (yetişkin + bebek kedi/ocelot: çömelme, koşma, oturma, yatma, rahatlama), `voxel-world/src/cat.rs` (yatma/kuyruk/rahatlama miktarları, uyuyan oyuncu araması), kedi veri kimlikleri ve `entity_variants`'a on bir kedi çeşidi (oracle'dan), `crates/voxel/src/felines.rs` (doku, yatış yerleşimi `MobPlacement.shift/roll_shift`, tasma katmanı kendi modeliyle). Kayıt: `docs/records/client-feline-parity-conformance.md`.
- Oracle: `feline-model` 48, `living-entity-animations` 6 kedi, `variants` +11, `variant-fields` +4.
- Sapma: uyuyan oyuncu dalı yalnız birim testi (sahnede kurulamıyor); ocelot güven verisi okunmuyor; ses yok.
- Açık: T32c, T32d.
