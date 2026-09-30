# Kodlama standartları

Bu belge Rust kodunun nasıl düzenleneceğini, adlandırılacağını ve hangi karmaşıklık sınırlarına uyacağını tanımlar. Davranış kuralları [AGENTS.md](../AGENTS.md) ve [davranış testi belgesindedir](behavior-testing.md); burada yalnızca kodun biçimi ve yapısı vardır. Temel kaynaklar Rust API Guidelines, `rustfmt`'in varsayılan stili, Clippy'nin varsayılan lint seti ve YAGNI/KISS/DRY ilkeleridir.

## Crate katmanları

Workspace altı kütüphane crate'i ve uygulama crate'i `voxel`'den oluşur; hepsi `crates/` altındadır, kökte kaynak kodu yoktur. Bağımlılık yalnızca aşağıya doğru akar; bir crate kendinden sonraki bir crate'i bilemez, döngü kurulmaz.

| Crate | Sorumluluk |
| --- | --- |
| `voxel-core` | Java semantiğine uyan temel tipler ve codec'ler: matematik, AABB, şekiller, wire/binary/NBT/JSON/UTF-8 okuma-yazma, kimlikler, rastgelelik, zamanlayıcı. |
| `voxel-data` | Oracle'dan üretilmiş tablolar ve onların elle yazılmış arama fonksiyonları. Başka mantık taşımaz. |
| `voxel-world` | İstemcinin dünya kopyası: chunk'lar, blok durumları, ışık, varlıklar, yerel oyuncu ve hareket. |
| `voxel-game` | Oyun kuralları: eşyalar, bileşenler, envanter ve yaratıcı menüleri, tooltip, saldırı, seçenekler. |
| `voxel-net` | Protokol, paketler, bağlantı ve oturum (`PlaySession`). |
| `voxel-render` | Vulkan arka ucu, model/atlas/mesh, gökyüzü ve hava, arayüz (`gui`). |
| `voxel` | `src/`: uygulama (`voxel` ikilisi); oyun döngüsü, pencere olayları, betikler. `examples/`: probe'lar ve davranış adaptörü. |

Yeni kod, sorumluluğunun ait olduğu en alttaki crate'e konur. Bir tip başka crate'e taşınırsa `pub(crate)` alanlar ya metoda ya da `pub`'a çevrilir; başka crate'in tipine inherent `impl` yazılamayacağı için yardımcı serbest fonksiyon olur.

## Modül düzeni

- Bir modül tek bir sorumluluk taşır ve adı o sorumluluktur (`sections`, `controls`, `atmosphere`). `utils` veya `helpers` gibi toplama modülleri açılmaz.
- Modül belgesi (`//!`) modülün ne olduğunu bir iki cümleyle söyler; referansta karşılığı varsa Java sınıfının adını verir.
- Arayüz kodu `voxel-render::gui` altındadır: çizim (`frame`, `atlas`, `font`, `items`), HUD (`hud`), metin kutusu (`edit_box`) ve ekranlar (`screen`).
- Birden fazla modülün paylaştığı bir yardımcı, en alttaki ortak crate'e ve oradaki doğru modüle konur. Yeni bir yardımcı yazmadan önce aynısının var olup olmadığına bakılır: sabit boyutlu okuma `voxel_core::wire::take`, JSON dosyası `voxel_core::json::read_object`, ARGB `voxel_render::tint`, Java `Math` kuralları `voxel-world` içinde `java_math`.

## Adlandırma ve yorumlar

- Rust adlandırma kuralları uygulanır: tipler `CamelCase`, fonksiyon ve değişkenler `snake_case`, sabitler `SCREAMING_SNAKE_CASE`. Kısaltma yerine tam kelime kullanılır.
- Referanstaki bir metodun karşılığı olan fonksiyonun belgesi o metodu ters tırnakla adlandırır (`AbstractContainerMenu.doClick`). Orijinal kod kopyalanmaz; davranış kendi cümlelerimizle anlatılır.
- Yorum kodun ne yaptığını tekrar etmez; neden öyle olduğunu, hangi referans davranışını koruduğunu veya birimleri (GUI birimi, tick, bit deseni) söyler.
- Kod içi yorumlar İngilizce, belgeler ve commit mesajları Türkçedir.

## Karmaşıklık sınırları

Sınırlar kökteki `clippy.toml` ve `Cargo.toml` içindeki `[workspace.lints.clippy]` ile her crate'e uygulanır:

