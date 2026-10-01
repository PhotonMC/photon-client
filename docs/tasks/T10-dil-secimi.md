# T10 — `LanguageSelectScreen` + dil yeniden yükleme

**Amaç:** Dil listesi ekranı; seçimde `Lang` yeniden yüklenir (Türkçe dahil), `options.txt` `lang` anahtarına yazılır.
**Bağımlılık:** T06. **Boy:** L (sığmazsa "yazı tipi sağlayıcıları" kısmını böl).

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/screens/options/LanguageSelectScreen.java` (185), `.data/minecraft/source/net/minecraft/client/resources/language/{LanguageManager,ClientLanguage,LanguageInfo}.java`, `.data/minecraft/source/net/minecraft/client/gui/components/AbstractSelectionList.java` (liste kalıbı T07'de), yazı tipi: `assets/minecraft/font/default.json` ve `include/*.json` (`grep -n "providers"`), `.data/minecraft/assets/jar/assets/minecraft/lang/` (tr_tr.json var mı bak), Java `Language` kullanımı.
- Rust: `crates/voxel-game/src/text.rs` (`Lang::load`, çok dilli yükleme/`fallback en_us`), `crates/voxel-render/src/gui/font.rs` (hangi sağlayıcılar yüklü; accented/nonlatin?).

## Yapılacaklar
1. `Lang` dil seçimli yüklenir: seçili dil + `en_us` yedek (Java `ClientLanguage.loadFrom`); `LanguageInfo` listesi `pack.mcmeta` yerine `languages` kaydından (jar `assets/minecraft/lang/*` + `languages.json` varsa; yoksa Java'nın `LanguageManager.getLanguages` yolunu oku, yalnız mevcut dosyalarla).
2. `LanguageSelectScreen`: liste (`OptionsList` benzeri), seçili vurgu, Done (`options.language.title`, "language.name (language.region)"), "force unicode" düğmesi etkisiz.
3. Seçimde: tüm ekranlar yeniden kurulur; sohbet satırları (çevrilebilir bileşen) yeni dille yeniden çözülür **(sohbet geçmişinin yeniden sarılması)**.
4. Türkçe glifler `font/default.json` bitmap sağlayıcısıyla mı geliyor kontrol et; eksikse (unifont/`accented`) bu görevde yalnız ekran ve dosya yükleme tamamlanır, glif eksiği `remaining-work.md`'ye yazılır.

## Oracle (G)
`language_screen` düzen operation'ı (gerçek `LanguageSelectScreen`, birkaç dille: en_us, tr_tr, de_de). Metin çözümü için ayrı `lang` küçük satırları (`Language.getOrDefault` ile 5–10 anahtar, iki dil): Rust `Lang` aynı sonucu vermeli.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
`unifont` yedeği, sağdan sola metin.

## Belgeler
`docs/gui-oracle.md`, `docs/remaining-work.md` (yazı tipi kalanı).

## Devir notu
(boş)
