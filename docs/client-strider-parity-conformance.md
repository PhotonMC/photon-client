# T32h — Strider — [client-parity]

Bu kayıt, striderin (yetişkin, bebek ve eyer katmanının modeli) Java 26.2 ile Rust'ta aynı köşe konum,
doku koordinatı ve normali üretmesini; doku seçiminin Java'nın sabitleriyle aynı olmasını kapsar. Kanıt
sınıfı `client-parity`'dir (model, tablo ve tik) ve canlı gözlemdir: gerçek `AdultStriderModel`,
`BabyStriderModel` ve eyerin katman tablosundan pişirilip gerçek `setupAnim` ile pozlanır.

## Java kaynak incelemesi

- `StriderModel.setupAnim`: yürüyüş hızı en çok `0,25` sayılır; binilmemişken gövde bakışı izler, binilmişken
  öne bakar; gövde `0,1·sin(konum·1,5)·4·hız` yana yatar; bacaklar `sin`/`cos(konum·1,5·0,5)` ile
  çaprazlama sallanır. Yetişkinde gövde `2` yükseklikten, bebekte `17,25`'ten yürüyüşle iner, bacak
  yükseklikleri (`8`/`20`) sallanır; tüyler (yetişkinde altı yan, bebekte üç) yürüyüşün akışıyla
  (`cos(konum·1,5+π)·hız`, tüy başına `0,6/1,2/1,3`) ve yaşa bağlı üç dalgayla (`0,1·sin(0,4·yaş)`,
  `0,1·sin(0,2·yaş)`, `0,05·sin(-0,4·yaş)`) oynar: yetişkinde Z, bebekte X dönüşüne eklenir.
- `StriderRenderer`: soğuk (boğulan) dokusu, yetişkin ve bebek için ayrı dört doku; eyer
  `SimpleEquipmentLayer` ile `strider_saddle` katmanında, eyer ekipman yuvasındaki eşyanın
  `equippable` bileşenindeki varlıkla, yetişkin modelin eyer katmanıyla ve aynı pozla çizilir (bebek eyer
  taşımaz). Boğulma ve binilmişlik çizim durumuna girer.
- Veri: `Strider.DATA_SUFFOCATING` 19 (boolean); binilmişlik yolcu olmasıdır.

## Kapsam

`entity/strider-model` 18 satır (üç katman × duruş, bakış, yürüyüş, koşu, binilmiş iki hâl);
`entity/strider-textures` 4 satır; `living-entity-animations`'a iki satır (sıcak, boğulan);
`entity/variant-fields`'a bir satır. Eyer katmanı genel bir "basit ekipman katmanı" yoluyla çizilir
(`Armor::simple_layer`); ileride atın, domuzun ve devenin eyerleri aynı yolla eklenir.

## Sapmalar (açık)

- Yapay zekâlı strider kendi başına (`Strider.tick`, istemcide de) boğulup boğulmadığını bloklardan ve
  lavdan hesaplar; istemci bunu taşımıyor, sunucunun verisine uyar (aynı kuraldan çıktığı için kararlı
  durumda aynıdır). Oracle senaryoları bu yüzden yapay zekâsızdır (`NoAI`, Java o zaman hesaplamaz).
- Boğulan strider titrer: [hoglin kaydındaki titreme](client-hoglin-parity-conformance.md) (T32i).
- Eyerin çizim sırası (`order`) ve eyer sesi yok; binici ve binme kontrolü ayrı görev (T44).
