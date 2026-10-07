# T32 — Kurt — [client-parity]

Bu kayıt, kurdun (yetişkin ve bebek) modelinin Java 26.2 ile Rust'ta aynı köşe konum, doku koordinatı
ve normali üretmesini; kurdun kendi istemci durumunun (ilgi eğimi, silkelenme, kuyruk açısı, öfke,
oturma, evcil bayrağı) aynı ilerlemesini; çeşit dokularının ve tasma boyası renklerinin Java ile aynı
tablodan gelmesini kapsar. Kanıt sınıfı `client-parity`'dir (model, tik ve tablolar) ve canlı gözlemdir:
gerçek `AdultWolfModel` ve `BabyWolfModel` katman tablosundan pişirilip gerçek `setupAnim` ile pozlanır;
kurdun tiki gerçek `Wolf` varlığıyla ve gerçek veri/olay paketleriyle karşılaştırılır.

## Java kaynak incelemesi

- `WolfModel.setupAnim`: öfkeliyken kuyruk yatayda 0, değilse yürüyüşle sallanır; oturuyorsa
  `setSittingPose` (gövde, kuyruk, arka ve ön bacaklar `ageScale` ile ölçeklenen kaymalar, yetişkinde
  üst gövde; bebekte gövde bir de `π/2` geri döner), değilse bacaklar çaprazlama sallanır; sonra
  `shakeOffWater`: gövde, yetişkinde kafa/üst gövde/kuyruk alt parçaları (`real_head`, `upper_body`,
  `real_tail`), bebekte kafa ve kuyruk `getBodyRollAngle(kayma)` kadar yana yatar; kafa bakışı; kuyruk
  `tailAngle` kadar kalkar.
- `Wolf.getTailAngle`: öfkeliyse 1,5393804; evcilse `(0,55 − hasarOranı·0,4)·π` (hasar oranı can ve en çok
  can özniteliğinden); değilse `π/5`.
- `Wolf.tick` (istemci): ilgi açısı her tikte hedefe `0,4` oranında yaklaşır; silkelenirken `shakeAnim`
  tikte `0,05` artar ve önceki değer ikiye varınca biter. Olay 8 silkelenmeyi başlatır, 56 iptal eder;
  `die` sıfırlar. `getHeadRollAngle = lerp(ilgi) · 0,15 · π`.
- `Wolf.getTexture`: çeşidin yetişkin/bebek varlıkları; evcilse evcil doku, değilse öfkeliyse öfkeli, yoksa
  yabani. Çeşit kayıt defterindeki yerle gelir (`DATA_VARIANT_ID` 23, serileştirici 25); verilmemişse
  `pale`. `WolfCollarLayer`: evcil ve görünürken `wolf_collar[_baby]` dokusu boya rengiyle çarpılarak
  aynı modelle çizilir (`getTextureDiffuseColor`).
- Veri: `TamableAnimal.DATA_FLAGS_ID` 18 (1 oturma, 4 evcil), `Wolf.DATA_INTERESTED_ID` 20,
  `DATA_COLLAR_COLOR` 21 (kırmızı varsayılan), `DATA_ANGER_END_TIME` 22 (`NeutralMob.isAngry`: bitiş oyun
  zamanından sonra; arı ile ortak `ClientEntity::is_angry`).

## Tablolar

`entity/wolf-variants` (dokuz çeşit × altı doku) ve `entity/sheep-colors`'a eklenen `texture-dye-*` (on
altı boya) satırları Java'dan yakalanır; `tools/generate/wolf_variants.py` ve `sheep_colors.py` Rust
tablolarını üretir, her koşuda baştan sona karşılaştırılır. Sürüm değişince `tools/generate/all.py` ile
yenilenir.

## Kapsam

`entity/wolf-model` 22 satır (yürüyüş, öfke, hasarlı evcil, oturma, ilgi, silkelenme); `living-entity-animations`'a
yedi kurt satırı (yabani, evcil oturan, hasarlı evcil, öfkelenen, ilgilenen, silkelenen, iptal edilen);
`entity/variant-fields`'a beş kurt satırı. Silkelenmenin 0,4'ten sonrası sıçrama parçacığı atar ve başsız
istemci parçacık ekleyemez: tam döngü `voxel-world` birim testindedir.

## Sapmalar (açık)

- Kurdun suda ya da yağmurda olması (`isInWaterOrRain`) izlenmiyor: ıslak kurt hiç yoktur, `wetShade`
  (koyulaşma) çizilmez, silkelenme yalnız sunucu olayıyla başlar.
- Silkelenme sesi, sıçrama parçacıkları ve kurdun ses/parçacık rastgele çekimleri yok.
- Kurt zırhı (`WolfArmorLayer`, gövde zırhı eşyası) çizilmiyor (ekipman görevi).
