# T01 — `behavior_adapter` GUI modülüne böl

**Amaç:** `crates/photon/examples/behavior_adapter.rs` 19k satır; her GUI görevi onu açıp gezinmek zorunda (token yükü). GUI ailesini ayrı dosyaya taşı. **Davranış değişmez.**
**Bağımlılık:** yok. **Boy:** S.

## Oku
- `crates/photon/examples/behavior_adapter.rs`: `grep -n "^fn gui_\|^fn .*gui\|^fn command_\|^fn chat_\|^fn sidebar\|^fn tab_list\|^fn container_screen" ...` ile GUI'ye ait fonksiyonların aralığını bul (`gui_result` ~18638 ve çevresindeki yardımcılar). Tümünü okuma; yalnız sınırları.
- `crates/photon/Cargo.toml` (`include` zaten `/examples/**`).

## Yapılacaklar
1. `examples/behavior_adapter.rs` → `examples/behavior_adapter/main.rs` (`git mv`; Cargo `behavior_adapter/main.rs`'yi aynı adlı örnek olarak bulur, `Cargo.toml` değişmez; derlenen yol `target/release/examples/behavior_adapter` aynı kalır).
2. GUI'ye ait fonksiyonları `examples/behavior_adapter/gui.rs`'ye taşı (`mod gui;`, `gui::gui_result`). Ortak yardımcılar (`string`, `hex`, `Value` alanı okuyucular) `main.rs`'de kalır, `super::` ile çağrılır (kopya yok).
3. Dosya düzeyindeki `#![expect(... too_many_lines ...)]` yeni modülde de gerekiyorsa orada tekrar et (gerekçe aynı); gerekmiyorsa `unfulfilled_lint_expectations` uyarısı gelir, kaldır.
4. `docs/gui-oracle.md` ve `docs/behavior-runner.md` içinde `behavior_adapter.rs` yolu geçiyorsa `behavior_adapter/` yap (`grep -rn "behavior_adapter.rs" docs/*.md tools tests` ile bul; `*-conformance.md` geçmiş kayıtlarına dokunma).

## Kabul
`python3 tools/photon.py check` → `CHECK OK`, parity sayısı değişmemeli (928). Capture gerekmez (Java tarafı değişmedi).

## Kapsam dışı
Başka aileleri bölmek; adaptörde mantık değiştirmek.

## Belgeler
Yukarıdaki yol düzeltmeleri. `AGENTS.md` token bölümündeki "büyük dosyalar" listesinde `behavior_adapter.rs` → `behavior_adapter/*.rs`.

## Devir notu
- `behavior_adapter.rs` → `behavior_adapter/main.rs` + `gui.rs` (`gui_result`, `gui_ops`, `NoClipboard`, `number_format`, `scoreboard_from`); `menu_named`, `string`, `command_*_result` `main.rs`'de, `super::` ile çağrılıyor.
- `include_str!` yolu bir seviye derinleşti (`../../../../tests/...`); yeni fixture include'ları buna göre yazılır.
- `gui.rs` kendi `#![expect(clippy::too_many_lines)]`'ini taşır (`cognitive_complexity` gerekmedi).
- Davranış değişmedi: `CHECK OK`, parity 928. Açık kalan yok.
