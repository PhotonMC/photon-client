# Q01 — Public repo hijyeni ve CI

**Amaç:** Depoyu public'e hazırlamak: izlenen gereksiz dosyalar, sabit araç zinciri, sürekli entegrasyon ve bağımlılık denetimi.
**Bağımlılık:** –. **Boy:** S.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §7
- `.gitignore`, `Cargo.toml`, `clippy.toml`

## Yapılacaklar
- `.DS_Store` Git'ten silinir, `.gitignore`'a eklenir.
- `rust-toolchain.toml` (kullanılan kararlı sürüm, `rustfmt`, `clippy` bileşenleri).
- `.github/workflows/ci.yml`: `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo build`, referans/asset gerektirmeyen testler. Referans verisi veya türetilmiş çıktı CI'ya yüklenmez; asset isteyen testler `VOXEL_ASSETS` yoksa atlanır ve bunu söyler.
- `deny.toml` + CI'da `cargo deny check` (lisans, güvenlik tavsiyesi, yinelenen sürüm).
- LICENSE dosyası: türü kullanıcıya sorulur (öneri MIT/Apache-2.0 ikili); karar yoksa bu adım açık bırakılır.

## Kabul
CI yeşil; `git ls-files | grep -i ds_store` boş.

## Kapsam dışı
Kod değişikliği (yalnız test atlatma koşulu).

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
