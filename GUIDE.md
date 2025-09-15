新手指南：建立與調整 Profile
=============================

前置
- 安裝 Rust 並 `cargo run` 啟動服務
- 確認範例 JSON 已存在於 `huggingface/lerobot/calibration/...`

方式一：透過 Web UI
1) 開啟 `http://localhost:3000/`
2) 點選 robots 或 teleoperators 的某個 profile 進入檢視頁
3) 在文字框編輯 JSON（如調整某關節的 `range_min` 或 `range_max`）
4) 按下「儲存（PUT）」提交，成功後導回同頁
5) 若要刪除，按頁面下方「刪除」按鈕（留意不可復原）

方式二：透過 REST API
1) 列表：`GET /api/profiles?kind=robots`
2) 讀取：`GET /api/profiles/robots/my_awesome_follower_arm`
3) 局部更新（PATCH）：
   請求體（JSON）：
   {
     "shoulder_pan": { "range_max": 3300 }
   }
4) 全量更新（PUT）：將整份 profile JSON 作為請求體送出。
5) 新增：`POST /api/profiles/robots`，請求體：`{"name":"my_new_profile","profile":{...}}`
6) 刪除：`DELETE /api/profiles/robots/my_new_profile`

備註
- 寫入前會驗證：`range_min < range_max`、`id > 0` 等；錯誤將以 JSON 回應。
- 路徑可用 `CALIB_ROOT` 環境變數覆蓋預設校正根目錄。

