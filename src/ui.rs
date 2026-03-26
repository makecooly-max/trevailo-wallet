use egui::{Color32, RichText, Ui};
use crate::node_client::TxStatusKind;

// ─── Helpers ─────────────────────────────────────────────────────────────────

pub fn show_messages(app: &mut crate::app::TrevailoWallet, ui: &mut Ui) {
    if let Some(err) = app.error_message.clone() {
        egui::Frame::none()
            .fill(Color32::from_rgb(254, 226, 226))
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("⚠ {}", err)).color(Color32::from_rgb(185, 28, 28)));
                    if ui.small_button("✕").clicked() {
                        app.error_message = None;
                    }
                });
            });
        ui.add_space(4.0);
    }
    if let Some(ok) = app.success_message.clone() {
        egui::Frame::none()
            .fill(Color32::from_rgb(220, 252, 231))
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("✓ {}", ok)).color(Color32::from_rgb(22, 163, 74)));
                    if ui.small_button("✕").clicked() {
                        app.success_message = None;
                    }
                });
            });
        ui.add_space(4.0);
    }
}

pub fn address_field(ui: &mut Ui, label: &str, value: &str) {
    ui.label(RichText::new(label).small().color(Color32::GRAY));
    ui.horizontal(|ui| {
        let truncated = if value.len() > 40 {
            format!("{}…{}", &value[..20], &value[value.len() - 8..])
        } else {
            value.to_string()
        };
        ui.monospace(RichText::new(&truncated).color(Color32::from_rgb(99, 102, 241)));
        if ui.small_button("📋").on_hover_text("Скопировать").clicked() {
            ui.output_mut(|o| o.copied_text = value.to_string());
        }
    });
}

pub fn primary_button(ui: &mut Ui, label: &str) -> egui::Response {
    let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE))
        .fill(Color32::from_rgb(99, 102, 241))
        .rounding(8.0)
        .min_size(egui::Vec2::new(120.0, 36.0));
    ui.add(btn)
}

pub fn danger_button(ui: &mut Ui, label: &str) -> egui::Response {
    let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE))
        .fill(Color32::from_rgb(220, 38, 38))
        .rounding(8.0)
        .min_size(egui::Vec2::new(120.0, 36.0));
    ui.add(btn)
}

pub fn password_strength(pass: &str) -> u8 {
    if pass.len() < 8 { return 0; }
    let u = pass.chars().any(|c| c.is_uppercase()) as u8;
    let d = pass.chars().any(|c| c.is_numeric()) as u8;
    let s = pass.chars().any(|c| !c.is_alphanumeric()) as u8;
    let l = (pass.len() >= 12) as u8;
    (u + d + s + l).min(3)
}

pub fn show_password_strength(ui: &mut Ui, pass: &str) {
    let strength = password_strength(pass);
    if pass.is_empty() { return; }
    ui.horizontal(|ui| {
        ui.label(RichText::new("Сила пароля: ").size(11.0).color(Color32::GRAY));
        let (color, text) = match strength {
            1 => (Color32::from_rgb(239, 68, 68), "Слабый"),
            2 => (Color32::from_rgb(234, 179, 8), "Средний"),
            3 => (Color32::from_rgb(34, 197, 94), "Сильный"),
            _ => (Color32::from_rgb(239, 68, 68), "Слишком короткий"),
        };
        ui.colored_label(color, RichText::new(text).size(11.0));
    });
}

/// Блок с pending транзакциями
pub fn show_pending_txs(app: &mut crate::app::TrevailoWallet, ui: &mut Ui) {
    if app.pending_txs.is_empty() { return; }

    ui.separator();
    ui.label(RichText::new("История транзакций").color(Color32::GRAY).size(13.0));
    ui.add_space(4.0);

    let txs = app.pending_txs.clone();
    for tx in &txs {
        let (icon, color, status_text) = match &tx.status {
            TxStatusKind::Sending => ("⏳", Color32::from_rgb(234, 179, 8), "Отправляется...".to_string()),
            TxStatusKind::Pending => ("⏳", Color32::from_rgb(234, 179, 8), "В мемпуле, ожидает блока".to_string()),
            TxStatusKind::Confirmed { block_height, confirmations } => (
                "✅",
                Color32::from_rgb(22, 163, 74),
                format!("Подтверждено в блоке #{} ({} подтв.)", block_height, confirmations),
            ),
            TxStatusKind::Failed(msg) => ("❌", Color32::from_rgb(239, 68, 68), msg.clone()),
        };

        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(8.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).size(16.0));
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{:.6} TVC", tx.amount_tvc)).strong());
                            ui.label(RichText::new("→").color(Color32::GRAY));
                            let short_to = if tx.to.len() > 20 {
                                format!("{}…{}", &tx.to[..10], &tx.to[tx.to.len()-6..])
                            } else {
                                tx.to.clone()
                            };
                            ui.monospace(RichText::new(short_to).color(Color32::GRAY).size(11.0));
                        });
                        ui.label(RichText::new(&status_text).color(color).size(11.0));
                        let elapsed = tx.sent_at.elapsed().as_secs();
                        let time_str = if elapsed < 60 {
                            format!("{} сек назад", elapsed)
                        } else {
                            format!("{} мин назад", elapsed / 60)
                        };
                        ui.label(RichText::new(time_str).color(Color32::GRAY).size(10.0));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("📋").on_hover_text("Скопировать хеш TX").clicked() {
                            ui.output_mut(|o| o.copied_text = tx.hash.clone());
                        }
                        if tx.hash.len() >= 8 {
                            ui.monospace(
                                RichText::new(format!("{}…", &tx.hash[..8]))
                                    .size(10.0).color(Color32::GRAY)
                            );
                        }
                    });
                });
            });
        ui.add_space(4.0);
    }
}

// ─── Wallet Selector ─────────────────────────────────────────────────────────

pub mod wallet_selector {
    use super::*;
    use crate::app::{Screen, TrevailoWallet};

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui) {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("₮ Trevailo Wallet").size(28.0).color(Color32::from_rgb(99, 102, 241)));
            ui.add_space(4.0);
            ui.label(RichText::new("Выберите кошелёк или создайте новый").color(Color32::GRAY));
        });
        ui.add_space(16.0);
        super::show_messages(app, ui);

        let keystores = app.keystore_manager.as_ref().map(|km| km.list()).unwrap_or_default();

        if keystores.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(RichText::new("🗂 Нет сохранённых кошельков").color(Color32::GRAY).size(15.0));
                ui.add_space(8.0);
                ui.label(RichText::new("Создайте новый или импортируйте существующий").color(Color32::GRAY).size(12.0));
            });
        } else {
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for entry in keystores.clone() {
                    egui::Frame::none()
                        .fill(ui.visuals().faint_bg_color)
                        .rounding(10.0)
                        .inner_margin(egui::Margin::symmetric(16.0, 12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("💼").size(24.0));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&entry.name).size(15.0).strong());
                                    ui.monospace(
                                        RichText::new(
                                            if entry.address.len() > 28 {
                                                format!("{}…{}", &entry.address[..16], &entry.address[entry.address.len()-8..])
                                            } else {
                                                entry.address.clone()
                                            }
                                        ).color(Color32::GRAY).size(11.0)
                                    );
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if super::primary_button(ui, "🔓 Открыть").clicked() {
                                        app.screen = Screen::Unlock(entry.clone());
                                        app.clear_messages();
                                    }
                                });
                            });
                        });
                    ui.add_space(6.0);
                }
            });
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if super::primary_button(ui, "＋ Создать кошелёк").clicked() {
                app.screen = Screen::CreateWallet;
                app.form_wallet_name.clear();
                app.form_passphrase.clear();
                app.form_passphrase_confirm.clear();
                app.form_show_passphrase = false;
                app.clear_messages();
            }
            ui.add_space(8.0);
            if ui.button("📥 Импорт ключа").clicked() {
                app.screen = Screen::ImportWallet;
                app.form_wallet_name.clear();
                app.form_passphrase.clear();
                app.form_passphrase_confirm.clear();
                app.form_import_key.clear();
                app.form_show_passphrase = false;
                app.clear_messages();
            }
        });
    }
}

// ─── Unlock ───────────────────────────────────────────────────────────────────

pub mod unlock {
    use super::*;
    use crate::app::{Screen, TrevailoWallet};
    use crate::keystore_manager::KeystoreEntry;

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui, entry: KeystoreEntry) {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("🔒 Разблокировка кошелька").size(22.0));
            ui.add_space(4.0);
            ui.label(RichText::new(&entry.name).size(16.0).color(Color32::from_rgb(99, 102, 241)));
            ui.monospace(RichText::new(&entry.address).size(11.0).color(Color32::GRAY));
        });
        ui.add_space(16.0);
        show_messages(app, ui);

        ui.vertical_centered(|ui| {
            ui.set_max_width(380.0);
            ui.label("Пароль:");
            ui.add_space(4.0);
            let pass_resp = ui.add(
                egui::TextEdit::singleline(&mut app.form_passphrase)
                    .password(!app.form_show_passphrase)
                    .hint_text("Введите пароль кошелька")
                    .desired_width(340.0)
            );
            ui.checkbox(&mut app.form_show_passphrase, "Показать пароль");
            ui.add_space(12.0);

            let can_unlock = !app.form_passphrase.is_empty();
            let enter = pass_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if (super::primary_button(ui, "🔓 Разблокировать").clicked() || enter) && can_unlock {
                let passphrase = app.form_passphrase.clone();
                match app.keystore_manager.as_ref().unwrap().unlock_wallet(&entry, &passphrase) {
                    Ok(wallet) => {
                        app.form_passphrase.clear();
                        app.reset_auto_lock_timer();
                        app.switch_wallet(wallet);
                        app.screen = Screen::Dashboard;
                    }
                    Err(e) => app.set_error(format!("Неверный пароль: {}", e)),
                }
            }
            ui.add_space(8.0);
            if ui.button("← Назад").clicked() {
                app.form_passphrase.clear();
                app.screen = Screen::WalletSelector;
                app.clear_messages();
            }
        });
    }
}

