use crate::AppState;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Datelike, Days, Local, NaiveDate};
use regex::Regex;
use ring::signature::{UnparsedPublicKey, ED25519};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tauri::State;

// 授权验签公钥（厂商私钥签发，客户端只内置公钥）
const PUBLIC_KEY: [u8; 32] = [
    0xad, 0xaa, 0x84, 0x83, 0xaf, 0x46, 0x2e, 0x83, 0x80, 0xba, 0x88, 0x1d, 0x3a, 0x81, 0xc2, 0x00,
    0x94, 0xd7, 0xce, 0x0a, 0xf1, 0x90, 0x96, 0x7a, 0x72, 0x2c, 0xec, 0xdb, 0x3e, 0x6b, 0x33, 0x53,
];

// ==================== 授权 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicensePayload {
    product: String,
    #[serde(default)]
    license_id: String,
    #[serde(default)]
    customer: String,
    #[serde(default)]
    edition: String,
    #[serde(default)]
    perpetual: bool,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    updates_until: String,
    #[serde(default)]
    issued_at: String,
}

#[derive(Debug, Serialize)]
pub struct SmartModuleStatus {
    pub entitled: bool,
    pub customer: String,
    pub edition: String,
    pub updates_until: String,
    pub licensed_at: String,
}

/// 校验签名并返回 payload（License 与规则包通用）
fn verify_signed(token: &str) -> Result<serde_json::Value, String> {
    let parts: Vec<&str> = token.trim().split('.').collect();
    if parts.len() != 3 || parts[0] != "SV1" {
        return Err("授权码格式错误".to_string());
    }
    let payload = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| "授权码内容解析失败".to_string())?;
    let sig = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| "授权签名解析失败".to_string())?;
    let pk = UnparsedPublicKey::new(&ED25519, &PUBLIC_KEY);
    pk.verify(&payload, &sig)
        .map_err(|_| "授权码签名无效".to_string())?;
    serde_json::from_slice(&payload).map_err(|_| "授权信息解析失败".to_string())
}

fn verify_token(token: &str) -> Result<LicensePayload, String> {
    let v = verify_signed(token)?;
    let p: LicensePayload =
        serde_json::from_value(v).map_err(|_| "授权信息解析失败".to_string())?;
    if p.product != "smart_voucher" {
        return Err("授权产品不匹配".to_string());
    }
    if !p.features.iter().any(|f| f == "smart_voucher") {
        return Err("授权不包含智能记账模块".to_string());
    }
    Ok(p)
}

fn stored_token(conn: &Connection) -> Option<String> {
    let row: Option<Option<String>> = conn
        .query_row("SELECT license_token FROM smart_entitlement WHERE id=1", [], |r| {
            r.get(0)
        })
        .optional()
        .ok()?;
    row.flatten()
}

fn ensure_entitled(conn: &Connection) -> Result<(), String> {
    let token = stored_token(conn).ok_or("智能记账模块未授权，请在模块中心激活")?;
    verify_token(&token)?;
    Ok(())
}

fn status_from(conn: &Connection) -> SmartModuleStatus {
    let licensed_at: String = conn
        .query_row("SELECT COALESCE(activated_at,'') FROM smart_entitlement WHERE id=1", [], |r| {
            r.get(0)
        })
        .optional()
        .ok()
        .flatten()
        .unwrap_or_default();
    match stored_token(conn).and_then(|t| verify_token(&t).ok()) {
        Some(p) => SmartModuleStatus {
            entitled: true,
            customer: p.customer,
            edition: p.edition,
            updates_until: p.updates_until,
            licensed_at,
        },
        None => SmartModuleStatus {
            entitled: false,
            customer: String::new(),
            edition: String::new(),
            updates_until: String::new(),
            licensed_at,
        },
    }
}

#[tauri::command]
pub fn smart_module_status(state: State<AppState>) -> Result<SmartModuleStatus, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(status_from(&db))
}

/// 联网兑换授权码（换取签名 Token）；返回 Token 文本
fn describe_reqwest_error(e: &reqwest::Error) -> String {
    let mut msg = e.to_string();
    let mut src = std::error::Error::source(e);
    while let Some(s) = src {
        msg.push_str(" -> ");
        msg.push_str(&s.to_string());
        src = std::error::Error::source(s);
    }
    msg
}

