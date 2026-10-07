# T22 — Uzaktaki oyuncu çizimi

**Amaç:** Başka oyuncuyu varsayılan deriyle (Steve/Alex, profil UUID'sinden) çiz: model, kafa dönüşü, yürüme animasyonu, ad etiketi.
**Bağımlılık:** T21. **Boy:** L (sığmazsa ad etiketini ayır).

## Oku
- Rust: `crates/voxel-render/src/entity/player_model.rs`, `player_renderer.rs` (birinci kişi/üçüncü şahıs yerel oyuncu çizimi — yeniden kullanılacak), `crates/voxel-render/src/vulkan.rs` (**bütün okuma**; `grep -n "player\|third_person\|draw_entity"`), `crates/voxel-render/src/scene.rs`, `crates/voxel/src/{view,play}.rs`, T21 snapshot sorgusu, `PlaySession` oyuncu listesi (`online_player_names`, `PlayerInfoUpdate` kayıtları: `grep -n "player_info\|tab_list" crates/voxel-net/src/connection/play.rs`).
- Java: `.data/minecraft/source/net/minecraft/client/renderer/entity/player/AvatarRenderer.java` (varsa; `find .data/minecraft/source/net/minecraft/client/renderer/entity -name "*Player*Renderer*.java" -o -name "AvatarRenderer.java"`), `.data/minecraft/source/net/minecraft/client/renderer/entity/LivingEntityRenderer.java` (323), `.data/minecraft/source/net/minecraft/world/entity/WalkAnimationState.java`, `.data/minecraft/source/net/minecraft/client/model/player/PlayerModel.java` (`setupAnim`), `.data/minecraft/source/net/minecraft/client/resources/DefaultPlayerSkin.java` (UUID → varsayılan deri), `.data/minecraft/source/net/minecraft/client/renderer/entity/EntityRenderer.java` (ad etiketi: `extractNameTag`).

## Yapılacaklar
1. Oyuncu entity'leri (`AddEntity` türü player) → `PlayerInfo` ile profil eşle (UUID); deri: yalnız `DefaultPlayerSkin` (UUID hash → varsayılan deri seti; ağ deri indirme **yok**).
2. Çizim: mevcut oyuncu modeli ve renderer'ı entity konumu/dönüşü ile çağır (`partial_tick`); `WalkAnimationState` (`position`/`speed` lerp, `limbSwing`) Java ile aynı formül.
3. Ad etiketi: yazı + arka plan kutusu (`EntityRenderer.submitNameTag`), mesafe ve görünürlük kuralları (`Entity.shouldShowName`), eğilme şeffaflığı.
4. Kafa dönüşü (`RotateHead`) ve gövde dönüşü (`yBodyRot` yumuşatma `LivingEntity.tickHeadTurn` — client-parity'de var mı bak; yoksa görevde belgeli sapma).

## Oracle
Model eklem dönüşümleri için mevcut `player_model` birim/oracle kaydı (`docs/records/player-model-conformance.md`). Yürüme animasyonu için küçük oracle: `WalkAnimationState` tik dizisi → `position(partial)`, `speed(partial)` (entity ailesine satır).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ikinci istemci (`VOXEL_USER=Tester`, başka süreç) yürürken görünür; ekran görüntüsünü bir kez oku.

## Kapsam dışı
Zırh/eşya/pelerin katmanları, ağdan deri, sahibinin kendi gölgesi.

## Belgeler
`docs/remaining-work.md` M4 madde 2–3.

## Devir notu
- Yapıldı: `voxel-world/living.rs` (`WalkAnimation` = `WalkAnimationState`, yüzme geçişi, gövde dönüşü `tickHeadTurn`; yerel oyuncu ve uzak oyuncu aynı kodu kullanır), `ClientEntity` oyuncu pozu (`tick_player_pose`), `EntitySnapshot` genişledi (gövde yaw'ı, yürüme konumu/hızı, yüzme, poz), `walk_animation` oracle'ı gerçek `WalkAnimationState`'e karşı, varsayılan derilerin tek atlası (`skin_atlas`, `atlas_uv`: 18 deri 6×3 hücre), `overlays::remote_players`, yerel oyuncu/el/envanter aynı atlastan.
- Canlı: iki istemciyle (`Tester9` + `Fresh12`) uzak oyuncu Steve derisiyle platformda görünüyor (ekran görüntüsü okundu).
- Sapma: ad etiketi yok (T22b); kol sallama (`Animate`) yok; suda eğilme için `in_water` hep false (yüzme pozu kuyruğa tek başına yetmez); profil derisi yok (varsayılan deri); kamera dışı/frustum ayıklama yok; ışık göz konumundan tek örnek.
- Açık: T22b, `Animate` paketi, uzak oyuncu zırhı/eşyası.
