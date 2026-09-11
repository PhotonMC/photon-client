# P08.b/c — Vanilla 26.2 chunk paketinin yapısal doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu geçici `/tmp/voxel-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative ve peaceful ayarlarıyla başlatıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example connection_probe -- 127.0.0.1:25566 RustPalette1
cargo run --offline --example connection_probe -- 127.0.0.1:25566 RustPalette2
```

İki bağımsız oturumda parser bütün alınan chunk payload'larını hatasız tüketti:

| Gözlem | RustChunk1 | RustChunk2 |
| --- | ---: | ---: |
| chunk packet | 27 | 24 |
| decoded chunk | 27 | 24 |
| chunk batch finished | 7 | 6 |
| toplam alınan packet | 485 | 496 |
| ilk chunk section sayısı | 24 | 24 |

`decoded chunk` sayısının `chunk packet` sayısına eşit olması, canlı koşuda her chunk gövdesinin parser'dan geçtiğini gösterir. Her iki koşuda da ilk chunk'ın section buffer'ı 24 section olarak ayrıştırıldı. Toplam packet ve chunk sayısı vanilla sunucunun geçici gönderim zamanlamasına bağlıdır.

İlk chunk gözlemleri ve section başlıkları aşağıdaki gibidir:

```text
RustPalette1:
first_chunk=x:1,z:-2,heightmaps:3,section_bytes:19524,block_entities:0,sky_mask_words:1,block_mask_words:1,empty_sky_mask_words:1,empty_block_mask_words:1,sky_updates:4,block_updates:5
first_section=non_empty:4096,fluid:264,block_bits:4,block_palette:10,block_palette_ids:[85, 30417, 564, 102, 124, 23452, 6884, 130, 5308, 132],block_words:256,biome_bits:0,biome_palette:1,biome_palette_ids:[4],biome_words:0
first_chunk_section_count=24

RustPalette2:
first_chunk=x:1,z:-3,heightmaps:3,section_bytes:22265,block_entities:0,sky_mask_words:1,block_mask_words:1,empty_sky_mask_words:1,empty_block_mask_words:1,sky_updates:4,block_updates:2
first_section=non_empty:4096,fluid:0,block_bits:4,block_palette:6,block_palette_ids:[85, 23452, 30417, 6884, 130, 132],block_words:256,biome_bits:0,biome_palette:1,biome_palette_ids:[4],biome_words:0
first_chunk_section_count=24
```

Bu sonuç x/z signed integer alanlarını, üç heightmap kaydını, section buffer sınırını, boş block entity listesini, dört ışık maskesini ve sky/block update listelerini vanilla wire sırasıyla ayırdığımızı gösterir. İki koşudaki 24 section'ın section sayaçları, palette bits değerleri, palette global ID listeleri ve storage word uzunlukları Java layout'una göre tüketildi; parser payload sonunda trailing byte bırakmadı. İlk chunk'ın konumu ve palette giriş sayısı, sunucunun packet gönderim sırasına göre değişebildiği için iki koşu ayrı kaydedildi.

## Açık kapsam

Bu kayıt packet gövdesinin ve section palette storage düzeninin yapısal tüketimidir. Palette ID'lerinin block state/biome kayıtlarına çözümü, ışık nibble değerleri, heightmap semantiği, registry ID çözümü, block entity world konumu, client chunk cache'i, load/update/unload sırası, dimension/respawn ve render açık kalır. Başarılı canlı parser, oynanabilir istemci veya Java ile tam dünya eşitliği anlamına gelmez.

Sözleşme ayrıntıları [chunk packet contract belgesindedir](chunk-packet-contract.md). Önceki login/configuration/play bağlantı kanıtı [vanilla oyun oturumu kaydında](vanilla-connection-parity.md), Pumpkin koşusu ise [ayrı kayıtta](game-connection-parity.md) tutulur. Bu test kullanıcının istediği üzere resmi vanilla sunucusuyla yapıldı; Pumpkin eksikleri bu baseline'a karıştırılmadı.
