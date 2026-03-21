use crate::keystore_manager::{KeystoreEntry, KeystoreManager, LoadedWallet};
use crate::node_client::{
    BalanceInfo, MineResponse, NodeClient, NodeInfo, PendingTx, TxStatusKind, UtxoInfo,
};
use egui::{Color32, FontId, RichText};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    WalletSelector,
    Unlock(KeystoreEntry),
    CreateWallet,
    ImportWallet,
    Dashboard,
    Mining,
    Send,
    Utxos,
    Receive,
    Buy,
    Settings,
}

#[derive(Debug, Clone)]
pub enum MiningOutcome {
    Success(MineResponse),
    RateLimited(u64),
    NonceNotFound,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuyOrderStatus {
    Idle,
    CreatingOrder,
    WaitingPayment { order_id: String },
    Polling { order_id: String, attempts: u32 },
    Delivered { order_id: String, tx_hash: String, tvc_delivered: f64 },
    Error(String),
}

pub struct TrevailoWallet {
    pub screen: Screen,
    pub keystore_manager: Option<KeystoreManager>,
    pub current_wallet: Option<LoadedWallet>,
    pub node_client: NodeClient,

    pub node_info: Option<NodeInfo>,
    pub balance: Option<BalanceInfo>,
    pub utxos: Vec<UtxoInfo>,
    pub last_refresh: Instant,

    pub pending_txs: Vec<PendingTx>,
    last_tx_poll: Instant,

    pub form_wallet_name: String,
    pub form_passphrase: String,
    pub form_passphrase_confirm: String,
    pub form_import_key: String,
    pub form_show_passphrase: bool,

    pub form_old_passphrase: String,
    pub form_new_passphrase: String,
    pub form_new_passphrase_confirm: String,

    pub send_to: String,
    pub send_amount: String,
    pub send_fee: String,

    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub node_connected: bool,
    pub auto_lock_timer: Instant,
    pub auto_lock_secs: u64,

    pending_txs_owner: Option<String>,
    pub confirm_delete: bool,

    pub buy_tvc_amount: String,
    pub buy_provider: String,
    pub buy_status: BuyOrderStatus,
    pub buy_price_usd: f64,

    pub buy_min_tvc: f64,
    pub buy_max_tvc: f64,
    pub buy_available_tvc: f64,
    /// Когда последний раз обновляли доступность
    pub buy_availability_last_fetch: std::time::Instant,
    pub buy_price_last_fetch: std::time::Instant,
    pub buy_poll_timer: std::time::Instant,

    pub mining_in_progress: bool,
    pub mining_auto_enabled: bool,
    pub mining_auto_interval_secs: u64,
    pub mining_task_rx: Option<Receiver<MiningOutcome>>,
    pub mining_auto_stop: Option<Arc<AtomicBool>>,
    pub last_mining: Option<MineResponse>,
}

pub const NODE_URL: &str = "http://31.131.21.11:8080";
pub const PAYMENT_URL: &str = "http://31.131.21.11:3000";

const KEY_AUTO_LOCK: &str = "auto_lock_secs";

impl TrevailoWallet {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let fonts = egui::FontDefinitions::default();
        cc.egui_ctx.set_fonts(fonts);

        let auto_lock_secs: u64 = cc
            .storage
            .and_then(|s| s.get_string(KEY_AUTO_LOCK))
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        let node_client = NodeClient::new(NODE_URL).expect("Failed to create HTTP client");

        let wallets_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("trevailo-wallet");

        let keystore_manager = KeystoreManager::new(&wallets_dir).ok();

        let mut app = TrevailoWallet {
            screen: Screen::WalletSelector,
            keystore_manager,
            current_wallet: None,
            node_client,
            node_info: None,
            balance: None,
            utxos: vec![],
            last_refresh: Instant::now().checked_sub(Duration::from_secs(60)).unwrap_or_else(Instant::now),
            pending_txs: vec![],
            last_tx_poll: Instant::now(),
            form_wallet_name: String::new(),
            form_passphrase: String::new(),
            form_passphrase_confirm: String::new(),
            form_import_key: String::new(),
            form_show_passphrase: false,
            form_old_passphrase: String::new(),
            form_new_passphrase: String::new(),
            form_new_passphrase_confirm: String::new(),
            send_to: String::new(),
            send_amount: String::new(),
            send_fee: "0.001".to_string(),
            error_message: None,
            success_message: None,
            node_connected: false,
            auto_lock_timer: Instant::now(),
            auto_lock_secs,
            pending_txs_owner: None,
            confirm_delete: false,
            buy_tvc_amount: "100".to_string(),
            buy_provider: "nowpayments".to_string(),
            buy_status: BuyOrderStatus::Idle,
            buy_price_usd: 0.10,
            buy_min_tvc: 120.0,
            buy_max_tvc: 100_000.0,
            buy_available_tvc: 0.0,
            buy_availability_last_fetch: Instant::now().checked_sub(Duration::from_secs(60)).unwrap_or_else(Instant::now),
            buy_price_last_fetch: Instant::now().checked_sub(Duration::from_secs(60)).unwrap_or_else(Instant::now),
            buy_poll_timer: std::time::Instant::now(),

            mining_in_progress: false,
            mining_auto_enabled: false,
            mining_auto_interval_secs: 60,
            mining_task_rx: None,
            mining_auto_stop: None,
            last_mining: None,
        };

