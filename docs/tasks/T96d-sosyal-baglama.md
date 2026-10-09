# T96d — Sosyal etkileşimler ekranını oyuna bağla

**Amaç:** P tuşuyla açılan ekran tıklanır ve yazılır; gizle/göster düğmesi sohbet süzgecine bağlanır.
**Bağımlılık:** T96c. **Boy:** M. [T96c](T96c-sosyal-etkilesimler.md)den ayrıldı.

## Oku
Java: `client/gui/screens/social/{SocialInteractionsScreen,PlayerEntry,SocialInteractionsPlayerList,PlayerSocialManager}.java`, `Minecraft.java` (`shouldHideMessageFrom`). Rust: `photon-render/src/gui/screen/social.rs`; `crates/photon/src/{controls,gui,play}.rs`, `controls/{pause,death,confirm}.rs` (`open_pause` örnek); sohbet günlüğü ve tablist oyuncu bilgisi.

## Yapılacaklar
- Ekran: sekme ve Done tıklaması, hide/show düğmesi (sohbete "gizlendi/gösterildi" sistem iletisi), arama kutusuna yazma ve odak (kaybedince ipucu), liste kaydırma ve tekerlek, P tuşu ile kapanma (arama odakta değilken), oyuncu girişi/çıkışı (`onAddPlayer`/`onRemovePlayer`).
- Oyun: `keySocialInteractions` ile açma; gizli oyuncuların sohbet iletileri `shouldHideMessageFrom` ile süzülür; tablist'ten oyuncu listesi.
- Oracle: tıklama/yazma adımları (`click`, `type`), kaydırma satırı.

## Kapsam dışı
Raporlama ekranları (çevrimiçi servis), engelleme (çevrimiçi mod), düğme ipuçları.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Belgeler
`docs/gui-oracle.md` satırını güncelle.

## Devir notu
(boş)
