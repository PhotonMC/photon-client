# T24b — Entity metadata serileştiricileri ve pig/cow çeşitleri — [client-parity]

Bu kayıt, entity verisindeki (`SetEntityData`) tüm serileştirici kimliklerinin
Java ile aynı numaralanmasını, domuz ve ineğin çeşit alanının yerini ve çeşit
verisini (doku, bebek dokusu, model ailesi) kapsar. Kanıt sınıfı `client-parity`'dir:
her tablo Java'nın kendi kayıtlarından yakalanır, `voxel-data` altına üretilir ve
her parity koşusunda baştan sona karşılaştırılır. Canlı kabul (soğuk/sıcak domuz
ve inek, bebekleri) `server-conformance` gözlemidir.

## Java kaynak incelemesi

- `network/syncher/EntityDataSerializers.java`: serileştirici kimliği
  `registerSerializer` sırasıdır (43 kayıt, `BYTE` 0 … `HUMANOID_ARM` 42).
  Değer biçimleri: bayt/bool bir bayt; `INT`, durum/enum/yön/poz ve kayıt
  tutacağı (`holderRegistry`) bir var int; `LONG` var long; `FLOAT` 4 bayt;
  `STRING` uzunluk + bayt; `COMPONENT` NBT; `ITEM_STACK` isteğe bağlı yığın
  (`count`, öğe, bileşen yaması); `ROTATIONS`/`VECTOR3` 12, `QUATERNION` 16 bayt;
  `OPTIONAL_LIVING_ENTITY_REFERENCE` bayrak + UUID; `OPTIONAL_GLOBAL_POS` bayrak +
  boyut adı + konum; `VILLAGER_DATA` üç var int. `PAINTING_VARIANT` `holder`
  (0 = değer gömülü).
- `world/entity/animal/pig/Pig.java:66`, `cow/Cow.java:33`: çeşit alanı sınıf
  zincirinin sırasıyla domuzda 19, inekte 18; varsayılan (`temperate`)
  gönderilmez, istemci aynı varsayılanı kullanır.
- `client/renderer/entity/{PigRenderer,CowRenderer}.java`: çeşit çözülemezse
  mob çizilmez; model ailesi `normal`/`cold`/`warm` ile yetişkin ve bebek
  katmanı (`pig`, `cold_pig`, `pig_baby`; `cow`, `cold_cow`, `warm_cow` ve
  `_baby`'leri) ve çeşidin yetişkin/bebek dokusu seçilir. Çeşit, kayıttaki yeri
  (`registry_names`) ile adlandırılır.

## Veri tabloları

- `entity/data-serializers` → `generated/entity_serializers.rs` (43 ad).
- `entity/variants` → `generated/entity_variants.rs` (6 çeşit).
- `entity/variant-fields` (veri indeksleri ve serileştirici kimlikleri Java'nın
  `DATA_VARIANT_ID` erişimcilerinden; Rust sabitleriyle karşılaştırılır — ilk
  tahminim domuz için 18'di, oracle 19 olarak düzeltti).
- Yenilemek için yakalama sonrası `python3 tools/generate/entity_serializers.py`
  ve `entity_variants.py <capture>/outputs.jsonl`.

## Kapsam ve sapmalar (açık)

- Serileştirici yürüyüşü `metadata.rs`'te; bayt biçimleri Java'dan yakalanmış
  bayt dizisiyle değil, kodek kaynağından yazılmış ve birim testlerle (her
  serileştirici için elle kurulmuş değer) denenmiştir: oracle Java'dan Rust'a bayt
  taşıyamadığı için. Canlı oturumda bilinmeyen değer, kendinden sonraki alanları
  okutmaz (öncekiyle aynı davranış).
- Yürünemeyen değerler: veri taşıyan parçacıklar (`PARTICLE` yalnız
  `entity_effect` rengiyle), `RESOLVABLE_PROFILE` ve gömülü boyalı `PAINTING_VARIANT`.
- Domuz ve inek dışındaki çeşitli mob'lar (kedi, kurt, kurbağa, tavuk…) bu
  alanı okumaz; çizimleri PLAN-M4'te.
- Eyer katmanı yok (kapsam dışıydı).
