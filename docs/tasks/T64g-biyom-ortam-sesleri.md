# T64g — Biyom ve mağara ortam sesleri

**Amaç:** `BiomeAmbientSoundsHandler` (`minecraft:audio/ambient_sounds`: döngü çapraz geçişi 40 tik, ek sesler, mood/mağara sesi). Olay/eşya kullanım sesleri, ekran ve ejderha müziği [T64h](T64h-olay-sesleri-ekran-muzigi.md)'dedir.
**Bağımlılık:** T64f. **Boy:** M.

## Oku
Java: `client/resources/sounds/BiomeAmbientSoundsHandler.java`, `client/Minecraft.java` (`getSituationalMusic`). Rust: `crates/photon/src/music.rs` (öznitelik yükleyici), `crates/photon-world/src/local_player/{underwater,sounds}.rs`, `crates/photon/src/sound.rs` (`handle_underwater` kalıbı).

## Kabul
`CHECK OK`.

## Devir notu
Yapıldı: `photon-game/sounds/ambient.rs` (öznitelik okuyucu, `BiomeAmbient`, `LoopFade`; birim testli), `photon/ambient.rs` (biyom > boyut özniteliği, tik ve ses sistemi köprüsü), `SoundSystem::play_ambient_*`. Oracle yok: Java işleyicisi düzey rastgelesini tohumsuz kullanır ve ekran/seviye sahteleri gerektirir; bu dilim `client-parity` değil birim testle sınanan uyum kaydıdır, canlı kulakla dinlenmedi. Sapma: boyut özniteliği oturum başında bir kez okunur (boyut değişimi yok). Açık: ortam özniteliği sistemi (zaman çizelgesi/hava değiştiricileri), T64h.
