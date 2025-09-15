## 開發代辦清單（lerobot-servo-adjust）

基準原則：每一項儘量小而可驗證，能在 30–90 分鐘內完成並提交。

### 專案初始化與文件
- [x] 統一檔案編碼為 UTF-8（包含 Windows 開發環境說明）
- [x] 重新整理 `README.md`（專案目標、快速開始、連結）
- [x] 補強 `DEVELOP.md`（執行流程、資料路徑、驗證規則）
- [x] 維護 `TODO.md`（持續拆分/重整）

### 相依與工具鏈
- [x] 審視 `Cargo.toml` 相依版本範圍與最小可行組合
- [x] 新增 `rust-toolchain.toml` 鎖定 Rust 版本與組件
- [x] 文件化 `clippy` 與 `rustfmt` 使用方式
- [x] 文件化 `cargo-watch` 使用方式

### 專案骨架
- [x] 建立模組目錄：`src/api`, `src/model`, `src/store`, `src/config`, `src/web`
- [x] `main.rs` 建立 Axum 最小伺服器與路由（含 `GET /healthz`）
- [x] 導入日誌：`tracing` + `tracing-subscriber`（支援 `RUST_LOG`）

### 設定管理（config）
- [x] 定義 `Config` 結構：`calib_root: PathBuf`
- [x] `Config::from_env()` 解析 `CALIB_ROOT`，預設 `huggingface/lerobot/calibration`
- [x] 單元測試：預設值、自訂環境變數兩種情境

### 資料模型（model）
- [x] 定義 `Joint`：`id`, `drive_mode`, `homing_offset`, `range_min`, `range_max`
- [x] 定義 `Profile`：`HashMap<String, Joint>` 或結構化型別
- [x] 實作驗證：`validate_joint()` 與 `validate_profile()`
- [x] 單元測試：序列化/反序列化、驗證（越界/鍵名不合法/壞檔）

### 檔案存取層（store）
- [x] 定義 `Kind` 列舉：`Robots` | `Teleoperators`
- [x] 列舉 profiles：`list_profiles(kind) -> Vec<ProfileMeta>`
- [x] 讀取 profile：`read_profile(kind, name) -> Profile`
- [x] 寫入 profile：`write_profile(kind, name, Profile)`（臨時檔 + 原子 `rename`）
- [x] 可選備份：寫入前產生 `*.bak`
- [x] 自訂錯誤型別：I/O、JSON、驗證錯誤（`thiserror`）
- [x] 單元測試：以臨時目錄模擬 I/O，覆蓋成功與失敗路徑

### REST API（api）
- [x] 路由前綴 `/api` 與中介層（請求/回應日誌）
- [x] 錯誤回應格式：`code`, `message`, `details`
- [x] `GET /api/profiles?kind=robots|teleoperators`（列表）
- [x] `GET /api/profiles/{kind}/{profile}`（讀取）
- [x] `PATCH /api/profiles/{kind}/{profile}`（局部更新：單/多關節）
- [x] `PUT /api/profiles/{kind}/{profile}`（全量覆寫）
- [x] `POST /api/profiles/{kind}`（新增：範本或空白）
- [x] `DELETE /api/profiles/{kind}/{profile}`（刪除）
- [x] 整合測試：Axum 測試客戶端 + 臨時目錄 store

### Web UI（Askama + axum）
- [x] 建立 `templates/` 與 `base.html`（UTF-8、共用區塊）
- [x] 首頁：列出 profiles、切換 robots/teleoperators
- [x] 檢視頁：顯示單一 profile 的關節與數值（JSON）
- [x] 編輯頁（表單）：可編修 JSON 並提交（PUT）
- [x] 後端處理表單：呼叫 API `PUT/DELETE`，完成後導回檢視頁
- [x] 錯誤顯示：驗證失敗/I-O 錯誤提示

### 日誌與觀測
- [x] 進出站請求日誌（method, path, status, latency）
- [x] store 寫入失敗與驗證錯誤詳情（tracing 訊息）
- [x] 設定 `EnvFilter` 範例與文件

### 品質與驗證
- [ ] `clippy` 淨空與 `rustfmt` 格式（執行與修正）
- [x] 單元測試覆蓋 model/store
- [x] 整合測試覆蓋常見 API 路徑（讀、寫、驗證失敗）

### 文件與示例
- [x] `README.md`：加入 API/畫面流程說明
- [x] `DEVELOP.md`：補 Lint/Watch 與 EnvFilter
- [x] 新手指南：如何新增/複製 profile 並調整參數（`GUIDE.md`）

### 後續（可選）
- [x] 發佈設定（binary 名稱、簡易打包）
- [ ] 版本化與回滾（保留歷史）
- [ ] 權限控管（唯讀/可寫、簡易驗證）