// ─── Create Wallet ────────────────────────────────────────────────────────────

pub mod create_wallet {
    use super::*;
    use crate::app::{Screen, TrevailoWallet};

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui) {
        ui.add_space(20.0);
        ui.label(RichText::new("＋ Создать кошелёк").size(22.0));
        ui.add_space(8.0);
        show_messages(app, ui);

        ui.vertical_centered(|ui| {
            ui.set_max_width(400.0);

            ui.label("Название кошелька:");
            ui.add_space(2.0);
            ui.add(egui::TextEdit::singleline(&mut app.form_wallet_name)
                .hint_text("Например: Основной").desired_width(360.0));
            ui.add_space(10.0);

            ui.label("Пароль (мин. 8 символов):");
            ui.add_space(2.0);
            ui.add(egui::TextEdit::singleline(&mut app.form_passphrase)
                .password(!app.form_show_passphrase)
                .hint_text("Придумайте надёжный пароль")
                .desired_width(360.0));
            super::show_password_strength(ui, &app.form_passphrase);

            ui.add_space(6.0);
            ui.label("Подтвердите пароль:");
            ui.add_space(2.0);
            ui.add(egui::TextEdit::singleline(&mut app.form_passphrase_confirm)
                .password(!app.form_show_passphrase)
                .hint_text("Повторите пароль")
                .desired_width(360.0));

            if !app.form_passphrase_confirm.is_empty() && app.form_passphrase != app.form_passphrase_confirm {
                ui.colored_label(Color32::from_rgb(239, 68, 68),
                    RichText::new("✗ Пароли не совпадают").size(11.0));
            }

            ui.add_space(4.0);
            ui.checkbox(&mut app.form_show_passphrase, "Показать пароль");
            ui.add_space(12.0);

            egui::Frame::none()
                .fill(Color32::from_rgb(239, 246, 255))
                .rounding(6.0)
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(
                        "🔐 Ключи генерируются локально и шифруются вашим паролем.\n\
                         Никто кроме вас не имеет к ним доступа."
                    ).size(11.0).color(Color32::from_rgb(37, 99, 235)));
                });

            ui.add_space(12.0);

            let can = !app.form_wallet_name.is_empty()
                && app.form_passphrase.len() >= 8
                && app.form_passphrase == app.form_passphrase_confirm;

            ui.add_enabled_ui(can, |ui| {
                if super::primary_button(ui, "✓ Создать").clicked() {
                    let name = app.form_wallet_name.clone();
                    let pass = app.form_passphrase.clone();
                    match app.keystore_manager.as_ref().unwrap().create_wallet(&name, &pass) {
                        Ok(wallet) => {
                            let wallet_name = wallet.name.clone();
                            app.form_passphrase.clear();
                            app.form_passphrase_confirm.clear();
                            app.form_wallet_name.clear();
                            app.switch_wallet(wallet);
                            app.set_success(format!("Кошелёк '{}' успешно создан!", wallet_name));
                            app.screen = Screen::Dashboard;
                        }
                        Err(e) => app.set_error(e.to_string()),
                    }
                }
            });
            ui.add_space(8.0);
            if ui.button("← Назад").clicked() {
                app.screen = Screen::WalletSelector;
            }
        });
    }
}

// ─── Import Wallet ────────────────────────────────────────────────────────────

pub mod import_wallet {
    use super::*;
    use crate::app::{Screen, TrevailoWallet};

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui) {
        ui.add_space(20.0);
        ui.label(RichText::new("📥 Импорт кошелька").size(22.0));
        ui.add_space(8.0);
        show_messages(app, ui);

        ui.vertical_centered(|ui| {
            ui.set_max_width(420.0);

            ui.label("Название кошелька:");
            ui.add_space(2.0);
            ui.add(egui::TextEdit::singleline(&mut app.form_wallet_name)
                .hint_text("Например: Импортированный").desired_width(400.0));
            ui.add_space(8.0);

            ui.label("Приватный ключ (hex, 64 символа):");
            ui.add_space(2.0);
            ui.add(egui::TextEdit::singleline(&mut app.form_import_key)
                .password(!app.form_show_passphrase)
                .hint_text("64 hex-символа...")
                .desired_width(400.0));
            let key_len = app.form_import_key.len();
            if key_len > 0 && key_len != 64 {
                ui.colored_label(Color32::from_rgb(239, 68, 68),
                    RichText::new(format!("✗ Ключ должен быть 64 символа (сейчас {})", key_len)).size(11.0));
            }
            ui.add_space(8.0);

            ui.label("Пароль для шифрования:");
            ui.add_space(2.0);
            ui.add(egui::TextEdit::singleline(&mut app.form_passphrase)
                .password(!app.form_show_passphrase)
                .hint_text("Мин. 8 символов")
                .desired_width(400.0));
            super::show_password_strength(ui, &app.form_passphrase);

            ui.add_space(4.0);
            ui.label("Подтвердите пароль:");
            ui.add_space(2.0);
            ui.add(egui::TextEdit::singleline(&mut app.form_passphrase_confirm)
                .password(!app.form_show_passphrase)
                .desired_width(400.0));
            if !app.form_passphrase_confirm.is_empty() && app.form_passphrase != app.form_passphrase_confirm {
                ui.colored_label(Color32::from_rgb(239, 68, 68),
                    RichText::new("✗ Пароли не совпадают").size(11.0));
            }

            ui.add_space(4.0);
            ui.checkbox(&mut app.form_show_passphrase, "Показать поля");
            ui.add_space(8.0);

            egui::Frame::none()
                .fill(Color32::from_rgb(254, 243, 199))
                .rounding(6.0)
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("⚠ Приватный ключ шифруется локально и никуда не передаётся.")
                        .size(12.0).color(Color32::from_rgb(120, 53, 15)));
                });

            ui.add_space(12.0);
            let can = !app.form_wallet_name.is_empty()
                && app.form_import_key.len() == 64
                && app.form_passphrase.len() >= 8
                && app.form_passphrase == app.form_passphrase_confirm;

            ui.add_enabled_ui(can, |ui| {
                if super::primary_button(ui, "📥 Импортировать").clicked() {
                    let name = app.form_wallet_name.clone();
                    let key = app.form_import_key.clone();
                    let pass = app.form_passphrase.clone();
                    match app.keystore_manager.as_ref().unwrap().import_wallet(&name, &key, &pass) {
                        Ok(wallet) => {
                            app.form_import_key.clear();
                            app.form_passphrase.clear();
                            app.form_passphrase_confirm.clear();
                            app.form_wallet_name.clear();
                            app.switch_wallet(wallet);
                            app.screen = Screen::Dashboard;
                        }
                        Err(e) => app.set_error(e.to_string()),
                    }
                }
            });
            ui.add_space(8.0);
            if ui.button("← Назад").clicked() {
                app.screen = Screen::WalletSelector;
            }
        });
    }
}

// ─── Dashboard ────────────────────────────────────────────────────────────────

pub mod dashboard {
    use super::*;
    use crate::app::{Screen, TrevailoWallet};

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui) {
        show_messages(app, ui);

        let Some(wallet) = app.current_wallet.as_ref() else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Нет активного кошелька").color(Color32::GRAY));
            });
            return;
        };
        let address = wallet.address.clone();
        let name = wallet.name.clone();
        let balance_tvc = app.balance.as_ref().map(|b| b.balance_tvc).unwrap_or(0.0);
        let balance_trev = app.balance.as_ref().map(|b| b.balance_trev).unwrap_or(0);

        // Баланс-карточка
        egui::Frame::none()
            .fill(Color32::from_rgb(238, 242, 255))
            .rounding(14.0)
            .inner_margin(egui::Margin::symmetric(20.0, 16.0))
            .show(ui, |ui| {
                ui.label(RichText::new(&name).size(14.0).color(Color32::GRAY));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("{:.6} TVC", balance_tvc))
                        .size(32.0)
                        .color(Color32::from_rgb(79, 70, 229))
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{} trev", balance_trev))
                        .size(12.0)
                        .color(Color32::GRAY),
                );
                ui.add_space(8.0);
                super::address_field(ui, "Адрес", &address);
            });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if super::primary_button(ui, "↑ Отправить").clicked() {
                app.screen = Screen::Send;
            }
            ui.add_space(8.0);
            if ui.button("↓ Получить").on_hover_text("Показать адрес для приёма средств").clicked() {
                app.screen = Screen::Receive;
            }
            ui.add_space(8.0);
            if ui.button("🔄 Обновить").clicked() {
                app.last_refresh = std::time::Instant::now().checked_sub(std::time::Duration::from_secs(60)).unwrap_or_else(std::time::Instant::now);
            }
        });

        // Pending транзакции
        super::show_pending_txs(app, ui);

        // Инфо о сети
        if let Some(info) = &app.node_info.clone() {
            ui.add_space(8.0);
            ui.separator();
            ui.label(RichText::new("Состояние сети").color(Color32::GRAY).size(13.0));
            ui.add_space(4.0);
            egui::Grid::new("ninfo").num_columns(4).spacing([16.0, 4.0]).show(ui, |ui| {
                ui.label(RichText::new("Блок:").color(Color32::GRAY).size(11.0));
                ui.label(RichText::new(info.height.to_string()).strong());
                ui.label(RichText::new("Сложность:").color(Color32::GRAY).size(11.0));
                ui.label(RichText::new(info.difficulty.to_string()).strong());
                ui.end_row();
                ui.label(RichText::new("Мемпул:").color(Color32::GRAY).size(11.0));
                ui.label(RichText::new(format!("{} tx", info.mempool_size)).strong());
                ui.label(RichText::new("Награда:").color(Color32::GRAY).size(11.0));
                ui.label(RichText::new(format!("{:.3} TVC", info.block_reward_tvc)).strong());
                ui.end_row();
                if info.protected_period {
                    ui.label(RichText::new("Режим:").color(Color32::GRAY).size(11.0));
                    ui.label(
                        RichText::new(format!("🛡 Защита ({} блоков)", info.protected_blocks_remaining))
                            .color(Color32::from_rgb(217, 119, 6))
                            .strong()
                            .size(11.0),
                    );
                    ui.label(RichText::new("").size(11.0));
                    ui.label(RichText::new("").size(11.0));
                    ui.end_row();
                }
            });
        }

        // UTXO (короткий список)
        if !app.utxos.is_empty() {
            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("UTXOs ({})", app.utxos.len())).color(Color32::GRAY).size(13.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Все →").clicked() {
                        app.screen = Screen::Utxos;
                    }
                });
            });
            ui.add_space(4.0);
            for utxo in app.utxos.iter().take(5) {
                ui.horizontal(|ui| {
                    ui.monospace(RichText::new(
                        format!("{}…:{}", &utxo.tx_hash[..12.min(utxo.tx_hash.len())], utxo.output_index)
                    ).size(11.0).color(Color32::GRAY));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{:.6} TVC", utxo.amount as f64 / 1_000_000.0))
                            .color(Color32::from_rgb(22, 163, 74)).size(12.0));
                    });
                });
            }
            if app.utxos.len() > 5 {
                ui.label(RichText::new(format!("… ещё {} UTXO", app.utxos.len() - 5)).color(Color32::GRAY).size(11.0));
            }
        }
    }
}

