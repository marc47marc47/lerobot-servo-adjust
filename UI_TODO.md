UI 代辦清單（Arm 影像操作）
==========================

目標：依照 `lerobot-arm.jpg` 圖示的節點位置，點選 L1–L6（Leader＝teleoperators）、F1–F6（Follower＝robots）以編輯對應 servo motor（id 代號）的參數。

已完成（初版）
- 在 `/arm/{kind}/{profile}` 顯示底圖與 6 個熱點（示意座標）。
- 點選熱點後，讀取 profile 以 id 對應到實際關節名稱並顯示表單。
- 表單提交：透過 API `PATCH /api/profiles/{kind}/{profile}` 更新對應關節的 id/drive_mode/homing_offset/range_min/range_max。
- 支援 Leader/Follower 的標籤（L/F）與列表導覽。
- 提供 `/assets/lerobot-arm.jpg` 供模板載入圖片。

待辦與進度
- [ ] 精準熱點座標：依 `lerobot-arm.jpg` 實際標示微調每一點 `top/left %`（必要時區分 L/F）。
- [x] 欄位驗證與限制：前端檢查 `range_min < range_max`、`id > 0`（仍保留後端驗證）。
- [x] 視覺與互動：
  - [x] 熱點 hover/active 與 label 浮出（顯示對應關節名稱）。
  - [x] 響應式樣式，窄螢幕自動改為單欄。
  - [x] 圖片與表單左右佈局。
- [ ] 多關節批次編輯：一次變更多個 joint 後送出 `PATCH`。
- [ ] 失敗回饋：介面顯示 API `details` 訊息並引導修正。
- [x] 切換 profile 與 kind 的控制列（下拉選單導覽）。
- [x] 權限/唯讀模式：以環境變數 `READ_ONLY` 控制，唯讀時停用提交按鈕。

命名與對應
- Leader => kind=`teleoperators`，熱點標籤 L1..L6；Follower => kind=`robots`，熱點標籤 F1..F6。
- 以 `id` 對應 JSON 中的實際關節鍵名（例如 `shoulder_pan` 等），由後端讀取 profile 決定。
