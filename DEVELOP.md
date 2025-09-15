# 開發說明（lerobot-servo-adjust）

本專案目標：以 Rust（Axum Web 框架）開發一個可對 LeRobot 相關校正檔（JSON）進行「伺服馬達參數微調」的服務／工具，支援 follower 與 leader 兩類型配置，直接讀寫專案內的校正 JSON 檔。

提示：目前 `README.md` 內容疑似因編碼而出現亂碼，但可辨識出「使用 axum 開發、微調 servo motor、調整 follower/leader、以 JSON 檔保存」等要點。以下開發文檔據此與現有檔案結構整理。

## 環境需求
- Rust 工具鏈：`rustup` + `cargo`（建議 stable 最新版）
- 平台：Windows / macOS / Linux 皆可
- 文字編碼：請以 UTF-8 儲存檔案（避免中文亂碼）

### Windows 編碼小提示
- PowerShell 請使用 UTF-8 輸出：`$PSStyle.OutputRendering = "Host"; [Console]::OutputEncoding = [Text.UTF8Encoding]::UTF8`
- 或改用 `chcp 65001` 切換至 UTF-8 代碼頁（可能需要重啟視窗）

## 專案結構
- `Cargo.toml`：Rust 專案設定（目前無依賴）
- `src/main.rs`：進入點（目前為 Hello, world!）
- `huggingface/lerobot/calibration/robots/so101_follower/my_awesome_follower_arm.json`：範例「跟隨者」機械臂校正檔
- `huggingface/lerobot/calibration/teleoperators/so101_leader/my_awesome_leader_arm.json`：範例「操作者」機械臂校正檔

兩份 JSON 結構相似，每個關節（如 `shoulder_pan`, `elbow_flex`, `wrist_roll`, `gripper` 等）包含：
- `id`：數值 ID
- `drive_mode`：驅動模式
- `homing_offset`：零點偏移
- `range_min` / `range_max`：可動範圍（應滿足 `range_min < range_max`）

## 技術選型（規劃）
- Web：`axum`
- 模板／伺服端渲染：`askama` + `askama_axum`（提供簡易 Web UI）
- 非同步執行：`tokio`
- 序列化：`serde`, `serde_json`
- 錯誤處理：`anyhow` 或 `thiserror`
- 日誌：`tracing`, `tracing-subscriber`
- 檔案與路徑：`walkdir`（列舉檔案）、`fs` 標準庫（原子寫入）

以上套件尚未加入 `Cargo.toml`，請依 TODO 計畫逐步引入。

## 開發與執行
- 編譯執行：`cargo run`
- 建議加入 `cargo-watch` 以便開發（選配）：`cargo watch -x run`
- 測試：`cargo test`
- 模板位置：`templates/`（Askama 預設），使用 `*.html` 以 UTF-8 儲存

### Lint 與格式化
- 格式：`cargo fmt --all`
- Lint：`cargo clippy --all-targets -- -D warnings`

### 觀測與日誌（tracing）
- 以 `TraceLayer` 記錄 HTTP 請求
- `store` 層對 I/O、JSON、驗證錯誤與成功寫入有 `tracing` 訊息
- 範例環境變數：
  - `RUST_LOG=info,axum=info,tower_http=info,lerobot_servo_adjust=debug`
  - Windows PowerShell：`$env:RUST_LOG='info,axum=info,tower_http=info,lerobot_servo_adjust=debug'`

### 開發效率
- 安裝 cargo-watch：`cargo install cargo-watch`
- 常用指令：`cargo watch -x 'check' -x 'test' -x 'run'`

## 發佈與打包
- 釘住版本：`rust-toolchain.toml` 已設定 stable + clippy/rustfmt
- 釋出建置：`cargo build --release`
- 簡易打包：
  - Windows（PowerShell）：`./scripts/pack.ps1` 或 `./scripts/pack.ps1 -Version 0.1.0`
  - Linux/macOS（bash）：`bash scripts/pack.sh` 或 `bash scripts/pack.sh 0.1.0`