// ─── Receive ─────────────────────────────────────────────────────────────────

pub mod receive {
    use super::*;
    use crate::app::TrevailoWallet;

    /// Генерирует пиксельную матрицу QR-кода для строки `data`.
    /// Возвращает (матрица строк по битам, размер стороны в модулях).
    fn make_qr(data: &str) -> Option<(Vec<Vec<bool>>, usize)> {
        use qrcode::{QrCode, EcLevel};
        let code = QrCode::with_error_correction_level(data, EcLevel::M).ok()?;
        let colors = code.to_colors();
        let side = (colors.len() as f64).sqrt() as usize;
        if side * side != colors.len() { return None; }
        let matrix: Vec<Vec<bool>> = (0..side)
            .map(|row| {
                (0..side)
                    .map(|col| colors[row * side + col] == qrcode::Color::Dark)
                    .collect()
            })
            .collect();
        Some((matrix, side))
    }

    /// Рисует QR-матрицу через egui Painter — чисто векторно, без текстур.
    fn draw_qr(ui: &mut Ui, matrix: &[Vec<bool>], side: usize, pixel_size: f32) {
        let total = pixel_size * side as f32;
        let padding = 12.0;

        // Белый фон с отступом (quiet zone)
        let (outer_rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(total + padding * 2.0, total + padding * 2.0),
            egui::Sense::hover(),
        );
        ui.painter().rect_filled(outer_rect, 8.0, Color32::WHITE);

        let origin = outer_rect.min + egui::Vec2::splat(padding);

        for (row, line) in matrix.iter().enumerate() {
            for (col, &dark) in line.iter().enumerate() {
                if dark {
                    let x = origin.x + col as f32 * pixel_size;
                    let y = origin.y + row as f32 * pixel_size;
                    let r = egui::Rect::from_min_size(
                        egui::Pos2::new(x, y),
                        egui::Vec2::splat(pixel_size),
                    );
                    ui.painter().rect_filled(r, 0.0, Color32::BLACK);
                }
            }
        }
    }

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui) {
        ui.label(RichText::new("↓ Получить TVC").size(22.0));
        ui.add_space(12.0);

        let Some(wallet) = app.current_wallet.as_ref() else { return; };
        let address = wallet.address.clone();

        ui.vertical_centered(|ui| {
            ui.set_max_width(520.0);

            egui::Frame::none()
                .fill(Color32::from_rgb(238, 242, 255))
                .rounding(14.0)
                .inner_margin(egui::Margin::symmetric(24.0, 20.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Ваш адрес для пополнения").color(Color32::GRAY).size(13.0));
                    ui.add_space(12.0);

                    // Генерируем и рисуем QR-код
                    match make_qr(&address) {
                        Some((matrix, side)) => {
                            // Подбираем размер пикселя под ~200px итогового QR
                            let pixel_size = (200.0 / side as f32).max(2.0).floor();
                            draw_qr(ui, &matrix, side, pixel_size);
                        }
                        None => {
                            // Фоллбэк если QR не сгенерировался
                            let (rect, _) = ui.allocate_exact_size(
                                egui::Vec2::splat(200.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 8.0, Color32::WHITE);
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "QR недоступен",
                                egui::FontId::proportional(14.0),
                                Color32::GRAY,
                            );
                        }
                    }

                    ui.add_space(12.0);
                    ui.label(RichText::new("Полный адрес:").size(11.0).color(Color32::GRAY));
                    ui.add_space(4.0);
                    // Адрес отображаем полностью — моноширинный шрифт, перенос по словам
                    ui.add(egui::Label::new(
                        RichText::new(&address)
                            .monospace()
                            .color(Color32::from_rgb(99, 102, 241))
                            .size(12.0)
                    ).wrap(true));
                    ui.add_space(8.0);
                    if super::primary_button(ui, "📋 Скопировать адрес").clicked() {
                        ui.output_mut(|o| o.copied_text = address.clone());
                        app.set_success("Адрес скопирован в буфер!");
                    }
                });

            ui.add_space(8.0);
            super::show_messages(app, ui);
            ui.add_space(4.0);

            egui::Frame::none()
                .fill(Color32::from_rgb(220, 252, 231))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(
                        "ℹ Отправьте этот адрес или QR-код отправителю.\n\
                         Средства появятся на балансе после подтверждения блоком."
                    ).size(12.0).color(Color32::from_rgb(22, 101, 52)));
                });
        });
    }
}

// ─── Send ─────────────────────────────────────────────────────────────────────

pub mod send {
    use super::*;
    use crate::app::TrevailoWallet;
    use crate::node_client::TxStatusKind;
    #[cfg(not(target_arch = "wasm32"))]
    use arboard::Clipboard;

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui) {
        ui.label(RichText::new("↑ Отправить TVC").size(22.0));
        ui.add_space(8.0);
        show_messages(app, ui);

        let wallet_locked = app.current_wallet.as_ref().map(|w| !w.is_unlocked()).unwrap_or(true);
        if wallet_locked {
            egui::Frame::none()
                .fill(Color32::from_rgb(254, 226, 226))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(16.0, 12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("🔒 Кошелёк заблокирован. Разблокируйте его для отправки.")
                        .color(Color32::from_rgb(185, 28, 28)));
                });
            return;
        }

        let balance_tvc = app.balance.as_ref().map(|b| b.balance_tvc).unwrap_or(0.0);

        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(10.0)
            .inner_margin(egui::Margin::symmetric(16.0, 12.0))
            .show(ui, |ui| {
                ui.label(RichText::new("Доступный баланс").color(Color32::GRAY).size(11.0));
                ui.label(RichText::new(format!("{:.6} TVC", balance_tvc))
                    .size(20.0).color(Color32::from_rgb(79, 70, 229)).strong());
            });

        ui.add_space(12.0);

        egui::Grid::new("send_form").num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
            ui.label("Адрес получателя:");
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut app.send_to)
                    .hint_text("Trevailo адрес...").desired_width(300.0));
                // Кнопка «Вставить» — читаем clipboard через egui (кроссплатформенно: Windows / Linux / macOS)
                if ui.small_button("📋 Вставить")
                    .on_hover_text("Вставить адрес из буфера обмена")
                    .clicked()
                {
                    // Ищем Event::Paste уже имеющийся в очереди (редко) ИЛИ
                    // используем arboard через платформенный бэкенд eframe
                    let from_events = ui.input(|i| {
                        i.events.iter().find_map(|e| {
                            if let egui::Event::Paste(t) = e { Some(t.clone()) } else { None }
                        })
                    });
                    if let Some(text) = from_events {
                        if !text.trim().is_empty() {
                            app.send_to = text.trim().to_string();
                        }
                    } else {
                        // Запрашиваем вставку: eframe 0.27 поддерживает clipboard через
                        // winit/arboard на всех платформах. Симулируем Ctrl+V через events.
                        // Самый надёжный способ — использовать ctx.send_viewport_cmd с фиктивным
                        // фокусом, но проще: просто помечаем что ждём paste на следующем кадре.
                        // eframe/egui сам прочитает clipboard когда TextEdit получит фокус + Ctrl+V.
                        // Для кнопки «Вставить» без фокуса используем arboard напрямую:
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Ok(mut clipboard) = Clipboard::new() {
                                if let Ok(text) = clipboard.get_text() {
                                    let text = text.trim().to_string();
                                    if !text.is_empty() {
                                        app.send_to = text;
                                    }
                                }
                            }
                        }
                    }
                }
            });
            ui.end_row();

            ui.label("Сумма (TVC):");
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut app.send_amount)
                    .hint_text("0.000000").desired_width(160.0));
                // Кнопка "Макс"
                if ui.small_button("Макс").on_hover_text("Отправить весь баланс за вычетом комиссии").clicked() {
                    let fee: f64 = app.send_fee.parse().unwrap_or(0.001);
                    let max = (balance_tvc - fee).max(0.0);
                    app.send_amount = format!("{:.6}", max);
                }
            });
            ui.end_row();

            ui.label("Комиссия (TVC):");
            ui.add(egui::TextEdit::singleline(&mut app.send_fee)
                .hint_text("0.001").desired_width(160.0));
            ui.end_row();
        });

        // Итоговая сумма списания
        let amount: f64 = app.send_amount.parse().unwrap_or(0.0);
        let fee: f64 = app.send_fee.parse().unwrap_or(0.001);
        if amount > 0.0 {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Итого спишется:").color(Color32::GRAY).size(11.0));
                ui.label(RichText::new(format!("{:.6} TVC", amount + fee))
                    .color(Color32::from_rgb(99, 102, 241)).size(11.0).strong());
            });
        }

        ui.add_space(4.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(239, 246, 255))
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.label(RichText::new(
                    "ℹ Транзакция попадёт в мемпул сразу. Подтверждение происходит\
                     когда майнер включает её в блок (обычно ~10-30 сек)."
                ).size(11.0).color(Color32::from_rgb(37, 99, 235)));
            });

        ui.add_space(12.0);

        let amount_ok = amount > 0.0;
        let fee_ok = fee >= 0.001;
        let addr_ok = !app.send_to.is_empty();
        let enough = amount + fee <= balance_tvc;

        // Блокируем кнопку если уже есть pending транзакция к тому же адресу
        let already_sending = app.pending_txs.iter().any(|tx|
            tx.to == app.send_to && matches!(tx.status, TxStatusKind::Pending | TxStatusKind::Sending)
        );

        // Показываем подсказки об ошибках
        if !fee_ok && !app.send_fee.is_empty() {
            ui.colored_label(Color32::GRAY,
                RichText::new("Мин. комиссия: 0.001 TVC").size(11.0));
        }
        if amount_ok && !enough {
            ui.colored_label(Color32::from_rgb(239, 68, 68),
                RichText::new("✗ Недостаточно средств").size(11.0));
        }
        if already_sending {
            ui.colored_label(Color32::from_rgb(234, 179, 8),
                "⚠ Уже есть pending транзакция на этот адрес");
        }

        let can_send = amount_ok && fee_ok && addr_ok && enough && !already_sending;
        ui.add_enabled_ui(can_send, |ui| {
            if super::primary_button(ui, "↑ Отправить").clicked() {
                let to = app.send_to.clone();
                let privkey = app.current_wallet.as_ref()
                    .and_then(|w| w.private_key()).unwrap_or("").to_string();

                match app.node_client.send_signed_tx(&privkey, &to, amount, fee) {
                    Ok(pending_tx) => {
                        let hash_short = if pending_tx.hash.len() >= 8 {
                            pending_tx.hash[..8].to_string()
                        } else {
                            pending_tx.hash.clone()
                        };
                        app.pending_txs.push(pending_tx);
                        app.set_success(format!("Отправлено! TX: {}… (ожидает блока)", hash_short));
                        app.send_to.clear();
                        app.send_amount.clear();
                        app.last_refresh = std::time::Instant::now().checked_sub(std::time::Duration::from_secs(60)).unwrap_or_else(std::time::Instant::now);
                    }
                    Err(e) => app.set_error(e.to_string()),
                }
            }
        });

        // История pending транзакций этого кошелька
        super::show_pending_txs(app, ui);
    }
}

