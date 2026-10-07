# A01c — Kimlik newtype'ları (`EntityId`, `BlockStateId`, `EntityTypeId`)

**Amaç:** A01b'nin bölünen kalanı: ham `i32`/`u32`/`i64` kimlikleri adlandırılmış tiplere almak; entity türünün dizeyle aranmasını kimlik tablosundan sabite çevirmek.
**Bağımlılık:** A01b. **Boy:** L.

## Oku
- [A01b devir notu](A01b-kalan-newtype.md), inceleme raporu §3.3
- `grep -rn "entity_id\|type_id\|state_id" crates/*/src` (≈1700 kullanım, 250 dosya)

## Yapılacaklar
- `EntityId(i32)`, `EntityTypeId(i32)`, `BlockStateId(u32)` `voxel-core`'da (`Debug`, `Copy`, `Ord`, `Hash`; `new`/`get`). `BlockStateId` için bugün üç tip karışık (`i32`, `u32`, `i64::from(state)`): birine indirilir.
- Önce voxel-world içinde (`ClientEntity.id`, `type_id`, `passenger_ids`, `vehicle_id`, `HashMap<i32, ClientEntity>` anahtarları, `state_or_air`), sonra sınırlarda: paket ayrıştırıcıları ham değeri okur ve tipe sarar (`voxel-net`), `voxel-game`/`voxel-render` imzaları.
- `entity_type::id("minecraft:fox")` gibi dize aramaları `EntityTypeId` sabitlerine (`voxel-data` kimlik tablosundan) iner; `VANILLA_*_ENTITY_TYPE_ID` sabitleri tek yerde.

## Kabul
`python3 tools/voxel.py check` → `CHECK OK`; davranış değişmez (kimlikler ağdaki değerlerle aynı).

## Kapsam dışı
Davranış değişikliği; `ClientEntity`'nin tür enum'una bölünmesi (A02, bitti).

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
(boş)
