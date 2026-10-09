# T85c — Menü arka planının bulanıklığı

**Amaç:** `Screen.extractBackground`'ın `blurBeforeThisStratum` çağrısı ve `GameRenderer.processBlurEffect`: ana hedef, `blur` post zincirinin altı geçişiyle (`box_blur`, yatay/dikey, yarıçap `menuBackgroundBlurriness`) bulanıklaştırılır; bulanıklıktan sonraki GUI katmanları üstüne çizilir.
**Bağımlılık:** T85b. **Boy:** M.

## Oku
- Java: `client/gui/screens/Screen.java` (`extractBackground`, `extractBlurredBackground`), `client/renderer/GameRenderer.java` (`processBlurEffect`), `client/gui/render/GuiRenderer.java` (`firstDrawIndexAfterBlur`), `client/renderer/state/gui/GuiRenderState.java` (`blurBeforeThisStratum`), `assets/minecraft/post_effect/blur.json`, `shaders/post/box_blur.fsh`.
- Rust: `crates/photon-render/src/vulkan/frame/scene.rs` (geçiş sırası), `panorama.rs` (T85b), GUI çizim listesi (`set_gui`).

## Yapılacaklar
1. GUI çizim listesini bulanıklık sınırında ikiye böl (`GuiFrame` içinde "bundan sonra bulanık" işareti).
2. Ana hedefi (swapchain görüntüsü) bir ara görüntüye kopyalayan ve `box_blur`'u altı geçişte uygulayan post geçişi; yarıçap `menuBackgroundBlurriness` seçeneğinden (varsayılan 5, `>= 1` ise).
3. Hangi menü ekranlarının bulanıklık istediğini Java'dan oku (`isInGameUi` ve `minecraft.level == null`).

## Oracle
GUI çizim oracle'ı bulanıklık işaretinin yerini kaydeder; bulanıklığın kendisi `box_blur` çekirdeğinin ve geçiş sayısının bit karşılaştırması ve `shot` ile kabul edilir.

## Kabul
`CHECK OK`; seçenekler/sunucu listesi ekranında bulanık panorama.

## Kapsam dışı
Oyun içi ekran örtüsü (`extractTransparentBackground`), `blur` dışındaki post zincirleri.

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
(boş)