// ─── UTXOs ────────────────────────────────────────────────────────────────────

pub mod utxos {
    use super::*;
    use crate::app::TrevailoWallet;

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui) {
        let total: u64 = app.utxos.iter().map(|u| u.amount).sum();
        let unspent: Vec<_> = app.utxos.iter().filter(|u| !u.spent).collect();
        let spent_count = app.utxos.len() - unspent.len();

        ui.horizontal(|ui| {
            ui.label(RichText::new("🪙 Мои UTXO").size(22.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Обновить").clicked() {
                    app.last_refresh = std::time::Instant::now().checked_sub(std::time::Duration::from_secs(60)).unwrap_or_else(std::time::Instant::now);
                }
            });
        });
        ui.add_space(8.0);

        // Сводка
        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(8.0)
            .inner_margin(egui::Margin::symmetric(16.0, 10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Всего UTXO").color(Color32::GRAY).size(11.0));
                        ui.label(RichText::new(app.utxos.len().to_string()).strong().size(18.0));
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Непотрачено").color(Color32::GRAY).size(11.0));
                        ui.label(RichText::new(unspent.len().to_string())
                            .color(Color32::from_rgb(22, 163, 74)).strong().size(18.0));
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Потрачено").color(Color32::GRAY).size(11.0));
                        ui.label(RichText::new(spent_count.to_string())
                            .color(Color32::GRAY).strong().size(18.0));
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Сумма (unspent)").color(Color32::GRAY).size(11.0));
                        ui.label(RichText::new(format!("{:.6} TVC", total as f64 / 1_000_000.0))
                            .color(Color32::from_rgb(79, 70, 229)).strong().size(14.0));
                    });
                });
            });

        ui.add_space(8.0);

        if app.utxos.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Нет UTXO").color(Color32::GRAY));
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for utxo in &app.utxos {
                let bg = if utxo.spent {
                    Color32::from_rgb(248, 248, 248)
                } else {
                    ui.visuals().faint_bg_color
                };
                egui::Frame::none()
                    .fill(bg)
                    .rounding(8.0)
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let status_icon = if utxo.spent { "🔴" } else { "🟢" };
                            ui.label(status_icon);
                            ui.vertical(|ui| {
                                let hash_display = if utxo.tx_hash.len() >= 20 {
                                    format!("{}…{}", &utxo.tx_hash[..12], &utxo.tx_hash[utxo.tx_hash.len()-6..])
                                } else {
                                    utxo.tx_hash.clone()
                                };
                                ui.monospace(RichText::new(format!("{}:{}", hash_display, utxo.output_index))
                                    .size(11.0).color(Color32::GRAY));
                                if utxo.spent {
                                    ui.label(RichText::new("Потрачен").color(Color32::GRAY).size(10.0));
                                }
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("📋").on_hover_text("Скопировать TX hash").clicked() {
                                    ui.output_mut(|o| o.copied_text = utxo.tx_hash.clone());
                                }
                                let color = if utxo.spent {
                                    Color32::GRAY
                                } else {
                                    Color32::from_rgb(22, 163, 74)
                                };
                                ui.label(RichText::new(format!("{:.6} TVC", utxo.amount as f64 / 1_000_000.0))
                                    .color(color).strong());
                            });
                        });
                    });
                ui.add_space(4.0);
            }
        });
    }
}

// ─── Settings ─────────────────────────────────────────────────────────────────

