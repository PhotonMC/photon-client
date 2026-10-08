# Q01 — Public repo hijyeni ve CI

**Amaç:** Depoyu public'e hazırlamak: izlenen gereksiz dosyalar, sabit araç zinciri, sürekli entegrasyon ve bağımlılık denetimi.
**Bağımlılık:** –. **Boy:** S.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §7
- `.gitignore`, `Cargo.toml`, `clippy.toml`

## Yapılacaklar
- `.DS_Store` Git'ten silinir, `.gitignore`'a eklenir.
- `rust-toolchain.toml` (kullanılan kararlı sürüm, `rustfmt`, `clippy` bileşenleri).
- `.github/workflows/ci.yml`: `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo build`, referans/asset gerektirmeyen testler. Referans verisi veya türetilmiş çıktı CI'ya yüklenmez; asset isteyen testler `PHOTON_ASSETS` yoksa atlanır ve bunu söyler.
- `deny.toml` + CI'da `cargo deny check` (lisans, güvenlik tavsiyesi, yinelenen sürüm).
- LICENSE dosyası: türü kullanıcıya sorulur (öneri MIT/Apache-2.0 ikili); karar yoksa bu adım açık bırakılır.

## Kabul
CI yeşil; `git ls-files | grep -i ds_store` boş.

## Kapsam dışı
Kod değişikliği (yalnız test atlatma koşulu).

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- `.DS_Store` izlemeden çıkarıldı, `.gitignore`'a eklendi. `rust-toolchain.toml` 1.97.1 + rustfmt/clippy.
- `.github/workflows/ci.yml`: fmt, clippy `-D warnings`, build, `cargo test --workspace --lib --bins --tests`; ayrı `deny` işi. Linux paketleri (cmake, ninja, ALSA, wayland/xkb/xorg) shaderc/glfw/cpal için eklendi.
- Boş klonda (`.data/` yok) 520 test geçti, 0 düştü: birim testler asset istemiyor, test atlatma koşulu gerekmedi. Asset isteyen işler `examples/` altında ve yerelde kalıyor.
- `deny.toml` yazıldı ama `cargo-deny` bu makinede kurulu değil; yerelde ve CI'da çalıştırılmadı. İlk CI koşusunda lisans izin listesi ve yinelenen sürüm uyarıları gözden geçirilmeli. CI'nın kendisi de GitHub'da henüz koşmadı (shaderc/glfw kaynaktan derleme süresi ve paket listesi doğrulanmadı).
- Lisans: kullanıcı MIT OR Apache-2.0 seçti. `LICENSE-MIT` (telif sahibi "photon contributors", gerçek ad kullanıcıya bırakıldı), `LICENSE-APACHE`, workspace `license` alanı ve her crate'te `license.workspace = true`.
