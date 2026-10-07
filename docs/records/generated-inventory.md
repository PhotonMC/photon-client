# Üretilmiş tabloların envanteri (D01)

Bu kayıt `crates/voxel-data/src/generated/` altındaki 28 dosyanın her birini Java kaynağına, hedef biçime ve doğrulayan karşılaştırmaya eşler. Sınıflandırma [kalite incelemesinin](../code-quality-review-2026-10-07.md) §6'sına dayanır: **(a)** Java kodunda tanımlı, elle yazılacak Rust modülü; **(b)** JAR `data/` JSON'u veya sunucunun eşitlediği kayıt, çalışma anında okunur; **(c)** Java'nın hesapladığı, hesaplanır. Bu kayıt kod taşımaz; D02–D07 bu tabloya göre ilerler.

Doğrulama: her dosya bir Java yakalamasındaki (`outputs.jsonl`) satırdan `tools/generate/<ad>.py` ile yazılır ve **her parity koşusunda** davranış adaptörü üzerinden baştan sona oracle ile karşılaştırılır ([yerel kurulum](../local-setup.md), "yenilemek için"). Elle yazılan karşılığın doğrulaması aynı karşılaştırmadır: tablo Rust çıktısından türetilmez, oracle'dan gelir.

## Tablo

| Dosya | Satır | Java kaynağı | Üretici (yakalama ailesi) | Sınıf | Hedef Rust yolu |
| --- | ---: | --- | --- | :-: | --- |
| `animations.rs` | 8146 | `client/animation/definitions/*Animation` (`WardenAnimation`, `CamelAnimation`…) | `animations.py` (entity) | a | `voxel-data/src/animation/<java_sınıfı>.rs` |
| `entity_models.rs` | 4341 | `client/model/**/*Model.createBodyLayer`, `LayerDefinitions.createRoots` | `entity_models.py` (entity) | a | `voxel-data/src/model/<java_sınıfı>.rs::create_body_layer()` |
| `block/blocks.rs` | 2408 | `world/level/block/Blocks` (tanım sırası) | `blocks.py` (block) | a | `voxel-data/src/block/blocks.rs`, Java'daki sırayla `Block::of(..)` zincirleri |
| `block/classes.rs` | 1773 | `world/level/block/*Block` sınıf hiyerarşisi (`extends`) | `blocks.py` (block) | a | `voxel-data/src/block/classes.rs`; kural aramaları bu ağaçtan yürür |
| `block/properties.rs` | 376 | `BlockStateProperties` alanları | `blocks.py` (block) | a | `voxel-data/src/block/properties.rs` |
| `block/tints.rs` | 37 | `client/color/block/BlockColors` kayıtları | `block_tints.py` (block) | a | `voxel-data/src/block/tints.rs` |
| `block_sounds.rs` | 1350 | `SoundType` sabitleri ve her bloğun türü | `block_sounds.py` (sound) | a | `voxel-data/src/sound/block_sounds.rs` |
| `sound_events.rs` | 1975 | `sounds/SoundEvents` (kayıt sırası) | `sound_events.py` (sound) | a | `voxel-data/src/sound/sound_events.rs` |
| `damage_effects.rs` | 70 | `DamageType` verisi (`data/minecraft/damage_type/*.json`) ve `DamageEffects` | `damage_effects.py` (sound) | b | çalışma anında yerel oyun dosyasından (`data/`) |
| `entity_attributes.rs` | 146 | `world/entity/ai/attributes/DefaultAttributes` | `entity_attributes.py` (entity) | a | `voxel-data/src/entity/default_attributes.rs` |
| `entity_picks.rs` | 167 | `EntityType` oluşturucu boyutları ve bayrakları (imleç seçimi) | `entity_picks.py` (movement) | a | `voxel-data/src/entity/entity_types.rs` içinde `EntityType` tanımlarıyla |
| `entity_serializers.rs` | 50 | `network/syncher/EntityDataSerializers` | `entity_serializers.py` (entity) | a | `voxel-data/src/entity/data_serializers.rs` |
| `entity_types.rs` | 165 | `world/entity/EntityType` (kayıt sırası) | `entity_types.py` (entity) | a | `voxel-data/src/entity/entity_types.rs` |
| `entity_variants.rs` | 34 | veri paketi: kedi/tavuk/inek/kurbağa/domuz/zombi nautilus çeşitleri (`data/**/*_variant/*.json`) | `entity_variants.py` (entity) | b | çalışma anında yerel oyun dosyasından; sunucu kaydı ile eşitlenir (`RegistryData`) |
| `wolf_variants.rs` | 18 | veri paketi kurt çeşitleri | `wolf_variants.py` (entity) | b | aynı |
| `painting_variants.rs` | 60 | veri paketi tablo çeşitleri | `painting_variants.py` (entity) | b | aynı |
| `sheep_colors.rs` | 44 | `DyeColor` doku renkleri (`getTextureDiffuseColor`) | `sheep_colors.py` (entity) | a | `voxel-data/src/entity/dye_colors.rs` |
| `villager_registries.rs` | 36 | `VillagerType`/`VillagerProfession` kayıtları; şapka bilgisi köylü doku `.mcmeta` dosyalarından | `villager_registries.py` (entity) | a + b | adlar elle (`villager_registries.rs`), şapka bilgisi çalışma anında asset'ten |
| `map_data.rs` | 81 | `world/level/material/MapColor`, `MapDecorationType` | `map_data.py` (entity) | a | `voxel-data/src/map_data.rs` |
| `item/items.rs` | 1892 | `world/item/Items` (kayıt sırası) | `items.py` (block) | a | `voxel-data/src/item/items.rs` |
| `item/classes.rs` | 312 | `world/item/*Item` sınıf hiyerarşisi | `items.py` (block) | a | `voxel-data/src/item/classes.rs` |
| `item/component_types.rs` | 240 | `core/component/DataComponents` (kayıt sırası) | `component_types.py` (block) | a | `voxel-data/src/item/component_types.rs` |
| `item/creative_tabs.rs` | 4345 | `world/item/CreativeModeTabs` (varsayılan özellikler, operatörsüz oyuncu) | `creative_tabs.py` (block) | a | `voxel-data/src/item/creative_tabs.rs`, sekme başına modül (sınır 600 satır) |
| `item/attributes.rs` | 50 | `world/entity/ai/attributes/Attributes` | `attributes.py` (block) | a | `voxel-data/src/item/attributes.rs` |
| `item/mob_effects.rs` | 50 | `world/effect/MobEffects` | `potions.py` (block) | a | `voxel-data/src/item/mob_effects.rs` |
| `item/potions.rs` | 55 | `world/item/alchemy/Potions` | `potions.py` (block) | a | `voxel-data/src/item/potions.rs` |
| `block/mod.rs`, `item/mod.rs`, `mod.rs` | 6 + 9 + 21 | — | elle (modül bildirimleri) | – | D07'de hedef modüllerle birleşir |

