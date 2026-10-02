# T24b — Pig/cow çeşitleri (variant) ve entity metadata okuyucusu

**Amaç:** T24'te domuz ve inek her zaman `temperate` çeşidiyle çiziliyor. Java `PigVariant`/`CowVariant` kayıtlarını sunucudan alır; hangi çeşidin giyildiği entity verisindeki holder kimliğidir (model: normal/soğuk/sıcak, doku, bebek dokusu).
**Bağımlılık:** T24. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/world/entity/animal/pig/{Pig,PigVariant,PigVariants}.java`, `.../cow/{Cow,CowVariant,CowVariants}.java`, `net/minecraft/network/syncher/EntityDataSerializers.java` (serileştirici kimlikleri: kayıt sırası), `.../SynchedEntityData.java` (paket biçimi), `net/minecraft/client/renderer/entity/{PigRenderer,CowRenderer}.java`.
- Rust: `crates/voxel-world/src/metadata.rs` (`skip_metadata_value`, `find_metadata_field` yalnız bilinen serileştiricileri atlar; yenileri ekle), `crates/voxel-net` registry sync (`registry_names("minecraft:pig_variant")`, kayıt verisi), `crates/voxel/src/mobs.rs` (`KINDS`).

## Yapılacaklar
1. `EntityDataSerializers` tüm kimlik ve değer biçimleri (oracle tablosu: kimlik → ad), `skip_metadata_value` hepsini atlayabilsin; böylece bilinmeyen serileştirici sonraki alanları kaybettirmez.
2. Pig/Cow çeşit alanının veri indeksi (Java sınıf sırası) → `ClientEntity.variant: Option<i32>`.
3. Çeşit kaydı verisi (`asset_id`, `baby_asset_id`, `model`) sunucudan okunur (registry data); `mobs.rs` çeşitten model katmanı ve doku seçer (`cold_pig#main`, `warm_cow#main`…).

## Oracle
`entity` ailesine serileştirici kimlik tablosu ve çeşit alanı okuma satırları (Java `SynchedEntityData` ile kodlanmış paketten).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`; canlı: soğuk biyomda domuz soğuk dokuyla.

## Kapsam dışı
Diğer mob'lar (PLAN-M4), eyer katmanı.

## Belgeler
`docs/remaining-work.md` M4.

## Devir notu
(boş)
