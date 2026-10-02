# T15 — Sohbet taslağı, seçici izin düzeyi

**Amaç:** Esc ile kapanan sohbet taslağını sakla ve yeniden açınca geri getir; yetkisiz oyuncuda `@` seçici önerme/kabul davranışını Java ile eşitle.
**Bağımlılık:** yok. **Boy:** S.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/screens/ChatScreen.java` (340: `ChatScreen.draft`, `onClose`/`removed`, `init` taslak yükleme: `grep -n "draft\|ChatComponent.Draft"`), `.data/minecraft/source/net/minecraft/client/gui/components/ChatComponent.java` (`saveAsDraft`, `Draft`, `extractRenderState` gri italik taslak), `.data/minecraft/source/net/minecraft/client/gui/components/CommandSuggestions.java` (`hasPermission`/seçici: `grep -n "permission\|@"`), `.data/minecraft/source/net/minecraft/commands/arguments/selector/EntitySelectorParser.java` (`allowSelectors`, `hasPermissions`), `.data/minecraft/source/net/minecraft/commands/arguments/EntityArgument.java` (`listSuggestions` izin kontrolü).
- Rust: `crates/voxel-render/src/gui/chat.rs` (`ChatScreen`, `show`), `crates/voxel-game/src/commands/{suggest,arguments,reader}.rs` (`entities()`), oracle `command_suggest` satırları (`tools/behavior/cases.py` `gui_command_suggest_cases`).

## Yapılacaklar
1. Taslak: Esc'te metin boş değilse `ChatComponent.draft` (metin + tür: sohbet/komut) sakla; `T` ile yeniden açınca doldur; gönderince/boş bırakınca temizle. Çizim tarafında Java hangi durumda "taslak" gösteriyor (`chat.draft` gri italik) okuyup uygula; yalnız davranış, çizim oracle'da yoksa `chat` operation'ına `draft` satırı ekle.
2. Seçici izin düzeyi: seçiciler (`@a…`) için `PermissionSet` kontrolü (`Permissions.COMMANDS_ENTITY_SELECTORS`); istemci izin düzeyini `EntityEvent`'ten (`24..28` op düzeyi) alıyor mu bak (`grep -n "permission" crates/voxel-net/src/connection/play.rs`); yoksa `LocalPlayer` izin düzeyi paketini ekle. Yetkisiz oyuncuda `@` öneri listesinde yok, ayrıştırma `Selector not allowed` hatası verir.
3. Bakılan varlık UUID önerisi T26b'ye kalır; `remaining-work.md`'de duruyor.

## Oracle
`command_parse`/`command_suggest` operation'ına `permissions` alanı (Java `ClientSuggestionProvider`/`CommandSourceStack` benzeri sahtede `hasPermission`); satırlar: izinli/izinsiz `@`, `@a[`, ad. **Yeni capture.**

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Bakılan varlık UUID'si, imzalı mesaj zinciri.

## Belgeler
`docs/remaining-work.md` M5 ("seçici izin düzeyi", "taslak" satırları) ve `docs/gui-oracle.md`.

## Devir notu
- Yapıldı: taslak (`ChatDrafts`/`ChatMethod`/`ExitReason`, `ChatScreen.as_draft`: Backspace siler, düzenleyince biter, gri italik çizim; `saveChatDrafts` seçeneği `options.txt` ve sohbet ayarları düğmesine bağlı; ölümde taslak kalır), seçici izni (entity olayı 34, işlem düzeyi 24–28, `PlaySession::can_use_selectors`, `CommandTree::parse(.., selectors)`, `ScoreHolder` argüman türü, öneri bağlamı).
- Oracle: `chat_draft` (gerçek ChatScreen/ChatComponent) ve `command_parse/command_suggest` `selectors: false` satırları; hepsi geçti.
- Sapma: Enter ile boş gönderim artık `Done` (Java'da DONE nedeni); bağlantı kopunca açık kutunun metni taslağa yazılmaz (Java'da ne olduğu doğrulanmadı). Komutla başlayan taslağın çizimi oracle'da yok (ağaç gerekir).
- Açık: yok (bakılan varlık UUID önerisi T26b'de yapıldı).