        app.node_connected = app.node_client.health_check();
        app
    }

    pub fn switch_wallet(&mut self, wallet: LoadedWallet) {
        let new_address = wallet.address.clone();
        if self.pending_txs_owner.as_deref() != Some(&new_address) {
            self.pending_txs.clear();
            self.pending_txs_owner = Some(new_address);
        }
        self.balance = None;
        self.utxos = vec![];
        self.last_refresh = Instant::now().checked_sub(Duration::from_secs(60)).unwrap_or_else(Instant::now);
        self.current_wallet = Some(wallet);
        self.confirm_delete = false;
        self.clear_messages();

        self.mining_in_progress = false;
        self.mining_auto_enabled = false;
        self.mining_task_rx = None;
        if let Some(stop) = &self.mining_auto_stop {
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.mining_auto_stop = None;
        self.last_mining = None;
    }

    pub fn logout(&mut self) {
        if let Some(w) = &mut self.current_wallet {
            w.lock();
        }

        if let Some(stop) = &self.mining_auto_stop {
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.mining_auto_enabled = false;
        self.mining_auto_stop = None;
        self.mining_in_progress = false;
        self.mining_task_rx = None;
        self.last_mining = None;

        self.balance = None;
        self.utxos = vec![];
        self.screen = Screen::WalletSelector;
        self.confirm_delete = false;
        self.clear_messages();
    }

    pub fn refresh_data(&mut self) {
        if self.last_refresh.elapsed() < Duration::from_secs(10) {
            return;
        }
        self.last_refresh = Instant::now();
        self.node_connected = self.node_client.health_check();

        if let Ok(info) = self.node_client.node_info() {
            self.node_info = Some(info);
        }

        if let Some(wallet) = &self.current_wallet {
            let address = wallet.address.clone();
            if let Ok(bal) = self.node_client.balance(&address) {
                self.balance = Some(bal);
            }
            if let Ok(utxos) = self.node_client.utxos(&address) {
                self.utxos = utxos;
            }
        }
    }

    pub fn refresh_tx_statuses(&mut self) {
        if self.last_tx_poll.elapsed() < Duration::from_secs(3) {
            return;
        }
        self.last_tx_poll = Instant::now();

        let current_addr = self.current_wallet.as_ref().map(|w| w.address.clone());
        if current_addr != self.pending_txs_owner {
            return;
        }

        let mut balance_changed = false;

        for tx in &mut self.pending_txs {
            if matches!(tx.status, TxStatusKind::Confirmed { .. } | TxStatusKind::Failed(_)) {
                continue;
            }
            if tx.sent_at.elapsed() > Duration::from_secs(600) {
                tx.status = TxStatusKind::Failed("Таймаут — блок не найден за 10 мин".to_string());
                continue;
            }
            match self.node_client.tx_status(&tx.hash) {
                Ok(new_status) => {
                    let was_pending = tx.status == TxStatusKind::Pending;
                    let now_confirmed = matches!(new_status, TxStatusKind::Confirmed { .. });
                    tx.status = new_status;
                    if was_pending && now_confirmed {
                        balance_changed = true;
                    }
                }
                Err(_) => {}
            }
        }

        self.pending_txs.retain(|tx| {
            !matches!(tx.status, TxStatusKind::Confirmed { .. })
                || tx.sent_at.elapsed() < Duration::from_secs(120)
        });

        if balance_changed {
            self.last_refresh = Instant::now().checked_sub(Duration::from_secs(60)).unwrap_or_else(Instant::now);
        }
    }

    pub fn check_auto_lock(&mut self) {
        if self.current_wallet.is_some()
            && self.auto_lock_timer.elapsed().as_secs() > self.auto_lock_secs
        {
            self.logout();
        }
    }

    pub fn reset_auto_lock_timer(&mut self) {
        self.auto_lock_timer = Instant::now();
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
        self.success_message = None;
    }

    pub fn set_success(&mut self, msg: impl Into<String>) {
        self.success_message = Some(msg.into());
        self.error_message = None;
    }

    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }

    pub fn pending_count(&self) -> usize {
        self.pending_txs
            .iter()
            .filter(|tx| matches!(tx.status, TxStatusKind::Pending | TxStatusKind::Sending))
            .count()
    }
}

impl eframe::App for TrevailoWallet {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(KEY_AUTO_LOCK, self.auto_lock_secs.to_string());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_auto_lock();

