# Yerel kurulum ve çalıştırma

`tools/voxel.py` Linux, macOS ve Windows'ta aynı biçimde çalışan geliştirici
aracıdır. Yalnız Python standart kütüphanesini kullanır. Windows'ta
`python3` yerine `python` yazılabilir. İndirdiği her şey depo dışındaki
`.data/` klasörüne gider. Bu klasör Git'e girmez; dağıtılan istemci de
(`Cargo.toml` paket listesi) bu aracı ve `.data/` içeriğini taşımaz.

## Gerekenler

- Python 3.9 ya da yenisi.
- Rust (`cargo`).
  - GLFW ve shaderc kaynaktan derlendiği için CMake ve bir C/C++ derleyicisi
    de gerekiyor: Windows'ta Visual Studio Build Tools, macOS'ta Xcode
    komut satırı araçları.
- Vulkan destekleyen bir sürücü. macOS'ta MoltenVK ile Vulkan loader'ı.
- Sunucu ve decompile için Java 25. Araç sırasıyla `--java`,
  `JAVA_HOME` ve `PATH` içinde 25 sürümlü `java` arar.

## Komutlar

```sh
python3 tools/voxel.py setup
python3 tools/voxel.py server --accept-eula --background
python3 tools/voxel.py play --user Walker
python3 tools/voxel.py rcon "time set noon" "weather clear"
python3 tools/voxel.py server --stop
```

- **`setup`:** Mojang'ın resmi dağıtımından sabitlenmiş 26.2 sürümünü
  indirip `.data/minecraft/` altına kuruyor:
  - sürüm dosyası, istemci ve sunucu jar'ı, bütün kütüphaneler, SHA1 ile
    doğrulanarak;
  - jar içindeki kaynaklar (`assets/jar`);
  - asset dizini ve nesneleri (sesler, diller; `--skip-objects` ile
    atlanır).

  Var olan ve hash'i tutan dosya yeniden indirilmiyor. İstemci jar'ının
  hash'i `tests/behavior/reference.json` ile tutmazsa duruyor.

  Seçenekler:
  - `--named`: nesnelere gezilebilir bir ağaç kuruyor (bağlantı
    oluşturulamazsa kopya).
  - `--decompile`: istemciyi ve JOML'u Vineflower ile
    `.data/minecraft/source` ve `joml-source` altına çıkarıyor. Mevcut
    kaynak klasörü silinip baştan üretiliyor.
- **`server`:** Test sunucusunu `.data/render/walk` altında kuruyor.
  - Minecraft EULA'sı yalnız `--accept-eula` verilince kabul ediliyor.
  - Araçların dayandığı ayarlar her seferinde yazılıyor, dosyanın geri
    kalanına dokunulmuyor: port 25600, çevrimdışı, barışçıl, tohum 776,
    görüş 12, RCON 25601 (parola `walk`).
  - `--background` sunucuyu ayrı süreçte başlatıyor, günlüğü `server.log`'a
    yazıyor ve oyun portu ile RCON açılana dek bekliyor.
  - `--stop` sunucuyu RCON ile durduruyor.
- **`play`:** `cargo build --release` yapıp (`--no-build` atlar) istemciyi
  `VOXEL_ASSETS` ayarlı başlatıyor. Seçenekler: `--user`, `--size`,
  `--fullscreen`, `--camera back|front`, `--profile`. Diğer `VOXEL_*`
  değişkenleri ortamdan olduğu gibi geçiyor.
- **`rcon`:** Komutları sırayla gönderip yanıtları yazıyor.

`--decompile` bu makinede çalıştırılmadı. Komut satırı referans kaynakları
üreten eski `decompile.sh` ile aynı.
