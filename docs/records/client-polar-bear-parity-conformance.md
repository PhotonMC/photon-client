# T32c — Kutup ayısı — [client-parity]

Bu kayıt, kutup ayısının (yetişkin ve bebek) modelinin Java 26.2 ile Rust'ta aynı köşe konum, doku
koordinatı ve normali üretmesini; ayağa kalkma animasyonunun istemci durumunun aynı ilerlemesini kapsar.
Kanıt sınıfı `client-parity`'dir (model ve tik) ve canlı gözlemdir: gerçek `PolarBearModel` katman
tablosundan pişirilip gerçek `setupAnim` ile pozlanır; ayının tiki gerçek `PolarBear` varlığıyla ve
gerçek veri paketiyle karşılaştırılır.

## Java kaynak incelemesi

- `PolarBearModel.setupAnim`: dört ayaklı pozu (`QuadrupedModel`) kurulduktan sonra ayağa kalkma ölçeği
  karesi (`standScale²`) gövdeyi `0,35π` geri yatırır ve `ageScale` ile yükseltir; ön bacaklar `ageScale`
  ile yukarı-öne kayar ve `0,45π` kalkar (sol bacak sağın yüksekliğini ve derinliğini alır); baş
  yukarı-öne kayar ve `0,15π` eğilir. Yetişkin ve bebek aynı sınıfı kullanır, yalnız katman farklıdır.
- `PolarBear.tick` (istemci): ayağa kalkma sayacı veri bayrağına göre tikte bir artar ya da azalır, 0 ile 6
  arasında kenetlenir; `getStandingAnimationScale` bunu kesre çevirir ve tikler arasında ara değer alır.
  Veri: `PolarBear.DATA_STANDING_ID` 18 (boolean).
- `PolarBearRenderer`: tek doku (yetişkin/bebek), çeşit yok.

## Kapsam

`entity/polar-bear-model` 10 satır (iki katman × duruş, bakış, yürüyüş, kalkarken, kalkmış);
`living-entity-animations`'a iki satır (boşta, kalkıp oturan); `entity/variant-fields`'a bir satır.

## Sapmalar (açık)

- Ayağa kalkınca ayının çarpışma kutusu da uzar (`getDefaultDimensions`); istemcide mob kutusu bu
  durumu taşımıyor (`ClientEntity::bounding_box` ölçek ve poz değişimini taşımaz), yalnız çizim eşit.
- Uyarı sesi (`warningSoundTicks`) ve ses yok.
