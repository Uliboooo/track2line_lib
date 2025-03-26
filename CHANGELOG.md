# 変更履歴

## 0.9.1 - 2025-03-27

### 修正

- `Config`のフィールドにアクセスできない問題を修正

## 0.9.0 - 2025-03-27

### 追加

- `Config`に`set_both()`メソッドを追加
  - 音声とテキストの拡張子を同時に設定可能な関数

## 0.8.0 - 2025-03-27

### 追加

- `Config`featureを追加
- 👆に伴い`file_ctrl.rs`を追加

### 変更

0.7.0より一応掲載(内容は0.7.0)

- `PathSets::new`からtranscription機能を`new_transcription()`に分離
  - `new_transcription()`は`experimental` featureのため通常使用不可
- `transcription::transcription()`の戻り値を`Vec<Stirng>`から`String`に変更

## 0.7.0 - 2025-03-27

### 追加

- `new_transcription`メソッドをPathSetsに追加

### 変更

- `PathSets::new`からtranscription機能を`new_transcription()`に分離
  - `new_transcription()`は`experimental` featureのため通常使用不可
- `transcription::transcription()`の戻り値を`Vec<Stirng>`から`String`に変更