Ek üretici çıktısı: `item_components.py` bir Rust dosyası değil `tests/behavior/fixtures/item-default-patches.txt` yazar (izlenen sentetik fixture); sınıf (a) eşdeğeri `item/` modüllerindeki bileşen yamalarıdır.

## Sınıf özeti

| Sınıf | Dosya | Satır | Not |
| --- | ---: | ---: | --- |
| (a) elle Rust modülü | 22 | ≈ 28 000 | Java'daki dosya düzenini izler; her modül 600 satırı geçmez ([kural](../coding-standards.md#dosya-boyutu)): `animations` ve `entity_models` sınıf başına, `creative_tabs` sekme başına bölünür |
| (b) çalışma anında veri | 4 | ≈ 180 | `damage_effects`, üç çeşit tablosu; kaynak oyunun yerel `data/` klasörü veya sunucunun eşitlediği kayıtlar |
| (a + b) | 1 | 36 | `villager_registries` |
| (c) hesaplanır | 0 | 0 | `block/classes.rs` ve `item/classes.rs` Java'da sınıf hiyerarşisidir (tanım), hesaplama değil; hiçbir tablo salt Java tarafından hesaplanmıyor |

## Kararlar ve açık noktalar

- **Elle modül düzeni:** Java sınıfı başına bir dosya, içinde Java'daki sırayla tanımlar. Okunur oluşturucu DSL'i (`Block::of(..)`, `ModelLayer::of(..)`) korunur. Büyük tablolar (blok, eşya, ses olayı) sınıf adı yerine **kayıt aralığına** göre ≤ 600 satırlık modüllere bölünür (örn. `blocks/stone.rs` yerine alfabetik veya Java sırasında ardışık parçalar).
- **(b) kaynağı:** `voxel-data` çalışma anında dosya okumaz; (b) tabloları `voxel-game`/`voxel-net` tarafında yüklenir (asset ile aynı yol, `VOXEL_ASSETS`) ve sunucu `RegistryData` ile eşitlenir. Çevrimdışı kalan istemci için varsayılan yok: eksik kaynak başarı sayılmaz.
- **Tek seferlik dönüşüm** mevcut üretici çıktısından betikle yapılabilir; ardından `tools/generate/` ve `generated/` silinir ve parity karşılaştırması olduğu gibi kalır (D07). Silme, yeni sürüme geçişte üretici zincirinin (Java oracle + Python) bırakılması demektir: kural metinlerindeki ("tablo oracle'dan yakalanır, üretilmiş Rust dosyasına yazılır") ifadeyi değiştirmek proje sahibinin kararıdır.
- **Hukuki not** ([inceleme](../code-quality-review-2026-10-07.md) §6): Java tanımlarının elle aktarılması da Mojang kodundan türev veridir; bu kayıt hukuki güvence vermez.
