use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

#[derive(Debug, Clone)]
pub struct LoadedWallet {
    pub name: String,
    pub address: String,
    pub public_key: String,
    pub file_path: PathBuf,
    private_key: Option<Zeroizing<String>>,
}

impl LoadedWallet {
    pub fn private_key(&self) -> Option<&str> {
        self.private_key.as_deref().map(|s| s.as_str())
    }

    pub fn is_unlocked(&self) -> bool {
        self.private_key.is_some()
    }

    pub fn lock(&mut self) {
        self.private_key = None;
    }
}

pub fn list_keystores(wallets_dir: &Path) -> Vec<KeystoreEntry> {
    let Ok(entries) = std::fs::read_dir(wallets_dir) else {
        return vec![];
    };

    entries
        .flatten()
        .filter(|e| {
            e.path().extension().map(|ext| ext == "tvc").unwrap_or(false)
        })
        .filter_map(|e| {
            let path = e.path();
            let content = std::fs::read_to_string(&path).ok()?;
            let ks: serde_json::Value = serde_json::from_str(&content).ok()?;
            Some(KeystoreEntry {
                name: ks["name"].as_str().unwrap_or("Unnamed").to_string(),
                address: ks["address"].as_str().unwrap_or("").to_string(),
                file_path: path,
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeystoreEntry {
    pub name: String,
    pub address: String,
    pub file_path: PathBuf,
}

pub struct KeystoreManager {
    pub wallets_dir: PathBuf,
}

impl KeystoreManager {
    pub fn new(data_dir: &Path) -> Result<Self> {
        let wallets_dir = data_dir.join("wallets");
        std::fs::create_dir_all(&wallets_dir)
            .context("Failed to create wallets directory")?;
        Ok(KeystoreManager { wallets_dir })
    }

    pub fn create_wallet(&self, name: &str, passphrase: &str) -> Result<LoadedWallet> {
        use trevailo_core::wallet::{Keystore, Wallet};

        if name.is_empty() {
            bail!("Wallet name cannot be empty");
        }
        if passphrase.len() < 8 {
            bail!("Passphrase must be at least 8 characters");
        }

        let wallet = Wallet::generate();
        let address = wallet.address();
        let public_key = wallet.public_key.clone();
        let privkey = wallet.private_key_hex();

        let ks = Keystore::encrypt(&wallet, name, passphrase)?;
        let filename = sanitize_filename(name);
        let file_path = unique_path(self.wallets_dir.join(format!("{}.tvc", filename)));
        ks.save(file_path.to_str().unwrap())?;

        tracing::info!("💾 Wallet '{}' saved to {:?}", name, file_path);

        Ok(LoadedWallet {
            name: name.to_string(),
            address,
            public_key,
            file_path,
            private_key: Some(privkey),
        })
    }

    pub fn import_wallet(
        &self,
        name: &str,
        private_key_hex: &str,
        passphrase: &str,
    ) -> Result<LoadedWallet> {
        use trevailo_core::wallet::{Keystore, Wallet};

        let wallet = Wallet::from_private_key(private_key_hex)
            .context("Invalid private key")?;
        let address = wallet.address();
        let public_key = wallet.public_key.clone();
        let privkey = wallet.private_key_hex();

        let ks = Keystore::encrypt(&wallet, name, passphrase)?;
        let filename = sanitize_filename(name);
        let file_path = unique_path(self.wallets_dir.join(format!("{}.tvc", filename)));
        ks.save(file_path.to_str().unwrap())?;

        Ok(LoadedWallet {
            name: name.to_string(),
            address,
            public_key,
            file_path,
            private_key: Some(privkey),
        })
    }

    pub fn unlock_wallet(&self, entry: &KeystoreEntry, passphrase: &str) -> Result<LoadedWallet> {
        use trevailo_core::wallet::Keystore;

        let ks = Keystore::load(entry.file_path.to_str().unwrap())?;
        let wallet = ks.decrypt(passphrase)?;

        Ok(LoadedWallet {
            name: entry.name.clone(),
            address: wallet.address(),
            public_key: wallet.public_key.clone(),
            file_path: entry.file_path.clone(),
            private_key: Some(wallet.private_key_hex()),
        })
    }

    pub fn list(&self) -> Vec<KeystoreEntry> {
        list_keystores(&self.wallets_dir)
    }

    pub fn delete_wallet(&self, entry: &KeystoreEntry) -> Result<()> {
        std::fs::remove_file(&entry.file_path)
            .with_context(|| format!("Failed to delete {:?}", entry.file_path))?;
        tracing::info!("🗑️  Wallet '{}' deleted", entry.name);
        Ok(())
    }

    pub fn change_passphrase(
        &self,
        entry: &KeystoreEntry,
        old_passphrase: &str,
        new_passphrase: &str,
    ) -> Result<()> {
        use trevailo_core::wallet::Keystore;

        if new_passphrase.len() < 8 {
            bail!("New passphrase must be at least 8 characters");
        }
        let ks = Keystore::load(entry.file_path.to_str().unwrap())?;
        let wallet = ks.decrypt(old_passphrase)?;
        let new_ks = Keystore::encrypt(&wallet, &entry.name, new_passphrase)?;
        new_ks.save(entry.file_path.to_str().unwrap())?;
        tracing::info!("🔑 Passphrase changed for '{}'", entry.name);
        Ok(())
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

fn unique_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let ext = path.extension().unwrap_or_default().to_string_lossy().to_string();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    for i in 1..=99 {
        let new_path = parent.join(format!("{}-{}.{}", stem, i, ext));
        if !new_path.exists() {
            return new_path;
        }
    }
    path
}
