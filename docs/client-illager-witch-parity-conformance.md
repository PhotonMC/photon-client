# T34b — Illager ve cadı — [client-parity]

Bu kayıt, dört illagerın (gözcü/vindicator, yağmacı/pillager, çağırıcı/evoker,
illusioner) ortak modelinin her kol pozunda ve cadının modelinin Java 26.2 ile
Rust'ta aynı köşe konum, doku koordinatı ve normali üretmesini kapsar. Kanıt sınıfı
`client-parity`'dir: gerçek `IllagerModel` ve `WitchModel` katman tablosundan
pişirilip gerçek `setupAnim` ile pozlanır, parça çizicisiyle çizilir ve her dörtgen
bit bit karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `IllagerModel.setupAnim`: baş bakışla döner; biniciyken kollar ve bacaklar sabit
  oturma pozunda, yürürken karşıt salınımlı. Sonra kol pozu kolları devralır:
  `ATTACKING` boş elde `animateZombieArms` (agresif), tutulan eşyayla `swingWeaponDown`
  (ana koldan hangisi öndeyse); `SPELLCASTING` ve `CELEBRATING` kolları yukarı açar;
  `BOW_AND_ARROW`, `CROSSBOW_HOLD` ve `CROSSBOW_CHARGE` başa göre döner, yük
  `clamp(ticksUsingItem, 0, süre) / süre` ile ilerler. Yalnız `CROSSED` iki kolu gizler
  ve kavuşturulmuş kolları (`arms`) gösterir. Şapka katmanı yapıcıda gizlenir;
  illusioner'ın çizicisi açar.
- Poz seçimi varlıktadır: gözcü saldırgansa `ATTACKING`, değilse kutlarken
  `CELEBRATING`, yoksa `CROSSED`; yağmacı arbaleti dolduruyorsa `CROSSBOW_CHARGE`,
  elindeyse `CROSSBOW_HOLD`, saldırgansa `ATTACKING`, yoksa `NEUTRAL`; çağırıcı büyü
  yapıyorsa `SPELLCASTING`, kutlarken `CELEBRATING`; illusioner büyüde `SPELLCASTING`,
  saldırganken `BOW_AND_ARROW`.
- Eldeki eşya katmanı: yağmacıda hep; gözcüde saldırganken; çağırıcıda büyüde;
  illusioner'da büyüde ya da saldırganken. Kafa katmanı hepsinde.
- `WitchModel.setupAnim`: baş ve bacaklar köylü gibi; burun `sin/cos(yaş * 0,01 *
  (varlıkNo % 10))` ile kıpırdar; elinde bir şey varsa burun öne çıkıp eğilir.

## Veri

`Raider.IS_CELEBRATING` (16, boolean) ve `SpellcasterIllager.DATA_SPELL_CASTING_ID`
(17, bayt; sıfırdan büyükse büyü yapıyor) veri indeksleri `entity/variant-fields`
satırlarıyla her koşuda karşılaştırılır.

## Kapsam

`entity/illager-model` 21 satır (dört katman × iki poz, binici, boş ve dolu elle saldırı
(iki ana kol), büyü, yay, arbalet tutma, kutlama, arbalet yükü üç noktada),
`entity/witch-model` 5 satır.

## Sapmalar (açık)

- Yağmacının arbalet doldurma pozu çizilmiyor: yük, kullanılan eşyanın kaç tik
  kullanıldığını ister (`ticksUsingItem`, `CrossbowItem.getChargeDuration`); mob için
  izlenmiyor, çizimde arbalet tutma pozu görünür (T41d).
- Illusioner'ın görünmezken çizilen yansımaları ve kutlama/büyü sesleri yok.
- Cadının tuttuğu eşya (`WitchItemLayer`) çizilmiyor; yalnız burun pozu var.
- Büyü yaparken çağırıcının tuttuğu eşya ve yağmacı/gözcü silahı, genel eldeki eşya
  katmanıyla çizilir; kullanım pozları T41d'de.
