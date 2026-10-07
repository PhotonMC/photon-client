# T85 — Uygulama durumu ve başlık ekranı

**Amaç:** Uygulama bugün `voxel play <adres>` ile doğrudan bağlanıyor. Java'nın akışı: pencere açılır,
`TitleScreen`, oradan bağlanılır, bağlantı kopunca/çıkınca başlığa dönülür. Bu görev uygulamayı
menü ↔ oyun durum makinesine çevirir ve `TitleScreen`'in düğme düzenini (logo, splash yazısı,
sürüm/telif satırları) oracle'a eşitler. Panorama T85b'dedir; bu görevde arka plan düz karartmadır.
**Bağımlılık:** — . **Boy:** L.

## Oku
- Java: `client/gui/screens/TitleScreen.java` (`init`, `createNormalMenuOptions`, `extractRenderState`),
  `client/gui/components/LogoRenderer.java`, `client/gui/components/SplashRenderer.java`,
  `client/resources/SplashManager.java`, `client/Minecraft.java` (`setScreen`, `disconnect`, başlığa dönüş).
- Rust: `crates/voxel/src/main.rs` (`play` alt komutu), `crates/voxel/src/play.rs` (`run_play`),
  `crates/voxel/src/gui.rs` (ekran yığını), `crates/voxel-render/src/gui/screen/pause.rs` (kalıp).

## Yapılacaklar
1. `voxel` argümansız açılınca başlık ekranı; `play <adres>` geliştirici kısayolu olarak kalır.
2. Durum makinesi: başlık → (T86/T87 gelene dek) sabit adrese bağlan → oyun; `Disconnect`/çıkış → başlık.
3. `TitleScreen` düzeni: Singleplayer (devre dışı, Java'daki gibi uyarı ipucu yok — kaynağa bak),
   Multiplayer, Realms (devre dışı), Options, Quit, dil ve erişilebilirlik simge düğmeleri.
4. Splash: `texts/splashes.txt`'den Java'nın seçim kuralı (tarih özel günleri dahil), sarı, eğik, nabız.
5. G tarifi: `title_screen` operation'ı (seçili splash sabitlenir).

## Oracle
G: `GuiOracle` `titleScreen` — düğmeler, logo ve splash çizimi bit eşit.

## Kabul
`python3 tools/voxel.py check --capture` → `CHECK OK`; `voxel` açılınca başlık görünür (`shot`).

## Kapsam dışı
Panorama (T85b), sunucu listesi (T86), Realms, tek oyunculu.

## Belgeler
`docs/live-play-session.md` başlatma paragrafı, `docs/gui-oracle.md` satırı.

## Devir notu
