# P01.d — VoxelShape getFaceShape sözleşmesi

Bu dilim Java 26.2 `VoxelShape.getFaceShape(Direction)` davranışını kapsar. Empty shape, tam blok, seçilen eksende tam 0/1 aralık, batı ve doğu yüz dilimleri, sınır voxel'i bulunmayan shape, üst yüz ve OR birleşimi için sekiz özgün senaryo hazırlanmıştır. Rust sonucu [parite kanıtında](shape-face-parity.md) Java çıktısıyla karşılaştırılmıştır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:202-235`, `SliceShape.java:6-33`, `SubShape.java:6-47`, `CubeVoxelShape.java:7-17` ve `Shapes.java:44-172` incelendi. `MathOracle` doğrudan `getFaceShape` çağrısını ve dönen shape'in box snapshot'ını gözler; beklenen yüzü Rust sonucundan üretmez.

Bu dilim block-local koordinatlarla üretilen box ve OR shape'lerinin yüz sonucunu kapsar. Rotation [ayrı sözleşmede](shape-rotate-contract.md), dünyadaki collider toplama sonraki dilimlerde ele alınır.

## Davranış sözleşmesi

- Empty shape için sonuç empty shape olur. Tam blok için ve seçilen eksen zaten 0 ile 1 arasında cube-like ise mevcut shape sonucu korunur.
- Diğer shape'lerde pozitif yön için `0.9999999`, negatif yön için `1e-7` örnek koordinatıyla seçilen eksenin sınır voxel'i bulunur.
- Sınır voxel'i boşsa sonuç empty shape olur. Doluysa seçilen eksen `[0, 1]` koordinatlarına normalize edilir; diğer eksenlerin koordinat düzlemleri ve dolu hücreleri korunur.
- Yüz slice'ı bütün eksenlerde cube-like ise `Shapes.block` snapshot'ı üretilir; kısmi yüzlerde slice'ın box sınırları kullanılır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| empty/block | 2 | Sliver empty sonucu ve birim blok |
| tam eksen | 1 | X eksenini 0/1 kapsayan, Y'de kısmi box |
| batı/ doğu slice | 2 | İlgili blok sınırına ulaşan kısmi box'lar |
| eksik sınır | 1 | Seçilen yüzde dolu voxel bulunmayan box |
| üst slice | 1 | Y ekseninde pozitif sınır ve X/Z'de kısmi yüz |
| OR birleşimi | 1 | İki yarım box'ın birleşik tam yüzü |

Girdiler `family: "math"`, `operation: "shape"` ve `kind: "face"` veya `kind: "face_join"` taşır. `direction` altı Java yön adından biridir. Shape sonucu `empty`, `boxes` ve non-empty durumda `bounds` alanlarıyla snapshot'lanır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Bu dilimin sekiz senaryosunu içeren math referans kimliği `22be0a4cb9fc67f18a020a971ff97c40b8df3f00566b14266f8ed2f447053381`'dir. Birleşik altı aile referansı `17d40db8607a5df22fc08a42ccd0065b6ffe818cef98507474d8663297c0ac04` ile doğrulanmıştır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
