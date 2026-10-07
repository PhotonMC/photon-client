# T32c — Tilki, panda, kutup ayısı

**Amaç:** Tilki (uyku, çömelme, saldırı), panda (oturma, yuvarlanma, hapşırma, gen) ve kutup ayısı (ayağa kalkma).
**Bağımlılık:** T32. **Boy:** L.

## Oku
- Java: `client/model/animal/fox/FoxModel.java`, `client/model/animal/panda/PandaModel.java`, `client/model/animal/polarbear/PolarBearModel.java`, ilgili `*Renderer.java`, `world/entity/animal/{fox/Fox,panda/Panda,polarbear/PolarBear}.java`.
- Rust: `crates/voxel/src/wolves.rs` (kalıp: çeşit dokusu, tasma katmanı), `voxel-render/src/mob_model.rs` (`setup_wolf`), `voxel-world/src/wolf.rs`, `docs/records/client-wolf-parity-conformance.md`.

## Yapılacaklar
1. Her aile için `setupAnim` dalı ve istemci durumu (`Fox` bayrakları, `Panda` sayaçları, kutup ayısı `standScale`).
2. Çeşit/doku (tilki kırmızı/kar, panda geni).

## Oracle
`wolf_model` / `living-entity-animations` kalıbı: gerçek model + `setupAnim`; tik durumu gerçek varlıkla.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır.

## Kapsam dışı
Evcil hayvan etkileşimi (sunucu), zırh/ekipman katmanları.

## Belgeler
`docs/index.md` satırı, yeni `docs/client-*-parity-conformance.md`.

## Devir notu
- Yapıldı: yalnız kutup ayısı (bölündü). `setup_polar_bear` (yetişkin + bebek, ayağa kalkma), `voxel-world/src/polar_bear.rs` (kalkış sayacı, olay yok: veri bayrağı 18), `crates/voxel/src/bears.rs`. Kayıt: `docs/records/client-polar-bear-parity-conformance.md`.
- Oracle: `polar-bear-model` 10, `living-entity-animations` 2 ayı, `variant-fields` +1.
- Sapma: ayağa kalkınca çarpışma kutusu uzamıyor (mob kutusu poz taşımaz); ses yok.
- Açık: T32e tilki, T32f panda.