pub mod settings {
    use super::*;
    use crate::app::TrevailoWallet;

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui) {
        ui.label(RichText::new("⚙ Настройки").size(22.0));
        ui.add_space(8.0);
        show_messages(app, ui);

        // ─── Подключение к ноде ───────────────────────────────────────────────
        ui.label(RichText::new("🌐 Нода").strong());
        ui.add_space(4.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(239, 246, 255))
            .rounding(8.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("URL:").color(Color32::GRAY).size(12.0));
                    ui.monospace(
                        RichText::new(crate::app::NODE_URL)
                            .size(12.0)
                            .color(Color32::from_rgb(37, 99, 235))
                    );
                    if ui.small_button("📋").on_hover_text("Скопировать").clicked() {
                        ui.output_mut(|o| o.copied_text = crate::app::NODE_URL.to_string());
                    }
                });
                ui.label(
                    RichText::new("Адрес ноды фиксирован и не может быть изменён пользователем.")
                        .size(11.0).color(Color32::GRAY)
                );
            });

        ui.add_space(12.0);

        // ─── Платёжный сервер ────────────────────────────────────────────────
        ui.label(RichText::new("💳 Платёжный сервер (покупка TVC)").strong());
        ui.add_space(4.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(239, 246, 255))
            .rounding(8.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("URL:").color(Color32::GRAY).size(12.0));
                    ui.monospace(
                        RichText::new(crate::app::PAYMENT_URL)
                            .size(12.0)
                            .color(Color32::from_rgb(37, 99, 235))
                    );
                    if ui.small_button("📋").on_hover_text("Скопировать").clicked() {
                        ui.output_mut(|o| o.copied_text = crate::app::PAYMENT_URL.to_string());
                    }
                });
                ui.label(
                    RichText::new("Адрес платёжного сервера фиксирован и не может быть изменён пользователем.")
                        .size(11.0).color(Color32::GRAY)
                );
            });

        ui.add_space(12.0);
        ui.separator();

        // ─── Авто-блокировка ──────────────────────────────────────────────────
        ui.label(RichText::new("🔒 Безопасность").strong());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Авто-блокировка:");
            ui.add(egui::Slider::new(&mut app.auto_lock_secs, 60..=3600).suffix(" сек"));
        });
        let mins = app.auto_lock_secs / 60;
        ui.label(RichText::new(format!("Кошелёк заблокируется через {} мин бездействия", mins))
            .size(11.0).color(Color32::GRAY));

        // ─── Смена пароля ─────────────────────────────────────────────────────
        if app.current_wallet.is_some() {
            ui.add_space(12.0);
            ui.separator();
            ui.label(RichText::new("🔑 Сменить пароль").strong());
            ui.add_space(4.0);

            egui::Grid::new("change_pass").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label("Текущий пароль:");
                ui.add(egui::TextEdit::singleline(&mut app.form_old_passphrase)
                    .password(true).hint_text("Текущий пароль").desired_width(260.0));
                ui.end_row();
                ui.label("Новый пароль:");
                ui.add(egui::TextEdit::singleline(&mut app.form_new_passphrase)
                    .password(true).hint_text("Мин. 8 символов").desired_width(260.0));
                ui.end_row();
                ui.label("Подтвердите:");
                ui.add(egui::TextEdit::singleline(&mut app.form_new_passphrase_confirm)
                    .password(true).hint_text("Повторите новый пароль").desired_width(260.0));
                ui.end_row();
            });

            if !app.form_new_passphrase.is_empty() {
                super::show_password_strength(ui, &app.form_new_passphrase);
            }
            if !app.form_new_passphrase_confirm.is_empty()
                && app.form_new_passphrase != app.form_new_passphrase_confirm
            {
                ui.colored_label(Color32::from_rgb(239, 68, 68),
                    RichText::new("✗ Пароли не совпадают").size(11.0));
            }

            ui.add_space(4.0);
            let can_change = !app.form_old_passphrase.is_empty()
                && app.form_new_passphrase.len() >= 8
                && app.form_new_passphrase == app.form_new_passphrase_confirm;

            ui.add_enabled_ui(can_change, |ui| {
                if ui.button("Сменить пароль").clicked() {
                    if let Some(wallet) = &app.current_wallet {
                        let entry = crate::keystore_manager::KeystoreEntry {
                            name: wallet.name.clone(),
                            address: wallet.address.clone(),
                            file_path: wallet.file_path.clone(),
                        };
                        let old = app.form_old_passphrase.clone();
                        let new = app.form_new_passphrase.clone();
                        match app.keystore_manager.as_ref().unwrap()
                            .change_passphrase(&entry, &old, &new)
                        {
                            Ok(_) => {
                                app.form_old_passphrase.clear();
                                app.form_new_passphrase.clear();
                                app.form_new_passphrase_confirm.clear();
                                app.set_success("Пароль успешно изменён!");
                            }
                            Err(e) => app.set_error(format!("Ошибка: {}", e)),
                        }
                    }
                }
            });

            // ─── Информация о кошельке ────────────────────────────────────────
            ui.add_space(12.0);
            ui.separator();
            ui.label(RichText::new("💼 Текущий кошелёк").strong());
            ui.add_space(4.0);
            if let Some(wallet) = &app.current_wallet {
                let addr = wallet.address.clone();
                let pk = wallet.public_key.clone();
                let privkey_opt = wallet.private_key().map(|s| s.to_string());
                let file = wallet.file_path.to_string_lossy().to_string();
                super::address_field(ui, "Адрес", &addr);
                ui.add_space(2.0);
                super::address_field(ui, "Публичный ключ", &pk);
                ui.add_space(2.0);

                // ─── Приватный ключ ───────────────────────────────────────────
                ui.label(RichText::new("Приватный ключ:").small().color(Color32::GRAY));
                egui::Frame::none()
                    .fill(Color32::from_rgb(254, 243, 199))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("⚠ Никогда не передавайте приватный ключ третьим лицам!")
                                .size(11.0)
                                .color(Color32::from_rgb(146, 64, 14)),
                        );
                    });
                ui.add_space(2.0);
                if let Some(privkey) = &privkey_opt {
                    let privkey_clone = privkey.clone();
                    ui.horizontal(|ui| {
                        if app.show_private_key {
                            let short = if privkey_clone.len() > 20 {
                                format!("{}…{}", &privkey_clone[..8], &privkey_clone[privkey_clone.len()-8..])
                            } else {
                                privkey_clone.clone()
                            };
                            ui.monospace(RichText::new(&short).size(11.0).color(Color32::from_rgb(185, 28, 28)));
                        } else {
                            ui.monospace(RichText::new("••••••••••••••••••••••••••••••••").size(11.0).color(Color32::GRAY));
                        }
                        if ui.small_button(if app.show_private_key { "🙈 Скрыть" } else { "👁 Показать" }).clicked() {
                            app.show_private_key = !app.show_private_key;
                        }
                        if app.show_private_key {
                            if ui.small_button("📋 Копировать").on_hover_text("Скопировать приватный ключ").clicked() {
                                ui.output_mut(|o| o.copied_text = privkey_clone.clone());
                            }
                        }
                    });
                    if app.show_private_key {
                        ui.add_space(2.0);
                        // Показываем полный ключ в TextEdit (только для чтения) для удобного копирования
                        let mut full = privkey_clone.clone();
                        ui.add(
                            egui::TextEdit::singleline(&mut full)
                                .desired_width(380.0)
                                .font(egui::TextStyle::Monospace)
                        );
                    }
                } else {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(239, 246, 255))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("🔒 Кошелёк заблокирован — разблокируйте для просмотра ключа.")
                                    .size(11.0)
                                    .color(Color32::from_rgb(37, 99, 235)),
                            );
                        });
                }

                ui.add_space(2.0);
                ui.label(RichText::new("Файл кошелька:").small().color(Color32::GRAY));
                ui.horizontal(|ui| {
                    ui.monospace(RichText::new(&file).size(10.0).color(Color32::GRAY));
                    if ui.small_button("📋").on_hover_text("Скопировать путь").clicked() {
                        ui.output_mut(|o| o.copied_text = file.clone());
                    }
                });
            }

            // ─── Удаление кошелька ────────────────────────────────────────────
            ui.add_space(12.0);
            ui.separator();
            ui.label(RichText::new("⚠ Опасная зона").strong().color(Color32::from_rgb(185, 28, 28)));
            ui.add_space(4.0);

            if !app.confirm_delete {
                if super::danger_button(ui, "🗑 Удалить кошелёк").clicked() {
                    app.confirm_delete = true;
                }
                ui.label(RichText::new("Удаление необратимо. Убедитесь что у вас есть резервная копия ключа.")
                    .size(11.0).color(Color32::GRAY));
            } else {
                egui::Frame::none()
                    .fill(Color32::from_rgb(254, 226, 226))
                    .rounding(8.0)
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("⚠ Вы уверены? Файл кошелька будет удалён безвозвратно.")
                            .color(Color32::from_rgb(185, 28, 28)).strong());
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if super::danger_button(ui, "Да, удалить").clicked() {
                                if let Some(wallet) = &app.current_wallet {
                                    let entry = crate::keystore_manager::KeystoreEntry {
                                        name: wallet.name.clone(),
                                        address: wallet.address.clone(),
                                        file_path: wallet.file_path.clone(),
                                    };
                                    match app.keystore_manager.as_ref().unwrap().delete_wallet(&entry) {
                                        Ok(_) => {
                                            app.confirm_delete = false;
                                            app.logout();
                                        }
                                        Err(e) => app.set_error(e.to_string()),
                                    }
                                }
                            }
                            ui.add_space(8.0);
                            if ui.button("Отмена").clicked() {
                                app.confirm_delete = false;
                            }
                        });
                    });
            }
        }

        // ─── О программе ──────────────────────────────────────────────────────
        ui.add_space(12.0);
        ui.separator();
        ui.label(RichText::new("ℹ О программе").strong());
        ui.add_space(4.0);
        ui.label("Trevailo Wallet v0.1.0");
        ui.label(RichText::new("Ключи хранятся локально. Шифрование: AES-256-GCM + Argon2id.")
            .size(11.0).color(Color32::GRAY));
    }
} // end mod settings

// ═════════════════════════════════════════════════════════════════════════════
// Вкладка "Купить TVC"
// ═════════════════════════════════════════════════════════════════════════════

pub mod buy {
    use super::*;
    use crate::app::{BuyOrderStatus, Screen, TrevailoWallet};

    // ── HTTP helpers (блокирующие — вызываем из egui потока) ─────────────────

    #[derive(serde::Deserialize)]
    struct PriceResp {
        tvc_usd: f64,
        #[allow(dead_code)]
        fee_percent: f64,
        min_purchase_tvc: f64,
        max_purchase_tvc: f64,
    }

    #[derive(serde::Deserialize)]
    struct AvailabilityResp {
        available_tvc: f64,
        max_single_purchase_tvc: f64,
    }

    fn fetch_availability(server_url: &str) -> Option<AvailabilityResp> {
        let url = format!("{}/availability", server_url.trim_end_matches('/'));
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()?;
        client.get(&url).send().ok()?.json::<AvailabilityResp>().ok()
    }

    fn cancel_order_req(server_url: &str, order_id: &str) -> bool {
        let url = format!("{}/orders/{}/cancel", server_url.trim_end_matches('/'), order_id);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok();
        if let Some(c) = client {
            c.post(&url).send().map(|r| r.status().is_success()).unwrap_or(false)
        } else {
            false
        }
    }

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct CreateOrderResp {
        order_id: String,
        payment_url: String,
        tvc_to_deliver: f64,
        usd_amount: f64,
        fee_tvc: f64,
    }

    #[derive(serde::Deserialize)]
    struct OrderStatusResp {
        status: String,
        tvc_to_deliver: f64,
        blockchain_tx_hash: Option<String>,
    }

    fn fetch_price(server_url: &str) -> Option<PriceResp> {
        let url = format!("{}/price", server_url.trim_end_matches('/'));
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()?;
        client.get(&url).send().ok()?.json::<PriceResp>().ok()
    }

    fn create_order(
        server_url: &str,
        buyer_address: &str,
        tvc_amount: f64,
        provider: &str,
    ) -> Result<CreateOrderResp, String> {
        let url = format!("{}/orders", server_url.trim_end_matches('/'));
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| e.to_string())?;