| Ölçü | Sınır | Lint |
| --- | --- | --- |
| Bilişsel karmaşıklık | 25 | `clippy::cognitive_complexity` |
| Fonksiyon uzunluğu | 100 satır | `clippy::too_many_lines` |
| Parametre sayısı | 7 | `clippy::too_many_arguments` |
| Tip karmaşıklığı | 250 | `clippy::type_complexity` |

Sınırı aşan kod önce bölünür: tek sorumluluklu yardımcı fonksiyon, durumu birlikte taşıyan bir yapı (`SectionMeshes`, `Controls`) veya parametreleri toplayan bir bağlam yapısı (`ScreenDraw`, `FrameView`). Bölmenin okunurluğu bozduğu yerlerde istisna `#[expect(lint, reason = "...")]` ile yazılır, `allow` kullanılmaz: `expect`, kod düzeltildiğinde uyarı vererek işaretin kaldırılmasını ister. Kabul edilen gerekçeler:

- Referans metodunu satır satır karşılaştırmak için bütün tutulan fonksiyon.
- Vulkan nesnelerinin API'nin istediği sırayla kurulması veya bir komut tamponunun çizim sırasıyla kaydı.
- Paket veya bileşen kimliği başına bir kolu olan dağıtım `match`'i.
- Baştan sona tek akış olarak okunan test, probe ve adaptör senaryoları (dosya düzeyinde `#![expect]`).

## Üretilen tablolar

- Oracle'dan yakalanan tablolar `crates/voxel-data/src` altındadır ve her biri `tools/generate/` içindeki bir üretici tarafından yazılır; dosyanın başındaki belge üreticinin adını verir. Üretilen dosya elle düzenlenmez.
- Tabloyu okuyan elle yazılmış kod (`registry`, `shape`, `mod.rs` içindeki aramalar) üretilen dosyadan ayrı tutulur.
- Ortak biçim `tools/generate/common.py` içindedir: float'lar referans bitlerine birebir dönen en kısa ondalık yazımla, baytlar bayt dizisi olarak, tekrar eden değerler tek listeye indekslenerek yazılır.
- Bütün tablolar `python3 tools/generate/all.py <capture>` ile yeniden üretilir ve her parity koşusunda davranış adaptörü üzerinden baştan sona karşılaştırılır.

## Davranışı koruma

- Sayısal kod referansın işlem sırasını ve tiplerini korur: `f32`/`f64` dönüşümleri aynı yerde yapılır, toplama sırası değiştirilmez; bit eşitliği gereken yerde karşılaştırma bitlerle yapılır.
- Java'nın NaN ve `-0.0` kuralları Rust'ınkinden farklıdır. NaN veya işaretli sıfır gelebilen yerde `f64::min`/`max` yerine `java_math` ve `voxel_core::math` içindeki yardımcılar kullanılır.
- Tamsayı taşması her profilde denetlenir; kasıtlı sarma `wrapping_*` ile yazılır.
- Yeniden düzenleme davranış değiştirmez. Bir düzenlemenin davranışı değiştirip değiştirmediği şüpheliyse değişiklik yapılmaz, fark kayda geçirilir.

## Hata yönetimi

- Hata olabilecek işlem `Result` döner ve `?` ile yayılır. Katmanlar arası hata dönüşümü `From` ile yazılır (`wire::ReadError` → `binary::ReadError`); aynı eşleme fonksiyonu modül modül tekrar edilmez.
- `expect` yalnızca daha önce denetlenmiş bir değişmez için ve bunu söyleyen bir mesajla kullanılır; `unwrap` üretim kodunda kullanılmaz.

## Arayüz ekranları

Bir kap ekranı `voxel_render::gui::screen::Screen` trait'ini uygular. Yerleşim, vurgu, arka plan ve tooltip `ContainerLayout`'tadır; fare ve sürükleme durumu `ContainerMouse`'tadır; çizim `ScreenDraw` bağlamıyla yapılır. Ekranın istediği tıklamalar `ScreenAction` olarak döner ve uygulama bunları oturuma iletir. Yeni bir ekran bu parçaları kullanır, kendi kopyalarını yazmaz.

## Doğrulama

Her değişiklik şu adımlardan geçer; uyarı sayısı sıfırdır:

1. `cargo fmt --all`
2. `cargo build --release --workspace --all-targets`
3. `VOXEL_ASSETS=$PWD/.data/minecraft/assets/jar/assets cargo test --release --workspace --all-targets`
4. `cargo clippy --release --workspace --all-targets`
5. [Davranış runner'ı](behavior-runner.md) ile parity karşılaştırması.