- 產出位置：`dist/lerobot-servo-adjust-<version>-<platform>-<arch>.zip`
- 內容：
  - `bin/lerobot-servo-adjust[.exe]`
  - `templates/`（Askama 模板）
  - `huggingface/`（範例資料）
  - `README.md`, `DEVELOP.md`, `GUIDE.md`

## 設定與檔案路徑
- 預設校正檔根目錄：`huggingface/lerobot/calibration`
  - `robots/...`：follower（被動端/實際機器）
  - `teleoperators/...`：leader（主動端/操作者）
- 可透過環境變數覆蓋：`CALIB_ROOT` 指向自訂根目錄（規劃中）

## API 與 UI 草案
Base path：`/api`

- `GET /api/profiles?kind=robots|teleoperators`：列出指定類型下的所有 profile 路徑/名稱
- `GET /api/profiles/{kind}/{profile}`：讀取單一校正 JSON（回傳整份）
- `PATCH /api/profiles/{kind}/{profile}`：更新部分欄位（例如某關節的 `range_min`）
- `PUT /api/profiles/{kind}/{profile}`：整份覆寫（需完整驗證）
- `POST /api/profiles/{kind}`：建立新 profile（可由範本複製或空白骨架）
- `DELETE /api/profiles/{kind}/{profile}`：刪除 profile（可選）
- `GET /api/validate`（可選）：提供外部 JSON 的預檢與錯誤清單

回應統一採用 JSON，錯誤回應包含 `code`、`message`、`details`。

UI（Askama 模板示例）：
- `GET /`：首頁，列出 profiles 與類型切換
- `GET /profiles/{kind}/{profile}`：顯示單一 profile，可在表單中調整關節參數
- `POST /profiles/{kind}/{profile}`：從表單提交更新（內部呼叫 API `PATCH/PUT` 後導回）

## JSON 結構與驗證
- Key 為關節名稱；Value 為物件：
  - `id`: 整數，> 0
  - `drive_mode`: 整數，允許值依硬體定義（暫以非負整數）
  - `homing_offset`: 整數，可正負
  - `range_min`, `range_max`: 整數，必須 `range_min < range_max`
- 允許的關節名稱集合由現有檔案自動推導，或在程式訂定白名單。
- 寫回檔案前進行完整驗證；不合法時拒絕並回傳錯誤。

## 檔案寫入策略
- 先寫入臨時檔（同資料夾，例如 `*.json.tmp`），完成後以原子 `rename` 取代原檔
- 寫入前建立備份（例如 `*.bak`，可選）
- 跨平台路徑與鎖定：優先使用標準庫，必要時加檔案鎖避免併發寫入

## 錯誤處理與日誌
- 使用 `thiserror/anyhow` 建立錯誤型別與脈絡
- 以 `tracing` 紀錄請求、I/O、驗證錯誤與統計

## 測試策略
- 單元測試：JSON 解析/序列化、驗證邏輯
- 整合測試：以 axum 測試客戶端呼叫 API、檔案 I/O 以暫存目錄隔離
- 邊界測試：非預期關節名稱、越界 range、空檔案、毀損 JSON

## 程式風格
- Rust 2024 edition，啟用 `rustfmt` 預設規則
- 模組劃分：
  - `api/` 路由與 handler
  - `model/` JSON 結構與驗證
  - `store/` 檔案列舉、讀寫、原子替換
  - `config/` 路徑與環境變數

## Git 與 Commit 建議
- Commit 小步前進，訊息使用動詞祈使句：如「Add Axum server skeleton」
- 與 Issue/TODO 對應，PR 說明清楚測試重點

## 未來擴充
- 角色與權限（限制誰可寫入）
- 版本化（保留歷史與回滾）
- 更友善的 Web UI（Slider/表單）或 CLI 互動模式