        let body = serde_json::json!({
            "buyer_address": buyer_address,
            "tvc_amount": tvc_amount,
            "provider": provider,
        });

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("Сервер недоступен: {}", e))?;

        if !resp.status().is_success() {
            let text = resp.text().unwrap_or_default();
            // Пытаемся вытащить error из JSON
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(e) = v["error"].as_str() {
                    return Err(e.to_string());
                }
            }
            return Err(format!("Ошибка сервера: {}", text));
        }

        resp.json::<CreateOrderResp>().map_err(|e| format!("Ответ сервера: {}", e))
    }

    fn poll_order_status(server_url: &str, order_id: &str) -> Option<OrderStatusResp> {
        let url = format!("{}/orders/{}", server_url.trim_end_matches('/'), order_id);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()?;
        client.get(&url).send().ok()?.json::<OrderStatusResp>().ok()
    }

    fn open_browser(url: &str) {
        #[cfg(target_os = "windows")]
        { let _ = std::process::Command::new("cmd").args(["/c", "start", url]).spawn(); }
        #[cfg(target_os = "macos")]
        { let _ = std::process::Command::new("open").arg(url).spawn(); }
        #[cfg(target_os = "linux")]
        { let _ = std::process::Command::new("xdg-open").arg(url).spawn(); }
    }

    // ── Render ───────────────────────────────────────────────────────────────

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui) {
        ui.label(RichText::new("💳 Купить TVC").size(22.0));
        ui.add_space(8.0);

        // Адрес текущего кошелька (нужен для заказа)
        let buyer_address = match &app.current_wallet {
            Some(w) => w.address.clone(),
            None => {
                ui.colored_label(
                    Color32::from_rgb(239, 68, 68),
                    "⚠ Откройте кошелёк прежде чем покупать TVC",
                );
                return;
            }
        };

        // ── Обновляем цену раз в 60 сек ──────────────────────────────────────
        if app.buy_price_last_fetch.elapsed() > std::time::Duration::from_secs(60) {
            if let Some(p) = fetch_price(crate::app::PAYMENT_URL) {
                app.buy_price_usd = p.tvc_usd;
                app.buy_min_tvc = p.min_purchase_tvc;
                app.buy_max_tvc = p.max_purchase_tvc;
                app.buy_price_last_fetch = std::time::Instant::now();
            }
        }

        // ── Обновляем доступный баланс раз в 15 сек ──────────────────────────
        if app.buy_availability_last_fetch.elapsed() > std::time::Duration::from_secs(15) {
            if let Some(a) = fetch_availability(crate::app::PAYMENT_URL) {
                app.buy_available_tvc = a.available_tvc;
                // Обновляем максимум одного заказа
                if a.max_single_purchase_tvc > 0.0 {
                    app.buy_max_tvc = a.max_single_purchase_tvc;
                }
                app.buy_availability_last_fetch = std::time::Instant::now();
            }
        }

        // ── Polling статуса заказа ────────────────────────────────────────────
        let (poll_order_id, poll_attempts) = match &app.buy_status {
            BuyOrderStatus::WaitingPayment { order_id } => {
                if app.buy_poll_timer.elapsed() > std::time::Duration::from_secs(5) {
                    app.buy_poll_timer = std::time::Instant::now();
                    (Some(order_id.clone()), 0u32)
                } else {
                    (None, 0)
                }
            }
            BuyOrderStatus::Polling { order_id, attempts } => {
                if app.buy_poll_timer.elapsed() > std::time::Duration::from_secs(4) {
                    app.buy_poll_timer = std::time::Instant::now();
                    (Some(order_id.clone()), *attempts)
                } else {
                    (None, 0)
                }
            }
            _ => (None, 0),
        };

        if let Some(oid) = poll_order_id {
            let attempts = poll_attempts + 1;
            // Таймаут через ~5 минут (75 попыток × 4 сек)
            if attempts > 75 {
                app.buy_status = BuyOrderStatus::Error(
                    "Время ожидания оплаты истекло. Попробуйте снова.".to_string(),
                );
            } else if let Some(status_resp) = poll_order_status(crate::app::PAYMENT_URL, &oid) {
                match status_resp.status.as_str() {
                    "delivered" => {
                        let tx_hash = status_resp.blockchain_tx_hash.unwrap_or_default();
                        app.buy_status = BuyOrderStatus::Delivered {
                            order_id: oid.clone(),
                            tx_hash,
                            tvc_delivered: status_resp.tvc_to_deliver,
                        };
                        app.last_refresh =
                            std::time::Instant::now().checked_sub(std::time::Duration::from_secs(60)).unwrap_or_else(std::time::Instant::now);
                        // Обновляем доступность — TVC отправлены
                        app.buy_availability_last_fetch = std::time::Instant::now()
                            - std::time::Duration::from_secs(999);
                    }
                    "delivery_failed" => {
                        app.buy_status = BuyOrderStatus::Error(
                            "Деньги получены, но перевод TVC не удался. Свяжитесь с поддержкой."
                                .to_string(),
                        );
                        app.buy_availability_last_fetch = std::time::Instant::now()
                            - std::time::Duration::from_secs(999);
                    }
                    "cancelled" => {
                        app.buy_status = BuyOrderStatus::Error("Заказ отменён.".to_string());
                        // TVC вернулись в пул
                        app.buy_availability_last_fetch = std::time::Instant::now()
                            - std::time::Duration::from_secs(999);
                    }
                    _ => {
                        app.buy_status = BuyOrderStatus::Polling {
                            order_id: oid.clone(),
                            attempts,
                        };
                    }
                }
            } else {
                app.buy_status = BuyOrderStatus::Polling {
                    order_id: oid.clone(),
                    attempts,
                };
            }
            ui.ctx().request_repaint_after(std::time::Duration::from_secs(4));
        }

        // ═════════════════════════════════════════════════════════════════════
        // Отображение по статусу
        // ═════════════════════════════════════════════════════════════════════

        match app.buy_status.clone() {
            // ── Главная форма покупки ─────────────────────────────────────────
            BuyOrderStatus::Idle | BuyOrderStatus::Error(_) => {
                // Ошибка (если есть)
                if let BuyOrderStatus::Error(err_msg) = app.buy_status.clone() {
                    let clear = egui::Frame::none()
                        .fill(Color32::from_rgb(254, 226, 226))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("⚠ {}", err_msg))
                                        .color(Color32::from_rgb(185, 28, 28)),
                                );
                                ui.small_button("✕").clicked()
                            }).inner
                        }).inner;
                    if clear {
                        app.buy_status = BuyOrderStatus::Idle;
                    }
                    ui.add_space(8.0);
                }

                render_buy_form(app, ui, &buyer_address);
            }

            // ── Создаём заказ ────────────────────────────────────────────────
            BuyOrderStatus::CreatingOrder => {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Создаём заказ...")
                            .size(16.0)
                            .color(Color32::GRAY),
                    );
                });
            }

            // ── Ждём оплаты ──────────────────────────────────────────────────
            BuyOrderStatus::WaitingPayment { ref order_id }
            | BuyOrderStatus::Polling { ref order_id, .. } => {
                render_waiting_payment(app, ui, order_id);
            }

            // ── Успешно доставлено ────────────────────────────────────────────
            BuyOrderStatus::Delivered {
                ref order_id,
                ref tx_hash,
                tvc_delivered,
            } => {
                render_success(app, ui, order_id, tx_hash, tvc_delivered);
            }
        }
    }

    // ── Форма выбора суммы и провайдера ──────────────────────────────────────

    fn render_buy_form(app: &mut TrevailoWallet, ui: &mut Ui, buyer_address: &str) {
        // ─── Информационный блок с ценой ─────────────────────────────────────
        egui::Frame::none()
            .fill(Color32::from_rgb(238, 242, 255))
            .rounding(10.0)
            .inner_margin(egui::Margin::symmetric(16.0, 12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("₮  1 TVC = ")
                            .size(18.0)
                            .color(Color32::from_rgb(99, 102, 241)),
                    );
                    ui.label(
                        RichText::new(format!("${:.2}", app.buy_price_usd))
                            .size(22.0)
                            .strong()
                            .color(Color32::from_rgb(67, 56, 202)),
                    );
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("Pre-listing цена")
                            .size(11.0)
                            .color(Color32::GRAY),
                    );
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Комиссия сервиса: 5% · После листинга на бирже цена изменится")
                        .size(11.0)
                        .color(Color32::GRAY),
                );
            });

        ui.add_space(8.0);

        // ─── Доступный остаток ────────────────────────────────────────────────
        egui::Frame::none()
            .fill(if app.buy_available_tvc > 0.0 {
                Color32::from_rgb(240, 253, 244)
            } else {
                Color32::from_rgb(254, 242, 242)
            })
            .rounding(8.0)
            .inner_margin(egui::Margin::symmetric(14.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Доступно к покупке:").size(13.0).color(Color32::GRAY));
                    ui.add_space(6.0);
                    if app.buy_available_tvc > 0.0 {
                        ui.label(
                            RichText::new(format!("{:.2} TVC", app.buy_available_tvc))
                                .size(15.0)
                                .strong()
                                .color(Color32::from_rgb(22, 163, 74)),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("≈ ${:.2}", app.buy_available_tvc * app.buy_price_usd))
                                .size(12.0)
                                .color(Color32::GRAY),
                        );
                    } else {
                        ui.label(
                            RichText::new("Нет доступных TVC")
                                .size(13.0)
                                .color(Color32::from_rgb(239, 68, 68)),
                        );
                    }
                });
            });

        ui.add_space(12.0);

        // ─── Количество TVC ──────────────────────────────────────────────────
        ui.label(RichText::new("Количество TVC").strong());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let _response = ui.add(
                egui::TextEdit::singleline(&mut app.buy_tvc_amount)
                    .desired_width(160.0)
                    .hint_text("например: 100"),
            );
            ui.label(RichText::new("TVC").color(Color32::GRAY));

            // Быстрые кнопки
            for &amount in &[50.0_f64, 100.0, 500.0, 1000.0] {
                if ui
                    .small_button(RichText::new(format!("{}", amount as u32)))
                    .clicked()
                {
                    app.buy_tvc_amount = format!("{}", amount as u32);
                }
            }
        });

        // Итог
        let tvc_f: f64 = app.buy_tvc_amount.trim().parse().unwrap_or(0.0);
        let tvc_to_receive = tvc_f * 0.95; // 5% комиссия
        let usd_total = tvc_f * app.buy_price_usd;

        // Эффективный максимум — минимум из конфига и доступного баланса
        let _effective_max = if app.buy_available_tvc > 0.0 {
            app.buy_max_tvc.min(app.buy_available_tvc)
        } else {
            app.buy_max_tvc
        };

        if tvc_f > 0.0 {
            ui.add_space(6.0);
            egui::Frame::none()
                .fill(Color32::from_rgb(240, 253, 244))
                .rounding(6.0)
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("К оплате:");
                        ui.label(
                            RichText::new(format!("${:.2}", usd_total))
                                .strong()
                                .color(Color32::from_rgb(22, 163, 74)),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("→").color(Color32::GRAY),
                        );
                        ui.add_space(8.0);
                        ui.label("Получите:");
                        ui.label(
                            RichText::new(format!("{:.2} TVC", tvc_to_receive))
                                .strong()
                                .color(Color32::from_rgb(99, 102, 241)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!("(комиссия: {:.2} TVC)", tvc_f - tvc_to_receive))
                                .size(11.0)
                                .color(Color32::GRAY),
                        );
                    });
                });
        }

        ui.add_space(16.0);

        // ─── Способ оплаты ────────────────────────────────────────────────────
        ui.label(RichText::new("Способ оплаты").strong());
        ui.add_space(6.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(238, 242, 255))
            .stroke(egui::Stroke::new(2.0, Color32::from_rgb(99, 102, 241)))
            .rounding(8.0)
            .inner_margin(egui::Margin::symmetric(14.0, 10.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("₿  Крипта (NOWPayments)")
                        .color(Color32::from_rgb(67, 56, 202))
                        .strong(),
                );
                ui.label(
                    RichText::new("USDT, BTC, ETH и другие")
                        .size(10.0)
                        .color(Color32::GRAY),
                );
            });

        ui.add_space(16.0);

        // ─── Адрес получателя ─────────────────────────────────────────────────
        ui.label(RichText::new("TVC придут на адрес:").small().color(Color32::GRAY));
        ui.horizontal(|ui| {
            let addr_display = if buyer_address.len() > 44 {
                format!("{}…{}", &buyer_address[..20], &buyer_address[buyer_address.len()-8..])
            } else {
                buyer_address.to_string()
            };
            ui.monospace(
                RichText::new(&addr_display).color(Color32::from_rgb(99, 102, 241)),
            );
            if ui.small_button("📋").on_hover_text("Скопировать адрес").clicked() {
                ui.output_mut(|o| o.copied_text = buyer_address.to_string());
            }
        });

        ui.add_space(20.0);

        // ─── Кнопка "Купить" ──────────────────────────────────────────────────
        // Эффективный максимум — не больше доступного остатка
        let effective_max = if app.buy_available_tvc > 0.0 {
            app.buy_max_tvc.min(app.buy_available_tvc)
        } else {
            app.buy_max_tvc
        };

        let no_stock = app.buy_available_tvc < app.buy_min_tvc && app.buy_available_tvc > 0.0;
        let out_of_stock = app.buy_available_tvc == 0.0;
        let can_buy = tvc_f >= app.buy_min_tvc
            && tvc_f <= effective_max
            && !no_stock
            && !out_of_stock;

        ui.horizontal(|ui| {
            let btn_text = if out_of_stock || no_stock {
                "⛔  Нет в наличии"
            } else {
                "🛒  Перейти к оплате"
            };

            let btn = egui::Button::new(
                RichText::new(btn_text).size(15.0).color(Color32::WHITE),
            )
            .fill(if can_buy {
                Color32::from_rgb(99, 102, 241)
            } else {
                Color32::from_rgb(180, 180, 200)
            })
            .rounding(10.0)
            .min_size(egui::Vec2::new(200.0, 44.0));

            if ui.add_enabled(can_buy, btn).clicked() {
                let server = crate::app::PAYMENT_URL.to_string();
                let address = buyer_address.to_string();
                let provider = app.buy_provider.clone();
                match create_order(&server, &address, tvc_f, &provider) {
                    Ok(resp) => {
                        open_browser(&resp.payment_url);
                        app.buy_poll_timer = std::time::Instant::now();
                        app.buy_status = BuyOrderStatus::WaitingPayment {
                            order_id: resp.order_id.clone(),
                        };
                        // Сбрасываем кэш доступности — она изменилась
                        app.buy_availability_last_fetch = std::time::Instant::now()
                            - std::time::Duration::from_secs(999);
                    }
                    Err(e) => {
                        app.buy_status = BuyOrderStatus::Error(e);
                    }
                }
            }

            if !can_buy && tvc_f > 0.0 && !out_of_stock && !no_stock {
                let msg = if tvc_f < app.buy_min_tvc {
                    format!("Минимум {:.0} TVC", app.buy_min_tvc)
                } else {
                    format!("Максимум {:.0} TVC доступно", effective_max)
                };
                ui.label(
                    RichText::new(msg)
                        .color(Color32::from_rgb(239, 68, 68))
                        .size(12.0),
                );
            }
        });

        ui.add_space(8.0);
        ui.label(
            RichText::new(
                "После нажатия откроется страница оплаты в браузере. \
                 Кошелёк автоматически получит TVC после подтверждения платежа.",
            )
            .size(11.0)
            .color(Color32::GRAY),
        );
    }

    // ── Экран ожидания оплаты ────────────────────────────────────────────────

    fn render_waiting_payment(app: &mut TrevailoWallet, ui: &mut Ui, order_id: &str) {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.spinner();
            ui.add_space(12.0);
            ui.label(
                RichText::new("⏳ Ожидаем оплату...")
                    .size(18.0)
                    .color(Color32::from_rgb(99, 102, 241)),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "Страница оплаты открыта в браузере.\n\
                     После оплаты TVC автоматически появятся на вашем кошельке.",
                )
                .size(13.0)
                .color(Color32::GRAY),
            );
            ui.add_space(16.0);

            ui.label(
                RichText::new(format!("Заказ: {}", order_id))
                    .size(11.0)
                    .color(Color32::GRAY)
                    .monospace(),
            );

            ui.add_space(20.0);

            // Кнопка "Отменить" — освобождает резервирование на сервере
            let cancel_btn = egui::Button::new(
                RichText::new("✕ Отменить покупку").color(Color32::from_rgb(239, 68, 68)),
            )
            .fill(Color32::from_rgb(254, 242, 242))
            .rounding(8.0)
            .min_size(egui::Vec2::new(160.0, 32.0));

            if ui.add(cancel_btn).clicked() {
                // Отправляем отмену на сервер чтобы освободить резервирование
                let server = crate::app::PAYMENT_URL.to_string();
                let oid = order_id.to_string();
                cancel_order_req(&server, &oid);
                app.buy_status = BuyOrderStatus::Idle;
                // Сбрасываем кэш доступности — TVC вернулись в пул
                app.buy_availability_last_fetch = std::time::Instant::now()
                    - std::time::Duration::from_secs(999);
            }

            ui.add_space(8.0);
            ui.label(
                RichText::new("При отмене зарезервированные TVC вернутся в пул")
                    .size(10.0)
                    .color(Color32::GRAY),
            );
        });

        // Request repaint to keep polling
        ui.ctx().request_repaint_after(std::time::Duration::from_secs(4));
    }

    // ── Экран успешной покупки ────────────────────────────────────────────────

    fn render_success(
        app: &mut TrevailoWallet,
        ui: &mut Ui,
        order_id: &str,
        tx_hash: &str,
        tvc_delivered: f64,
    ) {
        ui.add_space(24.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("✅").size(52.0));
            ui.add_space(8.0);
            ui.label(
                RichText::new("Покупка успешно завершена!")
                    .size(20.0)
                    .strong()
                    .color(Color32::from_rgb(22, 163, 74)),
            );
            ui.add_space(12.0);

            egui::Frame::none()
                .fill(Color32::from_rgb(240, 253, 244))
                .rounding(10.0)
                .inner_margin(egui::Margin::symmetric(20.0, 14.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("{:.4} TVC", tvc_delivered))
                            .size(28.0)
                            .strong()
                            .color(Color32::from_rgb(99, 102, 241)),
                    );
                    ui.label(
                        RichText::new("зачислены на ваш кошелёк")
                            .size(13.0)
                            .color(Color32::GRAY),
                    );
                });

            ui.add_space(16.0);

            // TX hash
            if !tx_hash.is_empty() {
                ui.label(
                    RichText::new("Транзакция в блокчейне:").size(11.0).color(Color32::GRAY),
                );
                ui.horizontal(|ui| {
                    let short_hash = if tx_hash.len() > 40 {
                        format!("{}…{}", &tx_hash[..20], &tx_hash[tx_hash.len()-8..])
                    } else {
                        tx_hash.to_string()
                    };
                    ui.monospace(
                        RichText::new(&short_hash).color(Color32::from_rgb(99, 102, 241)),
                    );
                    if ui.small_button("📋").clicked() {
                        ui.output_mut(|o| o.copied_text = tx_hash.to_string());
                    }
                });
            }

            ui.add_space(4.0);
            ui.label(
                RichText::new(format!("Заказ: {}", order_id))
                    .size(10.0)
                    .color(Color32::GRAY)
                    .monospace(),
            );

            ui.add_space(24.0);

            ui.horizontal(|ui| {
                // Купить ещё
                let buy_more_btn = egui::Button::new(
                    RichText::new("🛒 Купить ещё").color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(99, 102, 241))
                .rounding(8.0)
                .min_size(egui::Vec2::new(130.0, 36.0));

                if ui.add(buy_more_btn).clicked() {
                    app.buy_status = BuyOrderStatus::Idle;
                    app.buy_tvc_amount = "100".to_string();
                }

                ui.add_space(8.0);

                // Перейти на главную
                let dash_btn = egui::Button::new(
                    RichText::new("📊 На главную").color(Color32::from_rgb(99, 102, 241)),
                )
                .fill(Color32::from_rgb(238, 242, 255))
                .rounding(8.0)
                .min_size(egui::Vec2::new(130.0, 36.0));

                if ui.add(dash_btn).clicked() {
                    app.buy_status = BuyOrderStatus::Idle;
                    app.screen = Screen::Dashboard;
                }
            });
        });
    }
} // end mod buy

