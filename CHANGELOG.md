# 変更履歴

## 0.7.0 - 2025-03-27

### 追加

- `new_transcription`メソッドをPathSetsに追加

### 変更

- `PathSets::new`からtranscription機能を`new_transcription()`に分離
  - `new_transcription()`は`experimental` featureのため通常使用不可
- `transcription::transcription()`の戻り値を`Vec<Stirng>`から`String`に変更
