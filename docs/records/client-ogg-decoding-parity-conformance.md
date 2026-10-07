# T62 — Ogg/Vorbis çözümü — [client-parity]

Bu kayıt oyunun `.ogg` ses nesnelerinin 16 bit işaretli örneklere çözümünü kapsar. Kanıt sınıfı `client-parity`'dir **bir en küçük anlamlı bit toleransıyla** (aşağıda): biçim ve örnek sayısı gerçek `JOrbisAudioStream` ve `ChunkedSampleByteBuf` ile aynıdır, her örnek referansın örneğinden en çok 1 farklıdır. Çalma ve duyulan eşitlik bu kaydın dışındadır ([P20](../roadmap.md)).

## Java kaynak incelemesi

- `JOrbisAudioStream`: Ogg sayfa/paket okur, `DspState`/`Block` ile sentezler; biçim `(hız, 16, kanal, işaretli, küçük-sonlu)`. Akışın örnek sayısı, **akışı bitiren son sayfanın** granül konumudur (`totalSamplesInStream`); bundan fazla örnek yazılmaz (kodlayıcının son paketteki dolgusu atılır). Kanal sayısı 1, 2 ya da başka olabilir (örnekler kanal kanal dizilir).
- `ChunkedSampleByteBuf.accept`: `clamp((int)(örnek · 32767,5 − 0,5), −32768, 32767)`. `FloatSampleSource.read(n)`: paketler, örnek baytları `n`'e varana dek okunur; boş sonuç akış sonudur. `LoopingAudioStream`: boş okuma görünce akışı baştan açıp yeniden okur.

## Kapsam

- `sound/decode` 9 satır: gerçek dosyalar (tık, adım, domuz, mağara ambiyansı, köylü, çok kısa bebek tavuk adımı, 6 MB'lık müzik): kanal ve hız, örnek sayısı ve **her örnek, en çok bir bit farkla** (oracle örnekleri `.data/minecraft/reference-pcm/` altına yazar, izlenmez; adaptör aynı dosyalardan karşılaştırır).
- Rust: `voxel-audio::decode` (`OggStream`, `LoopingOggStream`, `decode_ogg`); çözücü `lewton` 0.10.2 (MIT/Apache-2.0), 16 bit dönüşümü ve son sayfa kırpması referansınındır.

## Neden toleranslı

`lewton`, JOrbis'in portu değil ayrı bir çözücüdür; kayan nokta işlemleri başka sırada yapıldığından bir örnek bazen bir bit farklı yuvarlanır. Ölçüm (üç dosya): 15 032 örnekten 1'i, 672 000'den 98'i, 43 835 400'den 19 773'ü (%0,045) farklıdır, en büyük fark 1. Bu −90 dB düzeyindedir. Bit eşitliği için JOrbis'in Rust'a çevirisi gerekir (karar kaydı: [audio-backend-decision.md](../audio-backend-decision.md)); gerekçe doğarsa ayrı görev olur.

## Sapmalar ve açık kalanlar

- Bit eşitliği iddia edilmez (yukarıda). Akış parçalama ve döngü mantığı birim testli değil, yalnız `read_all` oracle'dadır (parçalar paket sınırında kesilir; döngü başa dönüşü basittir).
- `.ogg` olmayan ses nesneleri (yok) ve bozuk dosya hataları (`IOException` metinleri) karşılaştırılmaz.
