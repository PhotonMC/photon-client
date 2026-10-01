# T02 — `cases.py` GUI üreticilerini ayır

**Amaç:** `tools/behavior/cases.py` 14,7k satır. `gui_*_cases` üreticilerini `tools/behavior/cases_gui.py`'ye taşı; GUI görevleri yalnız küçük dosyayı açsın.
**Bağımlılık:** T01 (sıra önerisi, teknik bağımlılık yok). **Boy:** S.

## Oku
- `tools/behavior/cases.py`: `grep -n "^def gui_\|generators\[\"gui\"\]\|^import\|^from" tools/behavior/cases.py`. GUI üreticileri ~14005–14660 aralığında (`gui_cases` … `gui_tab_list_cases` ve sonrası); yardımcılar (`command_tree_hex` vb.) GUI'ye özgü mü ortak mı kontrol et.
- `tools/behavior/run.py`: `cases` modülünü nasıl içe aktardığı (`grep -n "cases" tools/behavior/run.py`), capture kimliğinin neyi hash'lediği.

## Yapılacaklar
1. Yeni `tools/behavior/cases_gui.py`: taşınan üreticiler + yalnız onların kullandığı yardımcılar. Ortak yardımcı `cases.py`'de kalır ve `from cases import ...` ile alınır (döngüsel içe aktarma olmasın: `cases.py` GUI'yi `generators["gui"]` satırında lazily içe aktarsın).
2. `generators["gui"]` satırı `cases_gui.gui_generators()` gibi tek giriş noktasına bağlansın.
3. Üretilen case JSON'u **bayt bayt aynı** olmalı: taşımadan önce `python3 -c` ile `list(generators["gui"]())`'in `json.dumps(sort_keys=True)` özetini kaydet, sonra aynı özetle karşılaştır.
4. Capture kimliği kaynak dosya hash'ini içeriyorsa (`run.py`) yeni capture alınır; Java çıktısı aynı olmalı (`run.py` capture "Existing capture differs" derse dur, nedenini bul).

## Kabul
`python3 tools/voxel.py check --capture` → `CHECK OK` (928).

## Kapsam dışı
Başka ailelerin üreticileri; case içeriğini değiştirmek.

## Belgeler
`docs/gui-oracle.md` "Yeni bir ekran için" cümlesi ve `AGENTS.md` token bölümü listesi; `docs/tasks/README.md` G tarifinin 2. adımındaki "T02'den sonra" notu zaten var.

## Devir notu
- GUI üreticileri ve `command_tree_hex` (yalnız GUI kullanıyordu) `tools/behavior/cases_gui.py`'ye taşındı; giriş noktası `gui_generators()`, `build_cases` onu tembel içe aktarır.
- Taşımadan önce/sonra `build_cases(['gui'])` JSON özeti aynı (sha256 35c46bf1…); capture kimliği girdi baytlarına bağlı olduğundan yeni capture gerekmedi.
- Yeni GUI üreticisi artık `cases_gui.py`'de, `gui_generators()` zincirine eklenir.
- `check --capture`: CHECK OK, parity 928. Açık kalan yok.
