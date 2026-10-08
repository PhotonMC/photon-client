# T11 — Widget ipuçları + ok tuşlarıyla odak

**Amaç:** Üzerine gelince/odaklanınca çıkan widget ipuçları (`Tooltip`) ve ok tuşlarıyla odak gezinmesi (`FocusNavigationEvent`); duraklatma menüsündeki simge düğmeleri ve seçenek düğmeleri için.
**Bağımlılık:** T05. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/components/Tooltip.java` (72), `.data/minecraft/source/net/minecraft/client/gui/components/WidgetTooltipHolder.java` (79; gecikme 500 ms/mouse hareketi), `.data/minecraft/source/net/minecraft/client/gui/navigation/{FocusNavigationEvent,ScreenRectangle,ScreenDirection,CommonInputs}.java`, `.data/minecraft/source/net/minecraft/client/gui/screens/Screen.java` (`changeFocus`, `nextFocusPath`, `keyPressed` ok kolları: `grep -n "FocusNavigation\|ARROW\|getRectangle"`), `.data/minecraft/source/net/minecraft/client/gui/components/events/ContainerEventHandler.java` (`nextFocusPathInDirection`), `.data/minecraft/source/net/minecraft/client/gui/screens/PauseScreen.java` (ipucu metinleri: `grep -n "Tooltip"`).
- Rust: `crates/photon-render/src/gui/widgets.rs` (`Widget`, odak alanı), `pause.rs` (`tab`, `pressed`), `frame.rs` (ipucu kutusu çizimi: `tooltip` yardımcıları — kap ipucu zaten var, `grep -n "tooltip" crates/photon-render/src/gui/*.rs`).

## Yapılacaklar
1. `Tooltip`: kutu çizimi mevcut kap ipucunun aynısını kullan; konum: fare/odak, `ClientTooltipPositioner` (ekran sınırı sıkıştırma). Gecikme: fare 0.5 sn durunca; klavye odağında hemen.
2. Ok tuşları: `ScreenRectangle` yön araması (`nextFocusPathInDirection`) — Java'da `PauseScreen`'de ok tuşları komşu düğmeye geçer.
3. `PauseScreen`'e ipuçlarını bağla (feedback/report vb. simge düğmesi ipuçları), `OptionsScreen`/`VideoSettingsScreen` kaydırıcı ipuçlarına (varsa).

## Oracle (G)
`pause_screen` ve `widgets` operation'larına `hover_ticks` alanı ekle (ipucu gecikmesi sonrası ertelenmiş çizim: `extractDeferredElements` çağır); yeni capture. Ok tuşu odağı için satırlara `keys` dizisi (örnek `suggestions` operation'ındaki `keys`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Anlatıcı (narration), dokunmatik.

## Belgeler
`docs/gui-oracle.md` (ipucu sınırı notu güncellenir), `docs/remaining-work.md` pause satırı.

## Devir notu
- `gui/navigation.rs` (ScreenRectangle/yön araması portu, birim testli), `gui/tooltip.rs` (`Holder` = `WidgetTooltipHolder`: sıfır gecikme, `>` kıyası), `Positioner::{Default,Menu,BelowOrAbove}` (`frame.rs`). Pause: 4 simge ipucu + Tab/ok odağı; Options: ok tuşları (kaydırıcı sağ/sol'u alırsa gezinti yok). Ayar listeleri: `Control` ipucu (`with_tip`, `cycle_tip/slider_tip/named_tip/boolean_tip`): video, sohbet, fare, kontroller (restart notu dahil).
- Oracle: `pause_screen` adımları (key/frame, `ms`), ayar ekranlarında `frame` adımı + `ms` + `extractDeferredElements`; adaptör Java'nın adım arası karesini taklit eder. Kasıtlı: hover ipucu ilk karede çıkmaz (`>` 0 ms). Metin kayması saate bağlı (aynı `ms`).
- Açık: listelerde klavye gezintisi (T11b), dil listesi, ipucu için klavye odağı listelerde. Canlı denenmedi.