        if matches!(
            self.screen,
            Screen::Dashboard
                | Screen::Mining
                | Screen::Send
                | Screen::Utxos
                | Screen::Receive
                | Screen::Buy
        ) {
            self.refresh_data();
            self.refresh_tx_statuses();
            let repaint_interval = if self.pending_count() > 0 {
                Duration::from_secs(3)
            } else {
                Duration::from_secs(10)
            };
            ctx.request_repaint_after(repaint_interval);
        }

        if ctx.input(|i| i.pointer.any_click() || i.key_pressed(egui::Key::Space)) {
            self.reset_auto_lock_timer();
        }

        self.render_top_bar(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.screen.clone() {
                    Screen::WalletSelector => crate::ui::wallet_selector::render(self, ui),
                    Screen::Unlock(entry) => crate::ui::unlock::render(self, ui, entry),
                    Screen::CreateWallet => crate::ui::create_wallet::render(self, ui),
                    Screen::ImportWallet => crate::ui::import_wallet::render(self, ui),
                    Screen::Dashboard => crate::ui::dashboard::render(self, ui),
                    Screen::Mining => crate::ui::mining::render(self, ui),
                    Screen::Send => crate::ui::send::render(self, ui),
                    Screen::Utxos => crate::ui::utxos::render(self, ui),
                    Screen::Receive => crate::ui::receive::render(self, ui),
                    Screen::Buy => crate::ui::buy::render(self, ui),
                    Screen::Settings => crate::ui::settings::render(self, ui),
                }
            });
        });
    }
}

impl TrevailoWallet {
    fn render_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("₮ Trevailo Wallet")
                        .font(FontId::proportional(18.0))
                        .color(Color32::from_rgb(99, 102, 241)),
                );

                ui.separator();

                let pending_count = self.pending_count();
                if pending_count > 0 {
                    ui.label(
                        RichText::new(format!("⏳ {} pending", pending_count))
                            .color(Color32::from_rgb(234, 179, 8))
                            .size(12.0),
                    );
                    ui.separator();
                }

                let is_auth_screen = matches!(
                    self.screen,
                    Screen::WalletSelector | Screen::CreateWallet | Screen::ImportWallet
                );

                if !is_auth_screen {
                    if matches!(self.screen, Screen::Unlock(_)) {
                        if ui.small_button("← Назад").clicked() {
                            self.screen = Screen::WalletSelector;
                        }
                    } else {
                        if ui
                            .selectable_label(self.screen == Screen::Dashboard, "📊 Главная")
                            .clicked()
                        {
                            self.screen = Screen::Dashboard;
                        }
                        if ui
                            .selectable_label(self.screen == Screen::Mining, "⛏️ Майнинг")
                            .clicked()
                        {
                            self.screen = Screen::Mining;
                        }
                        if ui
                            .selectable_label(self.screen == Screen::Send, "↑ Отправить")
                            .clicked()
                        {
                            self.screen = Screen::Send;
                        }
                        if ui
                            .selectable_label(self.screen == Screen::Receive, "↓ Получить")
                            .clicked()
                        {
                            self.screen = Screen::Receive;
                        }
                        if ui
                            .selectable_label(self.screen == Screen::Buy, "💳 Купить TVC")
                            .clicked()
                        {
                            self.screen = Screen::Buy;
                        }
                        if ui
                            .selectable_label(self.screen == Screen::Utxos, "🪙 UTXO")
                            .clicked()
                        {
                            self.screen = Screen::Utxos;
                        }
                        if ui
                            .selectable_label(self.screen == Screen::Settings, "⚙ Настройки")
                            .clicked()
                        {
                            self.screen = Screen::Settings;
                            self.confirm_delete = false;
                        }

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.button("🔒 Выйти").clicked() {
                                    self.logout();
                                }

                                let (dot, label) = if self.node_connected {
                                    (Color32::from_rgb(34, 197, 94), "Подключено")
                                } else {
                                    (Color32::from_rgb(239, 68, 68), "Нет связи")
                                };
                                ui.colored_label(dot, format!("● {}", label));

                                if let Some(w) = &self.current_wallet {
                                    ui.label(
                                        RichText::new(format!("👤 {}", w.name))
                                            .color(Color32::GRAY),
                                    );
                                }
                            },
                        );
                    }
                }
            });
        });
    }
}
