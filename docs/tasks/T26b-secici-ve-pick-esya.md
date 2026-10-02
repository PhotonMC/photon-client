# T26b — Bakılan varlık UUID önerisi ve `PickItemFromEntity`

**Amaç:** Sohbette entity seçici argümanı yazılırken nişangâhtaki entity'nin UUID'si öneri olarak çıksın (`ClientSuggestionProvider.getSelectedEntities`); orta tık (`pickItem`) entity'ye bakarken `ServerboundPickItemFromEntity` gönderilsin.
**Bağımlılık:** T26. **Boy:** S.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/multiplayer/ClientSuggestionProvider.java` (`getSelectedEntities` 70–74), `commands/arguments/EntityArgument.java` (`listSuggestions` 138–142: yalnız `!playersOnly` iken eklenir), `client/Minecraft.java` (`pickItem`/`pickBlockOrEntity`: `grep -n "PickItemFromEntity\|pickBlock"`), `network/protocol/game/ServerboundPickItemFromEntityPacket.java`.
- Rust: `crates/voxel-game/src/commands/suggest.rs` (`entities`, `SuggestContext`), `crates/voxel-render/src/gui/chat.rs:929` (bağlam kurulumu), `crates/voxel-game/src/pick.rs`.

## Yapılacaklar
1. `SuggestContext`'e `selected_entities` ekle; `entities` argümanında yalnız oyuncu-dışı seçicide öner (`playersOnly` ayrımı). Oyun tarafı `pick` sonucunu bağlama geçirir.
2. `command_suggest` oracle'ına bakılan entity'li satırlar (`GuiOracle` sahtesindeki `getSelectedEntities` doldurulur).
3. `PickItemFromEntity`: paket kimliği/kodlayıcı, orta tık dalı, yaratıcı kip dışı davranış Java'ya göre.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
İmzalı mesaj zinciri, blok için `PickItemFromBlock` (ayrı iş olarak kayıtlı değilse burada not düş).

## Belgeler
`docs/tasks/T15-sohbet-kalanlari.md` açık satırı kapanır.

## Devir notu
- Yapıldı: orta tık (`pick_key`, betikte `pick`) → `Outgoing::PickItemFromBlock/Entity` (paket 36/37; blok ve entity birlikte, Java `pickBlockOrEntity` tek işlev), ctrl ile `includeData`; `SuggestContext.selected_entities` + `PlaySession::selected_entities`, entity argümanı (oyuncu-dışı) UUID önerir; `Uuid::to_string_java`. Kayıt: `docs/client-pick-item-suggestion-parity-conformance.md`.
- Oracle: `movement/entity-aim` +9 satır (Java penceresi tuşsuz, handle 1), `gui/command-suggest` +12 satır. Canlı: yaratıcı kipte domuza orta tık, ele domuz yumurtası geldi.
- Sapma: ctrl'li `includeData` oracle'da yok (birim test); tuş eşlemesi sabit Button3.
- Açık: yok.
