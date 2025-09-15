lerobot-servo-adjust
====================

使用 Rust 的 Axum 建立一個可對 LeRobot 校正檔（JSON）進行「伺服馬達參數微調」的服務／工具，支援 follower 與 leader 兩類型配置。並透過 Askama 模板提供簡單的網頁 UI。

功能概述
- 讀取與驗證校正 JSON（如 `homing_offset`, `range_min`, `range_max`）
- 以 REST API 調整關節參數
- 以 Askama（伺服端渲染）提供簡易操作頁面

API 端點（節選）
- `GET /api/profiles?kind=robots|teleoperators` 列出 profiles
- `GET /api/profiles/{kind}/{profile}` 讀取單一 profile
- `PATCH /api/profiles/{kind}/{profile}` 局部更新
- `PUT /api/profiles/{kind}/{profile}` 全量更新
- `POST /api/profiles/{kind}` 建立 profile
- `DELETE /api/profiles/{kind}/{profile}` 刪除 profile

UI 使用
- 首頁 `/`：瀏覽 robots 與 teleoperators 之 profiles 清單
- 點入 `/profiles/{kind}/{profile}`：查看並編輯 JSON，提交後寫回
- 若以表單提交，後端會透過 API（PUT/DELETE）更新資料並導回

資料路徑
- 既有範例檔：
  - `huggingface/lerobot/calibration/robots/so101_follower/my_awesome_follower_arm.json`
  - `huggingface/lerobot/calibration/teleoperators/so101_leader/my_awesome_leader_arm.json`

快速開始
1) 安裝 Rust（`rustup`）
2) 相依套件已於 `Cargo.toml` 設定（Axum、Tokio、Serde、Askama 等）
3) 執行：`cargo run`

更多資訊
- 開發說明：見 `DEVELOP.md`
- 開發代辦：見 `TODO.md`
