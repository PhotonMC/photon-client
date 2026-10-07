# T46 — Varlık öznitelikleri — [client-parity]

Bu kayıt, yaşayan varlıkların özniteliklerinin istemcide tutulmasını kapsar: her türün
başladığı değerler, sunucunun `UpdateAttributes` paketinin bunları değiştirmesi, değerin
modifikatörlerle hesabı ve aralığa sıkıştırılması; `getMaxHealth` ve `getScale` buna dayanır.
Kanıt sınıfı `client-parity`'dir: varsayılan tablo gerçek `DefaultAttributes`'tan üretilir ve
her koşuda karşılaştırılır; paket uygulaması ve değer, gerçek `ClientPacketListener` ve gerçek
varlıkla (`movement/living-entity-animations`) bit bit karşılaştırılır.

## Java kaynak incelemesi

- `DefaultAttributes`: her yaşayan tür bir `AttributeSupplier` ile başlar (tür başına farklı
  öznitelik kümesi ve taban değerler); istemci varlığı bunu kopyalar.
- `ClientPacketListener.handleUpdateAttributes`: türün başlangıçta sahip olmadığı öznitelik
  yok sayılır; sahip olduğunun tabanı paketin tabanı, modifikatörleri paketinkiler olur.
- `AttributeInstance.getValue`: taban + `ADD_VALUE`'lar, sonra `ADD_MULTIPLIED_BASE`, sonra
  `ADD_MULTIPLIED_TOTAL`; sonuç özniteliğin aralığına sıkışır (`RangedAttribute`:
  `max_health` 1–1024, `scale` 0,0625–16).
- `LivingEntity.getMaxHealth`, `getScale` bu değerlerdir.

## Veri

`tools/generate/entity_attributes.py` (yenilemek için `all.py`): `ENTITY_ATTRIBUTES` (93 tür)
ve `ATTRIBUTE_RANGES` (40 öznitelik: varsayılan ve aralık); `entity/attributes` satırlarıyla her
koşuda baştan karşılaştırılır. Bu kayıt oyuncunun kendi özniteliklerini (P10) değiştirmez.

## Kapsam

`entity/attributes` 1 satır (iki tablo), `living-entity-animations`'a beş satır (domuz ve demir
golem varsayılanı; değişmiş taban ve iki modifikatör; aralık dışı değerler; türün sahip
olmadığı öznitelik).

## Kullananlar

Demir golemin çatlak düzeyi (`max_health`), tüm varlıkların çizim ölçeği (`scale` ile
`placement`).

## Sapmalar (açık)

- Bebek ölçeği (`ageScale`) ayrı yolda; öznitelik değil.
- Hız, saldırı gibi öznitelikler okunur ama kullanan yok (hareket P10'da).
