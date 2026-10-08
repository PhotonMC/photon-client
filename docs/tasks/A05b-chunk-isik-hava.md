# A05b — `Chunks`, ışık, hava ve düzey bilgisi alt sistemleri

**Amaç:** A05'in kalanı: `ClientWorld`'ün chunk/ışık alanlarını, hava durumunu ve boyut bilgisini alt sistem tiplerine almak; yerel oyuncu–entity bağını çözmek.
**Bağımlılık:** A05. **Boy:** M.

## Oku
- [A05 devir notu](A05-clientworld-bolme.md) (`Entities` örneği ve gözlem)
- `crates/photon-world/src/world/{chunks,lighting,weather,dimension}.rs`

## Yapılacaklar
- `Chunks { map, cache, light, light_tasks, light_dirty }` ve yalnız bu alanlara dokunan yöntemleri taşı (`world/chunks.rs`, `lighting.rs`: 17 tek-alanlı yöntem). Zemin/ışık için tüm dünyayı isteyen yöntemler `ClientWorld`'de kalır.
- `Weather { rain_level, thunder_level, sky_flash_time }`, `Level` (boyut anahtarı, tip kimliği, tohum, bayraklar, deniz seviyesi, ortam).
- `local_player`'ı entity deposuna bağlayan yöntemler (`entity_exists`, `apply_entity_metadata`, `set_entity_vehicle_id`, …) için ortak bir "kimin entity'si bu" arayüzü; `Entities` yöntemleri `ClientWorld`'e bağımlı kalmasın.
- Çağıranlar betikle (A05 `Entities` örneği gibi) geçirilir.

## Kabul
`python3 tools/photon.py check` → `CHECK OK`; davranış değişmez.

## Kapsam dışı
Harita yapısı (hash/slab) değişikliği; davranış değişikliği.

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
(boş)