// ─── Mining ─────────────────────────────────────────────────────────────────
pub mod mining {
    use super::*;
    use crate::app::{MiningOutcome, TrevailoWallet};

    /// Преобразует Result<MineResponse> от ноды в типизированный MiningOutcome.
    /// Распознаёт HTTP 429 (rate limit) и HTTP 409 (nonce not found) по тексту ошибки.
    fn resolve_mine_outcome(result: anyhow::Result<crate::node_client::MineResponse>) -> MiningOutcome {
        match result {
            Ok(resp) => MiningOutcome::Success(resp),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("429") {
                    // Парсим секунды из текста "...подождите ещё 42s..."
                    // ВАЖНО: ищем число строго после слова "ещё" или "wait",
                    // иначе find_map найдёт 429 из HTTP-кода раньше.
                    let wait = msg
                        .split_whitespace()
                        .skip_while(|w| *w != "ещё" && *w != "wait")
                        .nth(1)
                        .and_then(|w| w.trim_end_matches('s').parse::<u64>().ok())
                        .unwrap_or(62);
                    MiningOutcome::RateLimited(wait)
                } else if msg.contains("409") || msg.contains("nonce") {
                    MiningOutcome::NonceNotFound
                } else {
                    MiningOutcome::Error(msg)
                }
            }
        }
    }

    pub fn render(app: &mut TrevailoWallet, ui: &mut Ui) {
        show_messages(app, ui);

        // Баннер защищённого периода
        if let Some(info) = app.node_info.as_ref() {
            if info.protected_period {
                egui::Frame::none()
                    .fill(Color32::from_rgb(254, 243, 199))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "🛡  Защищённый период — осталось {} блоков",
                                info.protected_blocks_remaining
                            ))
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(120, 80, 0)),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(
                                "Один адрес — одна попытка в минуту.                                  Шансы одинаковы для всех, независимо от мощности компьютера.
                                 Автомайнинг соблюдает это автоматически."
                            )
                            .size(11.0)
                            .color(Color32::from_rgb(120, 80, 0)),
                        );
                    });
                ui.add_space(10.0);
            }
        }

        // Poll background mining result (manual or auto).
        if let Some(rx) = &app.mining_task_rx {
            use std::sync::mpsc::TryRecvError;
            match rx.try_recv() {
                Ok(outcome) => {
                    match &outcome {
                        MiningOutcome::Success(resp) => {
                            app.last_mining = Some(resp.clone());
                            let badge = if resp.protected_period { " 🛡" } else { "" };
                            app.set_success(format!(
                                "⛏️ Block #{} mined | reward={:.3} TVC{}",
                                resp.height, resp.reward_tvc, badge
                            ));
                        }
                        MiningOutcome::RateLimited(wait) => {
                            app.set_error(format!(
                                "⏳ Защищённый период: подождите ещё {}с перед следующей попыткой.",
                                wait
                            ));
                        }
                        MiningOutcome::NonceNotFound => {
                            // Не показываем как ошибку — это нормально в защищённый период.
                            app.set_success(
                                "🎲 Нужный nonce не попался — попробуйте ещё раз!".to_string()
                            );
                        }
                        MiningOutcome::Error(e) => {
                            app.set_error(format!("Ошибка майнинга: {}", e));
                        }
                    }

                    // Для ручного майнинга завершаем задачу, для автo оставляем.
                    if !app.mining_auto_enabled {
                        app.mining_task_rx = None;
                        app.mining_in_progress = false;
                        app.mining_auto_stop = None;
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    // Поток завершился (например, после stop).
                    app.mining_task_rx = None;
                    app.mining_in_progress = false;
                    app.mining_auto_enabled = false;
                    app.mining_auto_stop = None;
                }
            }
        }

        ui.label(RichText::new("⛏️ Майнинг Trevailo Coin").size(22.0));
        ui.add_space(10.0);

        let address = match app.current_wallet.as_ref() {
            Some(w) => w.address.clone(),
            None => {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("Нет активного кошелька").color(Color32::GRAY));
                });
                return;
            }
        };

        let unlocked = app
            .current_wallet
            .as_ref()
            .and_then(|w| w.private_key())
            .is_some();

        egui::Frame::none()
            .fill(Color32::from_rgb(238, 242, 255))
            .rounding(14.0)
            .inner_margin(egui::Margin::symmetric(24.0, 20.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Кошелёк и награда")
                        .color(Color32::GRAY)
                        .size(13.0),
                );
                ui.add_space(6.0);
                super::address_field(ui, "Адрес майнера", &address);
                ui.add_space(10.0);

                if !unlocked {
                    ui.label(
                        RichText::new("Разблокируйте кошелёк, чтобы майнить.")
                            .size(12.0)
                            .color(Color32::from_rgb(185, 28, 28)),
                    );
                }

                // Последний результат
                if let Some(last) = &app.last_mining {
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Последний майнинг")
                            .size(13.0)
                            .color(Color32::GRAY),
                    );
                    ui.add_space(6.0);
                    egui::Grid::new("mining_last").num_columns(2).spacing([16.0, 4.0]).show(ui, |ui| {
                        ui.label(RichText::new("Блок:").color(Color32::GRAY).size(11.0));
                        ui.label(RichText::new(last.height.to_string()).color(Color32::from_rgb(30, 30, 30)).strong().size(12.0));
                        ui.end_row();
                        ui.label(RichText::new("Награда:").color(Color32::GRAY).size(11.0));
                        ui.label(RichText::new(format!("{:.3} TVC", last.reward_tvc)).color(Color32::from_rgb(30, 30, 30)).strong().size(12.0));
                        ui.end_row();
                        ui.label(RichText::new("Nonce:").color(Color32::GRAY).size(11.0));
                        ui.label(RichText::new(last.nonce.to_string()).color(Color32::from_rgb(30, 30, 30)).strong().size(12.0));
                        ui.end_row();
                        ui.label(RichText::new("Режим:").color(Color32::GRAY).size(11.0));
                        if last.protected_period {
                            ui.label(RichText::new("🛡 Защищённый").color(Color32::from_rgb(217, 119, 6)).strong().size(11.0));
                        } else {
                            ui.label(RichText::new("⚔️ Свободная конкуренция").color(Color32::from_rgb(30, 30, 30)).size(11.0));
                        }
                        ui.end_row();
                    });
                }

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(14.0);

                // Ручной майнинг
                ui.horizontal(|ui| {
                    let can_manual = unlocked
                        && !app.mining_in_progress
                        && !app.mining_auto_enabled
                        && app.mining_task_rx.is_none();
                    ui.add_enabled_ui(can_manual, |ui| {
                        if super::primary_button(ui, "⛏️ Майнить сейчас").clicked() {
                            let priv_key = app
                                .current_wallet
                                .as_ref()
                                .and_then(|w| w.private_key())
                                .expect("unlocked")
                                .to_string();
                            let node_client = app.node_client.clone();
                            let (tx, rx) = std::sync::mpsc::channel::<MiningOutcome>();
                            app.mining_task_rx = Some(rx);
                            app.mining_in_progress = true;
                            app.mining_auto_enabled = false;
                            app.mining_auto_stop = None;

                            std::thread::spawn(move || {
                                let outcome = resolve_mine_outcome(
                                    node_client.mine_with_private_key(&priv_key)
                                );
                                let _ = tx.send(outcome);
                            });
                        }
                    });

                    if app.mining_in_progress && !app.mining_auto_enabled {
                        ui.label(
                            RichText::new("Майнинг выполняется…")
                                .color(Color32::from_rgb(234, 179, 8))
                                .size(12.0),
                        );
                    }
                });

                ui.add_space(10.0);

                // Автомайнинг
                ui.label(
                    RichText::new("Автомайнинг")
                        .color(Color32::GRAY)
                        .size(13.0),
                );
                ui.add_space(6.0);

                let auto_toggle = ui.checkbox(&mut app.mining_auto_enabled, "Включить автo");
                let interval_disabled = app.mining_task_rx.is_some() && !app.mining_auto_enabled;
                ui.add_enabled_ui(unlocked && !interval_disabled, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Интервал (сек):").color(Color32::GRAY).size(12.0));
                        ui.add(egui::DragValue::new(&mut app.mining_auto_interval_secs).clamp_range(5..=3600));
                    });
                });

                if auto_toggle.clicked() {
                    if app.mining_auto_enabled {
                        // Запуск.
                        let start_ok = unlocked
                            && app.mining_task_rx.is_none()
                            && app.mining_auto_stop.is_none();
                        if !start_ok {
                            app.mining_auto_enabled = false;
                            app.set_error("Автомайнинг уже запущен или кошелёк не готов.".to_string());
                        } else {
                            let priv_key = app
                                .current_wallet
                                .as_ref()
                                .and_then(|w| w.private_key())
                                .expect("unlocked")
                                .to_string();
                            let node_client = app.node_client.clone();
                            let interval_secs = app.mining_auto_interval_secs.max(5);

                            let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                            app.mining_auto_stop = Some(stop_flag.clone());

                            let (tx, rx) = std::sync::mpsc::channel::<MiningOutcome>();
                            app.mining_task_rx = Some(rx);
                            app.mining_in_progress = true;

                            std::thread::spawn(move || {
                                while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                                    let outcome = resolve_mine_outcome(
                                        node_client.mine_with_private_key(&priv_key)
                                    );
                                    // При rate-limit нода говорит сколько ждать —
                                    // уважаем это значение вместо пользовательского интервала.
                                    let sleep_secs = match &outcome {
                                        MiningOutcome::RateLimited(wait) => *wait + 2,
                                        MiningOutcome::NonceNotFound => 1, // можно сразу
                                        _ => interval_secs,
                                    };
                                    if tx.send(outcome).is_err() {
                                        break; // UI больше не слушает.
                                    }
                                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                                        break;
                                    }
                                    std::thread::sleep(std::time::Duration::from_secs(sleep_secs));
                                }
                            });
                        }
                    } else {
                        // Остановка.
                        if let Some(stop) = &app.mining_auto_stop {
                            stop.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        app.mining_auto_stop = None;
                        // Поток может ещё отправить один результат — это нормально.
                        app.mining_in_progress = false;
                    }
                }

                if app.mining_auto_enabled {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Автомайнинг активен")
                            .color(Color32::from_rgb(34, 197, 94))
                            .size(12.0),
                    );
                }
            });
    }
}
