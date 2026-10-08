# T91b — Bildirim kutuları (toast)

**Amaç:** `ToastManager` ve türleri: ilerleme (`AdvancementToast`), tarif (`RecipeToast`), sistem
(`SystemToast`), öğretici (`TutorialToast`); kuyruk, kayma animasyonu, süre ve sesler.
**Bağımlılık:** T91. **Boy:** M.

## Oku
- Java: `client/gui/components/toasts/*.java`, `ClientAdvancements.update` (toast tetikleri),
  `ClientRecipeBook` (tarif toastı).
- Rust: `crates/photon-render/src/gui/hud.rs` (katman sırası), ses tetikleri (`crates/photon/src/sound.rs`).

## Yapılacaklar
1. Yönetici: görünür yuva sayısı, kuyruk, zaman çizelgesi.
2. Dört tür toast'ın çizimi.
3. G tarifi: `toasts`.

## Oracle
G (zaman sabitlenmiş kareler).

## Kabul
`CHECK OK`.

## Kapsam dışı
Anlatıcı.

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
