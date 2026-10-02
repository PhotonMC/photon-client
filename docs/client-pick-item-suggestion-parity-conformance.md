# T26b — Orta tık seçimi (`PickItemFrom*`) ve bakılan varlık önerisi — [client-parity]

Bu kayıt, orta tıkın (`keyPickItem`) bakılan blok ya da entity için gönderdiği
paketleri ve sohbette entity argümanı yazılırken bakılan entity'nin UUID'sinin
öneri olarak çıkmasını kapsar. Kanıt sınıfı `client-parity`'dir: seçim Java'nın
gerçek `Minecraft.pickBlockOrEntity`'siyle, öneri Java'nın gerçek
`CommandDispatcher`/`EntityArgument` kodu ve sahte `getSelectedEntities`'iyle
karşılaştırılır. Canlı sunucu kabulü (yaratıcı kipte domuza orta tık → ele domuz
yumurtası) `server-conformance` gözlemidir.

## Java kaynak incelemesi

- `client/Minecraft.java:1946` ve `2352-2365`: seçim anahtarı saldırı ve kullanma
  tıklarından sonra, basılı kullanmadan önce harcanır; isabet ıska değilse blok
  için `handlePickItemFromBlock(pos, includeData)`, entity için
  `handlePickItemFromEntity(entity, includeData)`; `includeData` sol/sağ ctrl
  basılıdır (`hasControlDown`: tuş 341/345, macOS'ta Cmd değil).
  `client/multiplayer/MultiPlayerGameMode.java:560-566` yalnız paketi yollar:
  `ServerboundPickItemFromBlock` (blok konumu `long`, bayrak) ve
  `ServerboundPickItemFromEntity` (var int kimlik, bayrak).
- `client/multiplayer/ClientSuggestionProvider.java:70-74`:
  `getSelectedEntities` bakılan entity'nin dize UUID'sidir.
  `commands/arguments/EntityArgument.java:138-142`: yalnız oyuncu almayan entity
  argümanı isimlere bunu da ekler; `GameProfileArgument` (oyuncu adı) etmez.

## Kapsam ve dal kararı

- `movement/entity-aim` (+9 satır): entity, blok, blok önünde entity, boşluk,
  menzil dışı, iki tık, kullanma/saldırıyla aynı tikte sıra, yaratıcı ve izleyici.
- `gui/command-suggest` (+12 satır): `/t6` (entity) boş, UUID öneki, ad, `@`,
  `selectors` kapalı; yalnız-oyuncu (`/single`), oyuncu profili ve skor tutucu
  argümanlarında öneri çıkmaması.
- Ctrl basılıyken `includeData = true` oracle'da kurulamaz (Java penceresinde
  tuş basılı tutulamaz); bayrağın baytı birim testle okunur
  (`control_asks_the_pick_for_data`).

## Sapmalar (açık)

- Orta tıkın tuş eşlemesi sabit (Button3); `options.txt` yeniden eşlemesi yok.
- İzleyici ve yaratıcı kip için Java bir ayrım yapmaz (sunucu karar verir);
  istemci tarafında seçim paketi her zaman gider.
- UUID dizesi `Uuid::to_string_java` ile biçimlenir; oturum bakılan entity'yi
  son `keys` çağrısındaki hedeften alır.
