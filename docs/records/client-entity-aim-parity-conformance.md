# T26 — Entity hedefleme, `Attack`/`Interact`/`SpectatorAction` — [client-parity]

Bu kayıt, oyuncunun nişangâhının neye indiğini (blok mu, entity mi) ve sol/sağ
tıkın entity hedefinde hangi paketleri gönderdiğini Java 26.2 ile Rust
istemcisinin aynı girdide aynı üretmesini kapsar. Kanıt sınıfı `client-parity`'dir:
aynı sahne Java'nın gerçek `ClientLevel`'ında, gerçek `LocalPlayer`'ı
(`raycastHitResult`) ve gerçek `Minecraft.startAttack`/`startUseItem`'iyle
koşturulur; gönderilen her paket Java'nın kendi codec'inin yazdığı baytlarla
karşılaştırılır. Canlı sunucu kabulü (`Attack` paketi domuzun canını 10 → 9
yaptı) ayrıca `server-conformance` gözlemidir, eşitlik kanıtı sayılmaz.

## Java kaynak incelemesi

- `client/player/LocalPlayer.java:1242-1291`: `raycastHitResult` önce aktif
  eşyanın `ATTACK_RANGE` bileşenine bakar (taşınmadı); yoksa `pick`: azami
  mesafe blok ve entity menzilinin büyüğü, blok isabeti (anahat) bulununca
  mesafe ona kısalır, entity yalnız blok isabetinden **daha yakınsa** seçilir,
  sonuç kendi menziline göre süzülür (`closerThan`, kesin küçüktür); süzülen
  entity isabeti, arkadaki bloğa düşmeden, ıska olur.
- `world/entity/projectile/ProjectileUtil.java:109-145`: kutu `getPickRadius`
  kadar şişirilir; `from` kutunun içindeyse en yakın sayılır (`nearest = 0`,
  `canBePickedFromInside`); aynı kök araçtaki entity yalnız `nearest == 0`
  iken alınır; eşit uzaklıkta ilk gelen kalır.
- `world/entity/player/Player.java:1992-2008`: menziller öznitelik değeridir
  (`block_interaction_range` 4.5, `entity_interaction_range` 3.0; yaratıcı
  kipte sunucunun `ADD_VALUE` değiştiricileri +0.5/+2.0 `UpdateAttributes` ile
  gelir). `isWithinEntityInteractionRange`: kutuya göz uzaklığı, kesin küçüktür.
- `client/Minecraft.java:1613-1685` `startAttack`: izleyici kipinde
  `spectate(entity)`/`spectatorNoAction` ve vuruş sona erer; entity hedefinde
  `attack` (paket + `resetAttackStrengthTicker`), ardından kol sallama.
  `:1687-1760` `startUseItem`: her el için entity hedefinde dünya sınırı
  denetimi (dışındaysa tüm eylem biter), menzil içindeyse `interact`, sonuç
  başarılı değilse elin eşyası `useItem` ile denenir.
- `client/multiplayer/MultiPlayerGameMode.java:426-446`: `ServerboundAttack`
  (kimlik), `ServerboundInteract` (kimlik, el, entity'ye göre isabet noktası
  `LpVec3` ile, sneaking), `ServerboundSpectatorAction` (`OptionalInt`:
  değer + 1, boşsa 0).

## Veri tablosu

`Entity.isPickable`, `getPickRadius`, `canBePickedFromInside`, duruş
pozundaki ve (bebeği olan türlerde) bebek boyutu her entity türü için
oracle'dan yakalanır (`movement/entity-pick-table`, `MovementOracle.entityPickTable`:
her tür gerçek `EntityType.create` ile yapılır), `tools/generate/entity_picks.py`
ile `photon-data/src/generated/entity_picks.rs`'e yazılır ve **her parity
koşusunda baştan sona karşılaştırılır**. Sürüm değişince yenilemek için yakalama
sonrası `python3 tools/generate/entity_picks.py <capture>/outputs.jsonl`.
Oyuncu entity yapmaz; kuralı (`isPickable` = izleyici değil) kodda.

## Kapsam ve dal kararı

Corpus: `entity-pick-table` (158 tür) ve `entity-aim` (27 satır). Dallar:
yakın/uzak ve sıra bağımsız en yakın entity, entity içinde başlama, blok
önünde/arkasında entity, menzil dışı ve menzil kenarı (blok varken ve yokken),
seçilemez entity (ok), `getPickRadius` ile şişen mermi (yakın ve uzak),
tekne, armor stand, yukarıdan bakış, arkaya dönük, saldırı (tek/çift/basılı),
kullanma (boş el, tek el, iki el, basılı, sneaking), yaratıcı/macera/izleyici
kipi, kılıç. Zombi ve iskelet aynı koddan geçtiği için yalnız konum
karşılaştırmasında birlikte kullanıldı; her tür için ayrı satır yok.

## Sapmalar (açık)

- `Player.interactOn`'un istemci tarafı tahmini yok: Java'da bazı entity'ler
  istemcide `Success`/`Consume` döner (yemlenen hayvan, çerçeve, köylü); Rust
  her `Interact` sonrasını geçilmiş sayıp elin eşyasını `useItem` ile dener ve
  kol sallamayı sunucuya bırakır. Boş el ve eşyanın `interactLivingEntity`'si
  etkisiz olan satırlar eşleşir; yenebilir eşyayla hayvana sağ tık ayrışabilir.
- Boyutlar yalnız türün başlangıç (ve bebek) boyutudur: `SCALE` özniteliği,
  eğilme/yüzme/uyku gibi poz değişimleri (oyuncu dışındaki mob'lar) ve
  armor stand `marker` bayrağı, sulfur cube gövde eşyası taşınmadı.
- Eşyanın `ATTACK_RANGE`/`PIERCING_WEAPON` bileşenleri, `isHandsBusy` ve
  `cannotAttackWithItem` (asgari saldırı şarjı) yok (blok yolunda da yoktu).
- `Aim::Entity` istemci kopyasındaki konumu kullanır; ender ejderhası parçaları
  entity değil.
