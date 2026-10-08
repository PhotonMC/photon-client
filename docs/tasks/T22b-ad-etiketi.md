# T22b — Uzaktaki oyuncunun ad etiketi

**Amaç:** T22'de çizilen uzaktaki oyuncuların başında ad etiketi (yazı + yarı saydam arka plan kutusu) görünsün; `Entity.shouldShowName`, mesafe ve eğilme kurallarıyla.
**Bağımlılık:** T22. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/renderer/entity/EntityRenderer.java` (`extractNameTag`/`submitNameTag`, `nameTagAttachment`), `LivingEntityRenderer.java` (`shouldShowName`, `isEntityUpsideDown`), `net/minecraft/client/renderer/state/` içinde `EntityRenderState.nameTag`/`nameTagAttachment` alanları, `net/minecraft/client/gui/Font.java` (`DisplayMode.SEE_THROUGH`/`NORMAL`, `drawInBatch`), `AvatarRenderer.java` (puan tahtası adı: `scoreText`), `net/minecraft/world/entity/Entity.java` (`shouldShowName`, `isDiscrete`), `net/minecraft/client/Options.java` (`backgroundColor`).
- Rust: `crates/photon-render/src/vulkan.rs` (yazı çizimi için ayrı boru hattı yok: bu görevde ya billboard'lı dörtgenler `entity_pipeline` ile ya da GUI atlasından örneklenen ikinci bir mesh), `crates/photon/src/overlays.rs` (`remote_players`), `crates/photon-render/src/gui/font.rs`.

## Yapılacaklar
1. Etiket yazısı: oyuncu adı (`PlayerInfo`'dan; yoksa UUID'den kısaltılmaz, çizilmez), puan tahtası altı satırı yok (kapsam dışı).
2. Konum/ölçek: `nameTagAttachment` (başın 0.5 üstü), kameraya dönük, `0.025` ölçek, mesafe < 64 blok, eğilen oyuncuda şeffaf/`SEE_THROUGH` kuralı.
3. Arka plan kutusu: `Options.getBackgroundColor(0.25)`.
4. Oracle: etiketin metin/kutu geometrisi için küçük satır (Java `Font` + `EntityRenderer` sahtesi) **veya** davranışı kaynak satırlarıyla belgeleyip yalnız kural parite testi.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`; canlı: ikinci istemci (`PHOTON_USER=Tester9`) ile `shot`, ad etiketi görünür.

## Kapsam dışı
Puan tahtası satırı, takım rengi, kafa üstü simgeler, ad etiketi kapatma seçeneği.

## Belgeler
`docs/remaining-work.md` M4.

## Devir notu
- Yapıldı: `photon-render/name_tag.rs` (kural + `submitNameTag` pozu/ışığı/renkleri + üçgenler), `glyph_quads` (`frame.rs`'ten ortak yazı dörtgenleri), `PipelineKind::Text/TextSeeThrough` + `set_name_tags` (derinlik testli ve testsiz), `overlays::name_tags`, `PlaySession::player_name`, `Quaternion::rotation_yxz`. `name_tags` oracle'ı (gui ailesi) gerçek `AvatarRenderer.extractNameTags` + `SubmitNodeStorage`'a karşı; kapsam `docs/gui-oracle.md`.
- Canlı: `Tester9` + `Viewer1` ile ad etiketi baş üstünde yarı saydam kutuyla görüldü (ekran görüntüsü okundu).
- Sapma/açık: `backgroundForChatOnly` seçeneği istemcide yok, arka plan hep 0.25 (varsayılan); F1 ile HUD gizleme istemcide yok (`hud_hidden` hep false); `NAME_TAG_DISTANCE` özniteliği varsayılan 64; takım rengi/puan tahtası satırı yok; etiket üçgenleri ve görünür/SEE_THROUGH çizimi oracle'da yok (yalnız submit kayıtları), çizim canlı ekranla doğrulandı; yazı gölgesi/kalın/italik etiketlerde kullanılmıyor.
- `tools/photon.py` içindeki `SERVER_PORT` değişikliği (25565) bu görevin değil; commit'e girmedi.
