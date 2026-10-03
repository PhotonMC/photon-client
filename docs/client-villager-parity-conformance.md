# T34 — Köylü ve gezgin tüccar — [client-parity]

Bu kayıt, köylünün ve gezgin tüccarın modelinin (yetişkin ve bebek, şapkalı ve
şapkasız) Java 26.2 ile Rust'ta aynı köşe konum, doku koordinatı ve normali
üretmesini; tür/meslek kayıt sırasının, şapka istemlerinin ve
`VillagerProfessionLayer` geçişlerinin (hangi model, hangi doku, hangi renk) aynı
olmasını kapsar. Kanıt sınıfı `client-parity`'dir: gerçek `VillagerModel` ve
`BabyVillagerModel` katman tablosundan pişirilip gerçek `setupAnim` ile pozlanır,
parça çizicisiyle çizilir ve her dörtgen bit bit karşılaştırılır; katman geçişleri
gerçek `VillagerProfessionLayer.submit`'ten (yakalayıcı bir `SubmitNodeCollector`
ile) okunur. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `VillagerModel.setupAnim`: baş bakışla döner; mutsuzken `zRot = 0,3 * sin(0,45 * yaş)`
  ve `xRot = 0,4`; bacaklar yarım genlikle çift yürür, `yRot` sıfır. Bebek model aynı
  `setupAnim`'i miras alır (parça adları aynı), yalnız gövde parçaları farklı.
- `VillagerProfessionLayer`: önce türün dokusu (bebekte `baby/` klasörü), meslek
  şapkası türün şapkasını örtüyorsa şapkasız modelle (`NONE` ya da `PARTIAL` meslek ve
  türün şapkası `FULL` değil → şapka görünür); sonra yalnız yetişkinde ve meslek
  `none` değilse mesleğin dokusu, `nitwit` değilse seviye rozeti
  (`clamp(seviye, 1, 5)` → taş, demir, altın, zümrüt, elmas). Şapka istemi doku
  dosyasının `villager` metadata bölümündedir; yoksa `NONE`.
- `VillagerRenderer`: gövde dokusu `villager`/`villager_baby`; görünmezde katman yok.
  `WanderingTraderRenderer`: tek model, tek doku, katman yok.

## Veri

- Tür ve meslek kayıt sırası ile şapka istemleri oracle'dan üretilir
  (`tools/generate/villager_registries.py`, yenilemek için `tools/generate/all.py`) ve
  `entity/villager-registries` satırlarıyla her koşuda baştan karşılaştırılır.
- Veri indeksleri `entity/variant-fields`: `Villager.DATA_VILLAGER_DATA` (19,
  serileştirici 18; tür, meslek, seviye kayıt kimlikleri) ve
  `AbstractVillager.DATA_UNHAPPY_COUNTER` (18, int). Varsayılan: ova, meslek yok, seviye 1.
  Bebek bayrağı (`AgeableMob.DATA_BABY_ID`) köylü için de okunur.

## Kapsam

`entity/villager-model` 25 satır (beş katman × beş poz), `entity/villager-layers` 432
satır (köylü ve zombi köylü klasörü; yetişkin ve bebek × tür × meslek; seviye dizisi),
`entity/villager-registries` 22 satır. Aile içi tür/meslek tablo satırıdır.

## Sapmalar (açık)

- Zombi köylü çizilmiyor (`ZombieVillagerModel` ayrı insansı model); katman mantığı
  oracle'da `zombie_villager` klasörüyle karşılaştırılır, çizime T34b'de bağlanır.
- Köylünün kafa eşyası (`CustomHeadLayer`, köylü dönüşümüyle) ve çapraz kolundaki eşya
  (`CrossedArmsItemLayer`) yok; tüccarın da.
- Ticaret ekranı, köylü sesleri ve yatakta uyuma pozu kapsam dışı.
- Katmanların dokuları sınır yüzeylerde aynı derinlikte çizilir; `GREATER_OR_EQUAL`
  derinlik testi sonradan çizileni öne alır, referanstaki sıra korunur.
