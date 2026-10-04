# T36f — Mutlu ghast, koşumu ve nautilusun eyeri ve zırhı — [client-parity]

Bu kayıt, mutlu ghast'ın (yetişkin, bebek) ve koşum katmanının modelinin ve nautilusun eyer ve zırh
modellerinin Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir: modeller gerçek
`HappyGhastModel`, `HappyGhastHarnessModel`, `NautilusSaddleModel` ve `NautilusArmorModel`'e karşı
karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `HappyGhastModel.setupAnim`: gövdede bir eşya (koşum) varsa gövde on altıda on beşe (`0,9375`) sıkışır; dokuz dokunaç
  (gövdenin çocukları) ghast'ınkiyle aynı biçimde `0,2 sin(0,3 yaş + sıra) + 0,4` ile salınır. Katman 4 ile ölçeklenmiş
  bir ağdır; bebek 0,2375 ek ölçekle ve bir iç gövdeyle kurulur (oracle'dan yakalanan tabloda).
- `HappyGhastHarnessModel.setupAnim`: gözlük binilmezken alna (`y = 9`, `xRot = −0,7854`) oturur, biniliyken gözlerin
  önüne iner (`y = 14`, `xRot = 0`); `y` değerleri ölçeklenmiş ağın üstüne mutlak yazılır (referansın kendi
  tuhaflığı, aynen alındı).
- `HappyGhastRenderer`: gövde slotundaki eşya `SimpleEquipmentLayer` ile (`happy_ghast_body` katman türü) koşum
  modelinde çizilir, bebekte bebek koşumu; binilmesi `isVehicle`'dan okunur.
- `NautilusRenderer` / `ZombieNautilusRenderer`: eyer (`nautilus_saddle`) ve gövde zırhı (`nautilus_body`) kendi
  modellerinde (`NautilusSaddleModel`, `NautilusArmorModel`, ikisi de `NautilusModel`'in `setupAnim`'ini kullanır),
  yalnız yetişkinde çizilir (bebek için model verilmemiştir).
- Mevcut `simple_layer` (strider eyeri) üç yeni katman türüyle kullanıldı: eşyanın `equippable` bileşeni slotu ve
  varlık kaynağını, kaynak ise katmanın dokusunu verir.

## Kapsam

`entity/happy-ghast-model` 14 satır (yetişkin ve bebek dört yaşta ve koşumlu; iki koşum modeli, binilmeyen ve binilen);
`entity/nautilus-model`'e dört satır (eyer ve zırh: durgun, bakıp yüzen).

## Sapmalar (açık)

- İp katmanı (`RopesLayer`) çizilmez: bir mobun bağlı (tasmalı) olduğu bilgisi (`SetEntityLink`) istemcide yok.
  Mutlu ghast'ın kutusunun çizim için aşağı uzatılması (`getBoundingBoxForCulling`) görüş alanı elemesine bağlı, yok.
  Koşum ve eyer dokularının yolu `equipment/<ad>.json`'dan okunur (oracle'la karşılaştırılmaz); mutlu ghast dokuları
  elle yazıldı. Sesler, parçacıklar, binme denetimi yok. Canlı gözlem yapılmadı.
