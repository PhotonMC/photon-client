# P01.c — Direction ve Axis enum adı Rust davranış kanıtı

Rust adapter `direction_enum_name` işlemi, ana Direction ve Axis enumlarının `name()`, `getName()`, `getSerializedName()` ve `toString()` sonuçlarını Java oracle ile karşılaştırır.

Java math capture kimliği `c26c352cd654343beebcfd73509e537efe7648d40c625692586886c7977757f2` olarak `.data/captures/behavior/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `c59b062efb49d8fcac9683b5d87bcc7929220c35f63dd99dcaf57c97ce937dc6` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Direction ve Axis enum ad/metin yüzlerini kapsar; alt enumların özel metinleri ayrı dilimdedir.
