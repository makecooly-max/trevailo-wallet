use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct NodeClient {
    pub base_url: String,
    client: reqwest::blocking::Client,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct NodeInfo {
    pub version: String,
    pub network: String,
    pub height: u64,
    pub tip_hash: String,
    pub difficulty: u32,
    pub mempool_size: usize,
    pub utxo_count: usize,
    pub block_reward_tvc: f64,
    /// true пока следующий блок находится в защищённом периоде (первые 10 000 блоков)
    #[serde(default)]
    pub protected_period: bool,
    /// Сколько блоков осталось до конца защищённого периода
    #[serde(default)]
    pub protected_blocks_remaining: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct BalanceInfo {
    pub address: String,
    pub balance_trev: u64,
    pub balance_tvc: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoInfo {
    pub tx_hash: String,
    pub output_index: u32,
    pub amount: u64,
    pub recipient: String,
    pub spent: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TxResponse {
    pub tx_hash: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MempoolStats {
    pub total: usize,
    pub valid: usize,
    pub min_fee: u64,
    pub max_fee: u64,
    pub avg_fee: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MineResponse {
    pub block_hash: String,
    pub height: u64,
    pub transactions: usize,
    pub nonce: u64,
    pub reward_tvc: f64,
    /// true если блок был намайнен в защищённый период
    #[serde(default)]
    pub protected_period: bool,
}

/// Статус транзакции для отображения в UI
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TxStatusKind {
    /// Только что отправлена, ждём подтверждения от ноды
    Sending,
    /// В мемпуле, ждёт майнинга
    Pending,
    /// Подтверждена в блоке
    Confirmed { block_height: u64, confirmations: u64 },
    /// Ошибка
    Failed(String),
}

/// Исходящая транзакция с трекингом статуса
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PendingTx {
    pub hash: String,
    pub to: String,
    pub amount_tvc: f64,
    pub fee_tvc: f64,
    pub status: TxStatusKind,
    pub sent_at: std::time::Instant,
}

impl NodeClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Trevailo-Wallet/0.1.0")
            .build()
            .context("Failed to create HTTP client")?;
        Ok(NodeClient {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        })
    }

    pub fn health_check(&self) -> bool {
        self.client
            .get(format!("{}/health", self.base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub fn node_info(&self) -> Result<NodeInfo> {
        self.get("/info")
    }

    pub fn balance(&self, address: &str) -> Result<BalanceInfo> {
        self.get(&format!("/wallet/{}/balance", address))
    }

    pub fn utxos(&self, address: &str) -> Result<Vec<UtxoInfo>> {
        self.get(&format!("/wallet/{}/utxos", address))
    }

    #[allow(dead_code)]
    pub fn mempool_stats(&self) -> Result<MempoolStats> {
        self.get("/mempool/stats")
    }

    /// Проверить статус транзакции
    pub fn tx_status(&self, hash: &str) -> Result<TxStatusKind> {
        let val: serde_json::Value = self.get(&format!("/tx/{}", hash))?;

        if let Some(err) = val.get("error") {
            return Ok(TxStatusKind::Failed(
                err.as_str().unwrap_or("unknown").to_string(),
            ));
        }

        match val.get("status").and_then(|s| s.as_str()) {
            Some("pending") => Ok(TxStatusKind::Pending),
            Some("confirmed") => {
                let block_height = val
                    .get("block_height")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let confirmations = val
                    .get("confirmations")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1);
                Ok(TxStatusKind::Confirmed {
                    block_height,
                    confirmations,
                })
            }
            _ => Ok(TxStatusKind::Failed("Transaction not found".to_string())),
        }
    }

    /// Майнинг из кошелька.
    ///
    /// В ноду уходит только (miner_address, public_key, signature, timestamp) — приватный ключ
    /// используется только для подписи локально.
    ///
    /// Возвращает Ok(MineResponse) при успехе.
    /// В защищённый период нода может вернуть:
    ///   - 429 (rate limit) — слишком рано, нужно подождать
    ///   - 409 (nonce not found) — не повезло, можно попробовать сразу
    pub fn mine_with_private_key(&self, private_key_hex: &str) -> Result<MineResponse> {
    use trevailo_core::wallet::Wallet;
    use trevailo_core::utils::now_unix;

    let wallet = Wallet::from_private_key(private_key_hex)
        .context("Invalid private key")?;

    let miner_address = wallet.address();
    let public_key = wallet.public_key.clone();
    let timestamp = now_unix();

    // Формируем подпись: "mine_request:{адрес}:{timestamp}"
    let signing_data = format!("mine_request:{}:{}", miner_address, timestamp);
    let signature = wallet.sign(signing_data.as_bytes());

    let body = serde_json::json!({
        "miner_address": miner_address,
        "public_key": public_key,
        "signature": signature,
        "timestamp": timestamp,
    });

    self.post("/mine", &body)
}

    /// Подписать транзакцию локально и отправить через /tx/broadcast.
    ///
    /// Приватный ключ НЕ покидает машину пользователя — по сети уходит
    /// только готовая подписанная транзакция.
    pub fn send_signed_tx(
        &self,
        private_key_hex: &str,
        to: &str,
        amount_tvc: f64,
        fee_tvc: f64,
    ) -> Result<PendingTx> {
        use trevailo_core::blockchain::types::COIN;
        use trevailo_core::wallet::Wallet;

        // 1. Восстанавливаем кошелёк из приватного ключа — только в памяти
        let wallet = Wallet::from_private_key(private_key_hex)
            .context("Invalid private key")?;

        // 2. Получаем UTXO с ноды
        let utxos_raw: Vec<UtxoInfo> = self
            .utxos(&wallet.address())
            .context("Failed to fetch UTXOs")?;

        if utxos_raw.is_empty() {
            anyhow::bail!("No UTXOs available — insufficient balance");
        }

        // 3. Конвертируем UtxoInfo → типы из trevailo_core
        let utxos: Vec<trevailo_core::blockchain::types::Utxo> = utxos_raw
            .iter()
            .map(|u| trevailo_core::blockchain::types::Utxo {
                tx_hash: u.tx_hash.clone(),
                output_index: u.output_index,
                amount: u.amount,
                recipient: u.recipient.clone(),
                spent: u.spent,
            })
            .collect();

        // 4. Строим и подписываем транзакцию локально
        let amount_trev = (amount_tvc * COIN as f64) as u64;
        let fee_trev = (fee_tvc * COIN as f64) as u64;

        let tx = wallet
            .create_transfer(&utxos, &to.to_string(), amount_trev, fee_trev)
            .context("Failed to build transaction")?;

        let tx_hash = tx.hash.clone();

        // 5. Отправляем подписанную транзакцию на /tx/broadcast
        //    Приватный ключ в теле запроса отсутствует
        let body = serde_json::json!({ "transaction": tx });
        let _resp: TxResponse = self
            .post("/tx/broadcast", &body)
            .context("Broadcast failed")?;

        Ok(PendingTx {
            hash: tx_hash,
            to: to.to_string(),
            amount_tvc,
            fee_tvc,
            status: TxStatusKind::Pending,
            sent_at: std::time::Instant::now(),
        })
    }

    fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .with_context(|| format!("GET {} failed", path))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body: serde_json::Value = resp.json().unwrap_or_default();
            anyhow::bail!(
                "API {} {}: {}",
                status.as_u16(),
                path,
                body["error"].as_str().unwrap_or("?")
            );
        }
        resp.json().context("Parse failed")
    }

    fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        let resp = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .json(body)
            .send()
            .with_context(|| format!("POST {} failed", path))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let b: serde_json::Value = resp.json().unwrap_or_default();
            anyhow::bail!(
                "API {} {}: {}",
                status.as_u16(),
                path,
                b["error"].as_str()
                    .or_else(|| b["message"].as_str())
                    .unwrap_or("?")
            );
        }
        resp.json().context("Parse failed")
    }
}