fn redeem_online(db: &Connection, code: &str) -> Result<String, String> {
    let url = get_setting(db, "smart_activation_url");
    if url.trim().is_empty() {
        return Err("未配置授权兑换地址（settings.smart_activation_url）".to_string());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(url.trim())
        .json(&json!({ "code": code }))
        .send()
        .map_err(|e| format!("联网兑换失败: {}", describe_reqwest_error(&e)))?;
    let status = resp.status();
    let body = resp.text().map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("兑换失败：服务器返回 {} {}", status, body.trim()));
    }
    let body = body.trim();
    if body.starts_with("SV1.") {
        return Ok(body.to_string());
    }
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|_| "兑换响应格式错误".to_string())?;
    v.get("license")
        .or_else(|| v.get("token"))
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "兑换响应缺少 license 字段".to_string())
}

/// 激活：授权码可直接是签名 Token，也可为卡密（联网兑换一次后得到 Token）
#[tauri::command]
pub fn smart_activate_code(state: State<AppState>, code: String) -> Result<SmartModuleStatus, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let token = if code.trim().starts_with("SV1.") {
        code.trim().to_string()
    } else {
        redeem_online(&db, code.trim())?
    };
    let p = verify_token(&token)?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let payload_json = serde_json::to_string(&p).map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO smart_entitlement(id, license_token, payload_json, activated_at, updates_until)
         VALUES(1, ?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET license_token=excluded.license_token,
           payload_json=excluded.payload_json, activated_at=excluded.activated_at,
           updates_until=excluded.updates_until",
        params![token, payload_json, now, p.updates_until],
    )
    .map_err(|e| e.to_string())?;
    Ok(status_from(&db))
}

