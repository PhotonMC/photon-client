# T34c — Golemler, enderman ve zombi köylü — [client-parity]

Bu kayıt, demir golemin, kar golemin, endermanın ve zombi köylünün modellerinin
Java 26.2 ile Rust'ta aynı köşe konum, doku koordinatı ve normali üretmesini; demir
golemin yürürken pozuna eklenen sallanmanın ve çatlak düzeyinin aynı olmasını;
demir golemin saldırı/çiçek zamanlayıcılarının gerçek `IronGolem` sınıfıyla aynı
ilerlemesini kapsar. Kanıt sınıfı `client-parity`'dir: gerçek `IronGolemModel`,
`SnowGolemModel`, `EndermanModel`, `ZombieVillagerModel` ve
`BabyZombieVillagerModel` katman tablosundan pişirilip gerçek `setupAnim` ile
pozlanır, parça çizicisiyle çizilir ve her dörtgen bit bit karşılaştırılır. Canlı
gözlem `server-conformance`.

## Java kaynak incelemesi

- `IronGolemModel.setupAnim`: kollar saldırıda `-2 + 1,5 * üçgenDalga(kalanTik, 10)`
  ile iner kalkar; çiçek sunarken sağ kol `-0,8 + 0,025 * üçgenDalga(tik, 70)`, sol kol
  sıfır; yoksa adımla karşıt salınır. Bacaklar `±1,5 * üçgenDalga(konum, 13) * hız`.
- `IronGolemRenderer.setupRotations`: hız 0,01'in altında değilken pozu z ekseninde
  `6,5 * üçgenDalga(konum + 6, 13)` derece sallar (ölüm düşüşünden sonra, ters çevirmeden önce).
- `Crackiness.GOLEM.byFraction`: sağlık oranı 0,75 / 0,5 / 0,25'in altına indikçe düşük,
  orta, yüksek çatlak; çatlak dokusu aynı modelle gövdenin üstüne çizilir.
- `IronGolem`: olay 4 saldırı zamanlayıcısını 10, olay 11 çiçek zamanlayıcısını 400
  yapar, olay 34 çiçeği sıfırlar; ikisi her tik bir azalır. Çizim, saldırıda kalan
  tikten kısmi tiki çıkarır.
- `SnowGolemModel.setupAnim`: baş bakışla döner, üst gövde bakışın dörtte biri kadar;
  kollar üst gövdeyle birlikte döner ve onun yönünde beş birim açılır.
- `EndermanModel.setupAnim`: insansı poz; kol ve bacak salınımı yarıya iner ve ±0,4 ile
  sınırlanır; blok taşırken kollar öne (`-0,5`, `±0,05`); ürkütülmüşken baş beş birim
  yukarı, şapka beş aşağı.
- `ZombieVillagerModel.setupAnim`: insansı poz + `animateZombieArms` (zombiyle aynı);
  zombi köylü bu yüzden insansı tablonun bir satırıdır, üstüne köylünün meslek
  katmanı (`VillagerProfessionLayer`, klasör `zombie_villager`) biner.

## Veri

`EnderMan.DATA_CARRY_STATE` (16, isteğe bağlı blok durumu: sıfır boş), `DATA_CREEPY` (17),
`ZombieVillager.DATA_VILLAGER_DATA` (20, serileştirici 18) veri indeksleri
`entity/variant-fields` satırlarıyla her koşuda karşılaştırılır. Demir golemin
zamanlayıcıları gerçek varlığın tikiyle `movement/living-entity-animations`'ta
karşılaştırılır.

## Kapsam

`entity/iron-golem-model` 9 satır, `snow-golem-model` 3, `enderman-model` 6,
`iron-golem-render` 16 (yürüyüş sallanması ve on sağlık oranı), `zombie-model`'a
zombi köylü için 6 satır, `living-entity-animations`'a beş demir golem satırı.

## Sapmalar (açık)

- Kar golemin balkabağı, demir golemin çiçeği ve endermanın taşıdığı blok T34d'de
  çizilir ([kayıt](client-entity-block-parity-conformance.md)).
- Endermanın gözleri (`EnderEyesLayer`, parlak katman) yok (T33c).
- Demir golemin can üst sınırı 100 sayılır; sunucunun değiştirdiği öznitelik mob için
  okunmuyor. Sağlık verisi gelene dek bir sağlık varsayılır (referans da öyle).
- Bakır golem yapılmadı (T34e). Zombi köylünün dönüşürken sallanması
  (`isConverting`) yok.
