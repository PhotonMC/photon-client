# P16.o — Blok tonlaması — [client-parity] kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı iklim değerleri ve aynı renk haritası
pinned JAR'daki `ColorMapColorUtil.get`, `ARGB.multiply` ve renk sabitlerine
verilir, sonuçlar bit düzeyinde karşılaştırılır. Bir istisna aşağıda
işaretlenmiştir.

## Java kaynak incelemesi

Bir yüz renk taşımaz, **tonlama yuvası** taşır (`tintindex`). Yuvanın neye
çözüldüğü bloğun nerede olduğuna bağlıdır ve referansta iki ayrı yol vardır:

- `colorInWorld(state, level, pos)` — dünyanın içinde, çevredeki biyomlardan
  ortalanır.
- `color(state)` — dünyanın dışında (elde, envanterde, bizim sondamızda),
  bloğa göre **sabit** bir değerdir.

Sonda tek bir bloğu dünyasız çiziyor, yani doğru yol ikincisidir ve hiç biyom
istemez.

Yuva çözüldükten sonra `ARGB.multiply` ile dörtgenin rengine katlanır. Çarpma
kanal kanal ve **tamsayı bölmesiyle** yapılır; iki taraftan biri beyazsa
(`0xffffffff`) diğeri olduğu gibi döner. Bu kısayol süs değil: bölme bir adım
kaybettiği için beyazla çarpmak `0xfe`'yi `0xfd`'ye düşürürdü.

Biyom yolu için renk haritası araması da okundu, çünkü sabit değerlerin
çoğu o haritanın varsayılanıdır. `ColorMapColorUtil.get` üç şey yapar:
yağışı önce sıcaklıkla ölçekler (yani aynı yağış soğuk bir biyomda başka bir
satır okur), iki ekseni de ters çevirip **sıfıra doğru** kırpar, ve iki
indisi toplamak yerine **or**'lar. Sonuncusu önemli: sıfır–bir aralığının
dışındaki bir sıcaklık satırın sonuna taşmakla kalmaz, diğer eksenin
bitlerine sızar.

Sınır denetimi asimetriktir: üst sınır aşılırsa haritanın varsayılanı döner,
alt sınır aşılırsa **istisna fırlar**. Bu bir kusur gibi duruyor ama davranış
budur ve pinlenmiştir.

## Girdiler

`image/colour-map` on üç satır taşır: üç harita türünün varsayılanları, ova
iklimi, yağış ölçeklemesinin görüldüğü bir satır, bir adımın altında kalan bir
kesir, ve dört sınır durumu. Harita sentetik bir 256x256 görüntüdür — pinlenen
indisleme, oyunun kendi grafiği değil.

`image/argb-multiply` dokuz satırda çarpmayı sürüyor: iki tarafta da beyaz,
yarımlar, siyah, ve her kanalı bir olan bir çift (sonuç sıfır, çünkü
`1 * 1 / 255 = 0`).

`image/tint-constants` yedi adlandırılmış sabiti listeliyor.

## Sonuçlar

Java ve Rust her satırda aynı değeri verdi. Öne çıkanlar:

| Satır | Sonuç | Neden |
| --- | --- | --- |
| `grass/above-one` | `out_of_bounds` | Sıcaklık birin üstünde, indis negatif |
| `grass/below-zero` | `ffff00ff` | İndis haritanın sonunu aşıyor, varsayılan dönüyor |
| `grass/just-under-a-step` | `ff7f7f01` | Kesir kırpılıyor, `grass/default` ile aynı |
| `one-each` | `00000000` | Kanal başına `1 * 1 / 255 = 0` |
| `max` | `fdfdfdfd` | `254 * 254 / 255 = 253`, beyaz kısayolu girmiyor |

Koşu: `python3 tools/behavior/run.py capture --family image` ardından
`compare --family image --adapter target/debug/examples/behavior_adapter`
→ `parity_passed`.

## Blok → tonlama tablosu

Bu kayıt yazıldığında tablo pinlenmemişti; kayıt defterini sormak denenmiş ve
oracle sürecini düşürmüştü. Teşhis yanlıştı ve
[P16.s](block-registry-conformance.md) ile düzeltildi: tablo artık referansın
kendi kayıt defterinden okunuyor ve her koşuda karşılaştırılıyor. Daha önce
tahmin yürütmemek için dışarıda bırakılan yapışık sap blokları da içeride.

## Çizime bağlanması

`Vertex` artık bir tonlama taşıyor ve parça gölgelendirici texel'i onunla
çarpıyor, referansın `block.fsh`'inin `texture(Sampler0, uv) * vertexColor`
yaptığı gibi. Yuvası olmayan bir yüz beyaz alıyor, yani çarpma onu değiştirmiyor.

## Açık kapsam

Biyom ortalamalı yol (`colorInWorld`) yok; bir dünya ve biyom verisi istiyor.
`BiomeColors.getAverageFoliageColor` çevredeki bloklarda dolaşıp ortalama
alıyor ve o yarıçap ayrıca pinlenmeli.

Su, kızıl taş teli, şeker kamışı ve sap blokları için `color(state)` sabit
değil, bloğun durumuna bakıyor; bunlar da açık.