// ==================== 场景模板 ====================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TaxDef {
    #[serde(default)]
    mode: String,
    #[serde(default)]
    rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settlement {
    #[serde(default)]
    when: Vec<String>,
    #[serde(default)]
    account: String,
    #[serde(default, rename = "default")]
    is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntryTmpl {
    side: String,
    account: String,
    amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct EntryVariants {
    #[serde(default)]
    general: Vec<EntryTmpl>,
    #[serde(default)]
    small: Vec<EntryTmpl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioDef {
    id: String,
    name: String,
    #[serde(default)]
    version: i32,
    keywords: Vec<String>,
    #[serde(default)]
    direction: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    amount_tax: String,
    #[serde(default)]
    tax: TaxDef,
    #[serde(default)]
    settlement: Vec<Settlement>,
    entries: EntryVariants,
    #[serde(default)]
    priority: i32,
}

const BUILTIN_SCENARIOS: &str = r#"[
{"id":"sale_service","name":"收取服务费","version":1,
 "keywords":["服务费","技术服务费","咨询服务费","咨询费","服务收入"],
 "direction":"in","category":"sale","amount_tax":"inclusive","tax":{"mode":"output"},
 "settlement":[{"when":["未收","欠","应收","赊"],"account":"1122"},{"when":["现金"],"account":"1001"},{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"{{settlement}}","amount":"total"},{"side":"credit","account":"6001","amount":"net"},{"side":"credit","account":"2221","amount":"tax"}],
            "small":[{"side":"debit","account":"{{settlement}}","amount":"total"},{"side":"credit","account":"6001","amount":"net"},{"side":"credit","account":"2221","amount":"tax"}]}},
{"id":"sale_goods","name":"销售商品/收取货款","version":1,
 "keywords":["销售商品","卖货","货款","销售","卖出","销货"],
 "direction":"in","category":"sale","amount_tax":"inclusive","tax":{"mode":"output"},
 "settlement":[{"when":["未收","欠","应收","赊"],"account":"1122"},{"when":["现金"],"account":"1001"},{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"{{settlement}}","amount":"total"},{"side":"credit","account":"6001","amount":"net"},{"side":"credit","account":"2221","amount":"tax"}],
            "small":[{"side":"debit","account":"{{settlement}}","amount":"total"},{"side":"credit","account":"6001","amount":"net"},{"side":"credit","account":"2221","amount":"tax"}]}},
{"id":"purchase_material","name":"购买原材料","version":1,
 "keywords":["原材料","材料","原料"],
 "direction":"out","category":"purchase","amount_tax":"inclusive","tax":{"mode":"input"},
 "settlement":[{"when":["未付","赊","欠","应付"],"account":"2202"},{"when":["现金"],"account":"1001"},{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"1403","amount":"net"},{"side":"debit","account":"2221","amount":"tax"},{"side":"credit","account":"{{settlement}}","amount":"total"}],
            "small":[{"side":"debit","account":"1403","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}]}},
{"id":"purchase_inventory","name":"采购库存商品","version":1,
 "keywords":["库存商品","进货","采购商品","采购"],
 "direction":"out","category":"purchase","amount_tax":"inclusive","tax":{"mode":"input"},
 "settlement":[{"when":["未付","赊","欠","应付"],"account":"2202"},{"when":["现金"],"account":"1001"},{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"1405","amount":"net"},{"side":"debit","account":"2221","amount":"tax"},{"side":"credit","account":"{{settlement}}","amount":"total"}],
            "small":[{"side":"debit","account":"1405","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}]}},
{"id":"expense_office","name":"办公费用","version":1,
 "keywords":["办公费","办公用品","办公","文具","打印"],
 "direction":"out","category":"expense","amount_tax":"inclusive","tax":{"mode":"input"},
 "settlement":[{"when":["未付","赊","欠","应付"],"account":"2202"},{"when":["现金"],"account":"1001"},{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"6602","amount":"net"},{"side":"debit","account":"2221","amount":"tax"},{"side":"credit","account":"{{settlement}}","amount":"total"}],
            "small":[{"side":"debit","account":"6602","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}]}},
{"id":"expense_rent","name":"房租/租金","version":1,
 "keywords":["房租","租金","租赁费","租赁"],
 "direction":"out","category":"expense","amount_tax":"inclusive","tax":{"mode":"input"},
 "settlement":[{"when":["未付","赊","欠","应付"],"account":"2202"},{"when":["现金"],"account":"1001"},{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"6602","amount":"net"},{"side":"debit","account":"2221","amount":"tax"},{"side":"credit","account":"{{settlement}}","amount":"total"}],
            "small":[{"side":"debit","account":"6602","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}]}},
{"id":"expense_marketing","name":"推广/销售费用","version":1,
 "keywords":["广告","推广","营销","销售费用","宣传"],
 "direction":"out","category":"expense","amount_tax":"inclusive","tax":{"mode":"input"},
 "settlement":[{"when":["未付","赊","欠","应付"],"account":"2202"},{"when":["现金"],"account":"1001"},{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"6601","amount":"net"},{"side":"debit","account":"2221","amount":"tax"},{"side":"credit","account":"{{settlement}}","amount":"total"}],
            "small":[{"side":"debit","account":"6601","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}]}},
{"id":"loan_in","name":"收到借款","version":1,
 "keywords":["借款","借入","借到","收到借款","贷款"],
 "direction":"in","category":"financing","amount_tax":"none","tax":{"mode":"none"},
 "settlement":[{"when":["现金"],"account":"1001"},{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"{{settlement}}","amount":"total"},{"side":"credit","account":"2241","amount":"total"}],
            "small":[{"side":"debit","account":"{{settlement}}","amount":"total"},{"side":"credit","account":"2241","amount":"total"}]}},
{"id":"loan_repay","name":"归还借款","version":1,
 "keywords":["还款","归还借款","偿还借款","还借款"],
 "direction":"out","category":"financing","amount_tax":"none","tax":{"mode":"none"},
 "settlement":[{"when":["现金"],"account":"1001"},{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"2241","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}],
            "small":[{"side":"debit","account":"2241","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}]}},
{"id":"bank_fee","name":"银行手续费","version":1,
 "keywords":["手续费","银行手续费","账户管理费"],
 "direction":"out","category":"expense","amount_tax":"none","tax":{"mode":"none"},
 "settlement":[{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"6603","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}],
            "small":[{"side":"debit","account":"6603","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}]}},
{"id":"bank_interest","name":"银行利息收入","version":1,
 "keywords":["利息收入","结息","利息"],
 "direction":"in","category":"other","amount_tax":"none","tax":{"mode":"none"},
 "settlement":[{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"{{settlement}}","amount":"total"},{"side":"credit","account":"6603","amount":"total"}],
            "small":[{"side":"debit","account":"{{settlement}}","amount":"total"},{"side":"credit","account":"6603","amount":"total"}]}},
{"id":"tax_payment","name":"缴纳税费","version":1,
 "keywords":["缴税","缴纳税费","交税","缴纳增值税","缴个税","缴纳个税","申报缴税"],
 "direction":"out","category":"tax","amount_tax":"none","tax":{"mode":"none"},
 "settlement":[{"default":true,"account":"1002"}],
 "entries":{"general":[{"side":"debit","account":"2221","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}],
            "small":[{"side":"debit","account":"2221","amount":"total"},{"side":"credit","account":"{{settlement}}","amount":"total"}]}}
]"#;

fn module_dir() -> PathBuf {
    let mut p = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("small-business-manager");
    p.push("modules");
    p.push("smart_voucher");
    let _ = fs::create_dir_all(&p);
    p
}

fn rulepack_path() -> PathBuf {
    module_dir().join("rulepack.token")
}

fn rulepack_meta_path() -> PathBuf {
    module_dir().join("rulepack_meta.json")
}

/// 读取已下载并验签的规则包场景
fn downloaded_scenarios() -> Vec<ScenarioDef> {
    let mut out = Vec::new();
    if let Ok(token) = fs::read_to_string(rulepack_path()) {
        if let Ok(v) = verify_signed(token.trim()) {
            if v.get("type").and_then(|x| x.as_str()) == Some("rulepack") {
                if let Some(arr) = v.get("scenarios").and_then(|x| x.as_array()) {
                    for s in arr {
                        if let Ok(def) = serde_json::from_value::<ScenarioDef>(s.clone()) {
                            out.push(def);
                        }
                    }
                }
            }
        }
    }
    out
}

fn load_all_scenarios(conn: &Connection) -> Vec<ScenarioDef> {
    let mut v: Vec<ScenarioDef> = serde_json::from_str(BUILTIN_SCENARIOS).unwrap_or_default();
    // 已下发的规则包（覆盖在内置之后）
    v.extend(downloaded_scenarios());
    // 用户自定义（最高优先）
    if let Ok(mut stmt) = conn.prepare("SELECT definition FROM smart_scenarios WHERE enabled=1") {
        if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
            for d in rows.flatten() {
                if let Ok(s) = serde_json::from_str::<ScenarioDef>(&d) {
                    v.push(s);
                }
            }
        }
    }
    v
}

#[derive(Debug, Serialize)]
pub struct RulepackStatus {
    pub installed: bool,
    pub version: String,
    pub installed_at: String,
    pub scenario_count: i64,
}

fn rulepack_status_inner() -> RulepackStatus {
    if let Ok(s) = fs::read_to_string(rulepack_meta_path()) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
            return RulepackStatus {
                installed: true,
                version: v.get("version").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                installed_at: v.get("installed_at").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                scenario_count: v.get("scenario_count").and_then(|x| x.as_i64()).unwrap_or(0),
            };
        }
    }
    RulepackStatus {
        installed: false,
        version: String::new(),
        installed_at: String::new(),
        scenario_count: 0,
    }
}

#[tauri::command]
pub fn smart_rulepack_status(state: State<AppState>) -> Result<RulepackStatus, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_entitled(&db)?;
    Ok(rulepack_status_inner())
}

/// 从配置的 rulepack 地址下载并验签安装（地址返回 SV1 Token 或 {"token": "..."}）
#[tauri::command]
pub fn smart_rulepack_update(state: State<AppState>) -> Result<RulepackStatus, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_entitled(&db)?;
    let url = get_setting(&db, "smart_rulepack_url");
    if url.trim().is_empty() {
        return Err("未配置规则包地址（settings.smart_rulepack_url）".to_string());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(url.trim())
        .send()
        .map_err(|e| format!("下载规则包失败: {}", describe_reqwest_error(&e)))?;
    let status = resp.status();
    let body = resp.text().map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("下载规则包失败：服务器返回 {}", status));
    }
    let body = body.trim();
    let token = if body.starts_with("SV1.") {
        body.to_string()
    } else {
        let v: serde_json::Value =
            serde_json::from_str(body).map_err(|_| "规则包响应格式错误".to_string())?;
        v.get("token")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "规则包响应缺少 token 字段".to_string())?
    };
    let payload = verify_signed(&token)?;
    if payload.get("type").and_then(|x| x.as_str()) != Some("rulepack") {
        return Err("不是有效的规则包".to_string());
    }
    let version = payload
        .get("version")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let count = payload
        .get("scenarios")
        .and_then(|x| x.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    fs::write(rulepack_path(), token).map_err(|e| format!("写入规则包失败: {}", e))?;
    let meta = json!({
        "version": version,
        "installed_at": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "scenario_count": count
    });
    fs::write(rulepack_meta_path(), meta.to_string())
        .map_err(|e| format!("写入规则包元数据失败: {}", e))?;
    Ok(rulepack_status_inner())
}

#[tauri::command]
pub fn smart_scenarios_list(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_entitled(&db)?;
    let builtins: Vec<ScenarioDef> =
        serde_json::from_str(BUILTIN_SCENARIOS).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for s in builtins {
        let mut v = serde_json::to_value(&s).map_err(|e| e.to_string())?;
        v["is_builtin"] = json!(true);
        out.push(v);
    }
    let mut stmt = db
        .prepare("SELECT definition FROM smart_scenarios ORDER BY priority DESC, id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    for d in rows.flatten() {
        if let Ok(s) = serde_json::from_str::<ScenarioDef>(&d) {
            let mut v = serde_json::to_value(&s).map_err(|e| e.to_string())?;
            v["is_builtin"] = json!(false);
            out.push(v);
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn smart_scenario_save(state: State<AppState>, definition: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_entitled(&db)?;
    let s: ScenarioDef =
        serde_json::from_str(&definition).map_err(|e| format!("模板格式错误: {}", e))?;
    if s.id.trim().is_empty() || s.name.trim().is_empty() || s.keywords.is_empty() {
        return Err("模板需包含 id、name、keywords".to_string());
    }
    let def = serde_json::to_string(&s).map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO smart_scenarios(scenario_id, name, definition, is_builtin, enabled, priority)
         VALUES(?1, ?2, ?3, 0, 1, ?4)
         ON CONFLICT(scenario_id) DO UPDATE SET name=excluded.name, definition=excluded.definition,
           priority=excluded.priority, updated_at=datetime('now','localtime')",
        params![s.id, s.name, def, s.priority],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn smart_scenario_delete(state: State<AppState>, scenario_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_entitled(&db)?;
    db.execute("DELETE FROM smart_scenarios WHERE scenario_id=?1", params![scenario_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 解析 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftEntry {
    pub account_code: String,
    pub account_name: String,
    pub debit_amount: f64,
    pub credit_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDraft {
    pub raw_text: String,
    pub date: String,
    pub summary: String,
    pub scenario_id: String,
    pub scenario_name: String,
    pub entries: Vec<DraftEntry>,
    pub confidence: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SmartParseResult {
    pub drafts: Vec<SmartDraft>,
    pub notes: Vec<String>,
}

fn get_setting(conn: &Connection, key: &str) -> String {
    conn.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
        r.get::<_, String>(0)
    })
    .unwrap_or_default()
}

fn get_setting_f64(conn: &Connection, key: &str, default: f64) -> f64 {
    get_setting(conn, key).parse().unwrap_or(default)
}

fn set_setting(conn: &Connection, key: &str, value: &str) {
    let _ = conn.execute(
        "INSERT INTO settings(key, value) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=datetime('now','localtime')",
        params![key, value],
    );
}

// ==================== LLM 兜底（用户自带 Key） ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub provider: String,
}

fn llm_config(db: &Connection) -> LlmConfig {
    LlmConfig {
        enabled: get_setting(db, "smart_llm_enabled") == "1",
        endpoint: get_setting(db, "smart_llm_endpoint"),
        api_key: get_setting(db, "smart_llm_api_key"),
        model: get_setting(db, "smart_llm_model"),
        provider: get_setting(db, "smart_llm_provider"),
    }
}

#[tauri::command]
pub fn smart_llm_get_config(state: State<AppState>) -> Result<LlmConfig, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_entitled(&db)?;
    Ok(llm_config(&db))
}

#[tauri::command]
pub fn smart_llm_save_config(state: State<AppState>, config: LlmConfig) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_entitled(&db)?;
    set_setting(&db, "smart_llm_enabled", if config.enabled { "1" } else { "0" });
    set_setting(&db, "smart_llm_endpoint", &config.endpoint);
    set_setting(&db, "smart_llm_api_key", &config.api_key);
    set_setting(&db, "smart_llm_model", &config.model);
    set_setting(&db, "smart_llm_provider", &config.provider);
    Ok(())
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn llm_infer(
    cfg: &LlmConfig,
    clause: &str,
    amount: f64,
    date: &str,
    names: &HashMap<String, String>,
) -> Option<(String, Vec<DraftEntry>, String)> {
    if !cfg.enabled || cfg.api_key.trim().is_empty() || cfg.endpoint.trim().is_empty() {
        return None;
    }
    let accounts: Vec<String> = names.iter().map(|(c, n)| format!("{}:{}", c, n)).collect();
    let system = "你是会计助手。只能使用给定的会计科目代码，不得臆造。金额为含税金额，纳税人类型为小规模纳税人，采购进项税不得抵扣（全额计入成本或费用）。只输出 JSON，不要解释。";
    let user = format!(
        "业务描述：{}\n金额：{:.2} 元\n日期：{}\n可选科目（代码:名称）：{}\n请输出 JSON：{{\"summary\":\"摘要\",\"entries\":[{{\"account_code\":\"1002\",\"debit\":100.0,\"credit\":0.0}}],\"confidence\":0.8}}",
        clause,
        amount,
        date,
        accounts.join("; ")
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .ok()?;
    let resp = client
        .post(cfg.endpoint.trim())
        .bearer_auth(cfg.api_key.trim())
        .json(&json!({
            "model": cfg.model,
            "temperature": 0,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ],
            "response_format": {"type": "json_object"}
        }))
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().ok()?;
    let content = v
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?
        .as_str()?
        .to_string();
    let cleaned = content.trim();
    let cleaned = cleaned.strip_prefix("```json").unwrap_or(cleaned);
    let cleaned = cleaned.strip_prefix("```").unwrap_or(cleaned);
    let cleaned = cleaned.strip_suffix("```").unwrap_or(cleaned).trim();
    let obj: serde_json::Value = serde_json::from_str(cleaned).ok()?;
    let summary = obj
        .get("summary")
        .and_then(|x| x.as_str())
        .unwrap_or(clause)
        .to_string();
    let mut entries = Vec::new();
    for e in obj.get("entries")?.as_array()? {
        let code = e.get("account_code")?.as_str()?.to_string();
        if !names.contains_key(&code) {
            return None;
        }
        let d = e.get("debit").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let c = e.get("credit").and_then(|x| x.as_f64()).unwrap_or(0.0);
        entries.push(DraftEntry {
            account_name: names.get(&code).cloned().unwrap_or_default(),
            account_code: code,
            debit_amount: round2(d),
            credit_amount: round2(c),
        });
    }
    if entries.is_empty() {
        return None;
    }
    let td: f64 = entries.iter().map(|e| e.debit_amount).sum();
    let tc: f64 = entries.iter().map(|e| e.credit_amount).sum();
    if (td - tc).abs() > 0.005 || td <= 0.0 {
        return None;
    }
    let conf = obj.get("confidence").and_then(|x| x.as_f64()).unwrap_or(0.6);
    Some((
        summary,
        entries,
        format!("由 LLM 识别（置信度 {:.0}%）", conf * 100.0),
    ))
}

fn account_names(conn: &Connection) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT code, name FROM accounts") {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) {
            for (c, n) in rows.flatten() {
                m.insert(c, n);
            }
        }
    }
    m
}

fn shift(base: NaiveDate, delta: i64) -> NaiveDate {
    if delta >= 0 {
        base.checked_add_days(Days::new(delta as u64)).unwrap_or(base)
    } else {
        base.checked_sub_days(Days::new((-delta) as u64)).unwrap_or(base)
    }
}

fn split_clauses(text: &str) -> Vec<String> {
    let mut norm = String::new();
    for ch in text.chars() {
        match ch {
            '，' | ',' | '。' | '；' | ';' | '\n' | '、' => norm.push('\n'),
            _ => norm.push(ch),
        }
    }
    for kw in ["另外", "还有", "以及", "同时"] {
        norm = norm.replace(kw, "\n");
    }
    norm.split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn extract_date(text: &str, base: NaiveDate) -> (String, String) {
    let mut t = text.to_string();
    for (kw, delta) in [("前天", -2i64), ("昨天", -1), ("昨日", -1), ("今天", 0), ("今日", 0)] {
        if t.contains(kw) {
            let d = shift(base, delta);
            t = t.replace(kw, " ");
            return (d.format("%Y-%m-%d").to_string(), t);
        }
    }
    if let Ok(re) = Regex::new(r"(\d{4})[-/年](\d{1,2})[-/月](\d{1,2})日?") {
        if let Some(c) = re.captures(&t) {
            let y: i32 = c[1].parse().unwrap_or(base.year());
            let m: u32 = c[2].parse().unwrap_or(1);
            let d: u32 = c[3].parse().unwrap_or(1);
            if let Some(nd) = NaiveDate::from_ymd_opt(y, m, d) {
                let s = nd.format("%Y-%m-%d").to_string();
                t = re.replace(&t, " ").to_string();
                return (s, t);
            }
        }
    }
    if let Ok(re) = Regex::new(r"(\d{1,2})月(\d{1,2})日?") {
        if let Some(c) = re.captures(&t) {
            let m: u32 = c[1].parse().unwrap_or(1);
            let d: u32 = c[2].parse().unwrap_or(1);
            if let Some(nd) = NaiveDate::from_ymd_opt(base.year(), m, d) {
                let s = nd.format("%Y-%m-%d").to_string();
                t = re.replace(&t, " ").to_string();
                return (s, t);
            }
        }
    }
    (base.format("%Y-%m-%d").to_string(), t)
}

fn extract_amount(text: &str) -> Option<f64> {
    let unit = Regex::new(r"(\d+(?:\.\d+)?)\s*(万|元|块)").ok()?;
    if let Some(c) = unit.captures(text) {
        let v: f64 = c[1].parse().ok()?;
        return Some(if &c[2] == "万" { v * 10000.0 } else { v });
    }
    let num = Regex::new(r"\d+(?:\.\d+)?").ok()?;
    let mut last = None;
    for m in num.find_iter(text) {
        if let Ok(v) = m.as_str().parse::<f64>() {
            last = Some(v);
        }
    }
    last
}

fn period_sales(conn: &Connection, date: &str) -> f64 {
    if date.len() < 7 {
        return 0.0;
    }
    let start = format!("{}-01", &date[..7]);
    conn.query_row(
        "SELECT COALESCE(SUM(ve.credit_amount),0)
         FROM voucher_entries ve JOIN accounts a ON ve.account_id=a.id
         JOIN vouchers v ON ve.voucher_id=v.id
         WHERE a.code IN ('6001','6051') AND v.date>=?1 AND v.date<=?2",
        params![start, date],
        |r| r.get(0),
    )
    .unwrap_or(0.0)
}

#[tauri::command]
pub fn smart_parse(
    state: State<AppState>,
    text: String,
    date: Option<String>,
) -> Result<SmartParseResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_entitled(&db)?;
    parse_text(&db, &text, date)
}

fn parse_text(db: &Connection, text: &str, date: Option<String>) -> Result<SmartParseResult, String> {
    let base = date
        .as_deref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| Local::now().date_naive());
    let scenarios = load_all_scenarios(db);
    let names = account_names(db);
    let taxpayer = get_setting(db, "taxpayer_type");
    let is_small = taxpayer.contains("小规模") || taxpayer.is_empty();
    let rate_default = if is_small {
        get_setting_f64(db, "small_vat_rate", 0.03)
    } else {
        get_setting_f64(db, "vat_rate", 0.13)
    };
    let threshold = get_setting_f64(db, "small_exempt_threshold_month", 100000.0);
    let llm_cfg = llm_config(db);

    let mut drafts = Vec::new();
    let mut notes = Vec::new();

    for clause in split_clauses(text) {
        let (d, rest) = extract_date(&clause, base);
        let amount = match extract_amount(&rest) {
            Some(a) if a > 0.0 => a,
            _ => {
                notes.push(format!("无法识别金额：{}", clause));
                continue;
            }
        };
        let lower = clause.to_lowercase();

        let mut best: Option<&ScenarioDef> = None;
        let mut best_score = i32::MIN;
        for s in &scenarios {
            let mut score = s.priority;
            let mut hits = 0i32;
            for kw in &s.keywords {
                if lower.contains(&kw.to_lowercase()) {
                    hits += 1;
                    score += 100 + kw.chars().count() as i32;
                }
            }
            if hits > 0 && score > best_score {
                best_score = score;
                best = Some(s);
            }
        }
        let scen = match best {
            Some(s) => s,
            None => {
                // 规则未命中时，若开启 LLM 兜底则尝试云端识别（用户自带 Key）
                if let Some((summary, entries, note)) =
                    llm_infer(&llm_cfg, &clause, amount, &d, &names)
                {
                    drafts.push(SmartDraft {
                        raw_text: clause.clone(),
                        date: d.clone(),
                        summary,
                        scenario_id: "llm".to_string(),
                        scenario_name: "LLM 识别".to_string(),
                        entries,
                        confidence: 0.6,
                        warnings: vec![note],
                    });
                    continue;
                }
                notes.push(format!("未匹配业务类型：{}", clause));
                continue;
            }
        };

        // 结算科目
        let mut settle: Option<String> = None;
        let mut settle_default = true;
        for rule in &scen.settlement {
            if rule.when.iter().any(|w| lower.contains(&w.to_lowercase())) {
                settle = Some(rule.account.clone());
                settle_default = false;
                break;
            }
        }
        if settle.is_none() {
            settle = scen
                .settlement
                .iter()
                .find(|r| r.is_default)
                .map(|r| r.account.clone());
        }
        let settle_code = settle.unwrap_or_else(|| "1002".to_string());

        // 税额
        let mode = scen.tax.mode.as_str();
        let rate = scen.tax.rate.unwrap_or(rate_default);
        let tax_applies = match mode {
            "output" => true,
            "input" => !is_small,
            _ => false,
        };
        let variant = if is_small && !scen.entries.small.is_empty() {
            &scen.entries.small
        } else {
            &scen.entries.general
        };

        let net = if tax_applies { round2(amount / (1.0 + rate)) } else { amount };
        let tax = if tax_applies { round2(amount - net) } else { 0.0 };

        let mut entries: Vec<DraftEntry> = Vec::new();
        for et in variant {
            let code = if et.account == "{{settlement}}" {
                settle_code.clone()
            } else {
                et.account.clone()
            };
            let amt = match et.amount.as_str() {
                "net" => net,
                "tax" => tax,
                _ => amount,
            };
            let (dr, cr) = if et.side == "debit" { (amt, 0.0) } else { (0.0, amt) };
            entries.push(DraftEntry {
                account_name: names.get(&code).cloned().unwrap_or_default(),
                account_code: code,
                debit_amount: round2(dr),
                credit_amount: round2(cr),
            });
        }

        // 平衡
        let td: f64 = entries.iter().map(|e| e.debit_amount).sum();
        let tc: f64 = entries.iter().map(|e| e.credit_amount).sum();
        let diff = round2(td - tc);
        if diff.abs() > 0.005 {
            if diff > 0.0 {
                if let Some(e) = entries.iter_mut().find(|e| e.credit_amount > 0.0) {
                    e.credit_amount = round2(e.credit_amount + diff);
                }
            } else if let Some(e) = entries.iter_mut().find(|e| e.debit_amount > 0.0) {
                e.debit_amount = round2(e.debit_amount - diff);
            }
        }

        let mut warnings = Vec::new();
        if settle_default && scen.settlement.iter().any(|r| r.is_default) {
            warnings.push("未说明结算方式，默认按银行存款/现金科目处理".to_string());
        }
        if tax_applies {
            warnings.push(format!("金额按含税处理，税率 {:.0}%", rate * 100.0));
        }
        if is_small && mode == "output" {
            let sales = period_sales(db, &d);
            if sales + net <= threshold {
                warnings.push(format!(
                    "本笔后当月销售额约 {:.2} 元，未超小规模免税额度 {:.0} 元，可能适用免税，请确认",
                    sales + net,
                    threshold
                ));
            }
        }

        let confidence = ((0.60 + (best_score.max(0) as f64 / 320.0)).min(0.98) * 100.0).round() / 100.0;

        drafts.push(SmartDraft {
            raw_text: clause.clone(),
            date: d,
            summary: clause.clone(),
            scenario_id: scen.id.clone(),
            scenario_name: scen.name.clone(),
            entries,
            confidence,
            warnings,
        });
    }

    Ok(SmartParseResult { drafts, notes })
}

#[tauri::command]
pub fn smart_commit(state: State<AppState>, drafts: Vec<SmartDraft>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_entitled(&db)?;

    let mut id_map = HashMap::new();
    {
        let mut stmt = db.prepare("SELECT code, id FROM accounts").map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;
        for (c, i) in rows.flatten() {
            id_map.insert(c, i);
        }
    }

    let mut out = Vec::new();
    for d in &drafts {
        if d.entries.is_empty() {
            continue;
        }
        let td: f64 = d.entries.iter().map(|e| e.debit_amount).sum();
        let tc: f64 = d.entries.iter().map(|e| e.credit_amount).sum();
        if (td - tc).abs() > 0.005 {
            return Err(format!("凭证借贷不平衡：{}", d.summary));
        }
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM vouchers WHERE date=?1", [&d.date], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let voucher_no = format!("{}-{:04}", d.date.replace('-', ""), count + 1);
        db.execute(
            "INSERT INTO vouchers(voucher_no, date, summary) VALUES(?1, ?2, ?3)",
            params![voucher_no, d.date, d.summary],
        )
        .map_err(|e| e.to_string())?;
        let vid = db.last_insert_rowid();
        for e in &d.entries {
            let aid = *id_map
                .get(&e.account_code)
                .ok_or_else(|| format!("科目不存在: {}", e.account_code))?;
            db.execute(
                "INSERT INTO voucher_entries(voucher_id, account_id, debit_amount, credit_amount, summary)
                 VALUES(?1, ?2, ?3, ?4, ?5)",
                params![vid, aid, e.debit_amount, e.credit_amount, d.summary],
            )
            .map_err(|e| e.to_string())?;
        }
        let entries_json = serde_json::to_string(&d.entries).unwrap_or_default();
        db.execute(
            "INSERT INTO smart_learning_samples(raw_text, scenario_id, entries, adopted) VALUES(?1,?2,?3,1)",
            params![d.raw_text, d.scenario_id, entries_json],
        )
        .ok();
        out.push(voucher_no);
    }
    Ok(out)
}

