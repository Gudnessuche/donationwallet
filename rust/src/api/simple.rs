use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use bip39::rand::RngCore;
use bitcoin::{
    consensus::encode::serialize_hex,
    secp256k1::{PublicKey, SecretKey},
    OutPoint, Txid,
};

use crate::frb_generated::StreamSink;
use log::{debug, info};

use crate::{
    blindbit,
    logger::{self, LogEntry, LogLevel},
    stream::{self, ScanProgress, SyncStatus},
};

use anyhow::{anyhow, Result, Error};

use sp_client::{
    db::{JsonFile, Storage},
    spclient::{derive_keys_from_seed, Psbt, SpClient, SpWallet, SpendKey},
};

const PASSPHRASE: &str = ""; // no passphrase for now

type SpendingTxId = String;
type MinedInBlock = String;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum OutputSpendStatus {
    Unspent,
    Spent(SpendingTxId),
    Mined(MinedInBlock),
}

impl From<sp_client::spclient::OutputSpendStatus> for OutputSpendStatus {
    fn from(value: sp_client::spclient::OutputSpendStatus) -> Self {
        match value {
            sp_client::spclient::OutputSpendStatus::Unspent => OutputSpendStatus::Unspent,
            sp_client::spclient::OutputSpendStatus::Spent(txid) => OutputSpendStatus::Spent(txid),
            sp_client::spclient::OutputSpendStatus::Mined(block) => OutputSpendStatus::Mined(block),
        }
    }
}

impl From<OutputSpendStatus> for sp_client::spclient::OutputSpendStatus {
    fn from(value: OutputSpendStatus) -> Self {
        match value {
            OutputSpendStatus::Unspent => sp_client::spclient::OutputSpendStatus::Unspent,
            OutputSpendStatus::Spent(txid) => sp_client::spclient::OutputSpendStatus::Spent(txid),
            OutputSpendStatus::Mined(block) => sp_client::spclient::OutputSpendStatus::Mined(block),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Amount(pub u64);

impl From<bitcoin::Amount> for Amount {
    fn from(value: bitcoin::Amount) -> Self {
        Amount(value.to_sat())
    }
}

impl From<Amount> for bitcoin::Amount {
    fn from(value: Amount) -> bitcoin::Amount {
        bitcoin::Amount::from_sat(value.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OwnedOutput {
    pub blockheight: u32,
    pub tweak: String,
    pub amount: Amount,
    pub script: String,
    pub label: Option<String>,
    pub spend_status: OutputSpendStatus,
}

impl From<sp_client::spclient::OwnedOutput> for OwnedOutput {
    fn from(value: sp_client::spclient::OwnedOutput) -> Self {
        OwnedOutput {
            blockheight: value.blockheight,
            tweak: value.tweak,
            amount: value.amount.into(),
            script: value.script,
            label: value.label,
            spend_status: value.spend_status.into(),
        }
    }
}

impl From<OwnedOutput> for sp_client::spclient::OwnedOutput {
    fn from(value: OwnedOutput) -> Self {
        sp_client::spclient::OwnedOutput {
            blockheight: value.blockheight,
            tweak: value.tweak,
            amount: value.amount.into(),
            script: value.script,
            label: value.label,
            spend_status: value.spend_status.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Recipient {
    pub address: String, // either old school or silent payment
    pub amount: Amount,
    pub nb_outputs: u32, // if address is not SP, only 1 is valid
}

impl From<sp_client::spclient::Recipient> for Recipient {
    fn from(value: sp_client::spclient::Recipient) -> Self {
        Recipient {
            address: value.address,
            amount: value.amount.into(),
            nb_outputs: value.nb_outputs,
        }
    }
}

impl From<Recipient> for sp_client::spclient::Recipient {
    fn from(value: Recipient) -> Self {
        sp_client::spclient::Recipient {
            address: value.address,
            amount: value.amount.into(),
            nb_outputs: value.nb_outputs,
        }
    }
}
pub struct WalletStatus {
    pub amount: u64,
    pub birthday: u32,
    pub scan_height: u32,
}

pub fn create_log_stream(s: StreamSink<LogEntry>, level: LogLevel, log_dependencies: bool) {
    logger::init_logger(level.into(), log_dependencies);
    logger::FlutterLogger::set_stream_sink(s);
}
pub fn create_sync_stream(s: StreamSink<SyncStatus>) {
    stream::create_sync_stream(s);
}
pub fn create_scan_progress_stream(s: StreamSink<ScanProgress>) {
    stream::create_scan_progress_stream(s);
}
pub fn create_amount_stream(s: StreamSink<u64>) {
    stream::create_amount_stream(s);
}

#[flutter_rust_bridge::frb(sync)] 
pub fn wallet_exists(label: String, files_dir: String) -> bool {
    let storage = JsonFile::new(&files_dir, &label);
    if let Ok(_) = <JsonFile as Storage<SpWallet>>::load(&storage) {
        true
    } else {
        false
    }
}

pub enum WalletType {
    New,
    Mnemonic(String),
    PrivateKeys(String, String),
    WatchOnly(String, String)
}

pub async fn setup(
    label: String,
    files_dir: String,
    wallet_type: WalletType,
    birthday: u32,
    is_testnet: bool,
) -> Result<()> {
    if wallet_exists(label.clone(), files_dir.clone()) {
        return Err(anyhow!(label));
    }; // If the wallet already exists we just send the label as an error message

    // We create the file on disk
    let storage = JsonFile::new(&files_dir, &label);
    <JsonFile as Storage<SpWallet>>::create(&storage)?;

    let sp_client: SpClient;

    match wallet_type {
        WalletType::New => {
            // We create a new wallet and return the new mnemonic
            let m = bip39::Mnemonic::generate(12).unwrap();
            let seed = m.to_seed(PASSPHRASE);
            let (scan_sk, spend_sk) =
                derive_keys_from_seed(&seed, is_testnet)?;
            sp_client = SpClient::new(
                label,
                scan_sk,
                SpendKey::Secret(spend_sk),
                Some(m.to_string()),
                is_testnet,
            )?;
        }
        WalletType::Mnemonic(mnemonic) => {
            // We restore from seed
            let m = bip39::Mnemonic::from_str(&mnemonic)?;
            let seed = m.to_seed(PASSPHRASE);
            let (scan_sk, spend_sk) =
                derive_keys_from_seed(&seed, is_testnet)?;
            sp_client = SpClient::new(
                label,
                scan_sk,
                SpendKey::Secret(spend_sk),
                Some(mnemonic),
                is_testnet,
            )?;
        }
        WalletType::PrivateKeys(scan_sk_hex, spend_sk_hex) => {
            // We directly restore with the keys
            let scan_sk = SecretKey::from_str(&scan_sk_hex)?;
            let spend_sk = SecretKey::from_str(&spend_sk_hex)?;
            sp_client = SpClient::new(
                label,
                scan_sk,
                SpendKey::Secret(spend_sk),
                None,
                true
            )?;
        },
        WalletType::WatchOnly(scan_sk_hex, spend_pk_hex) => {
            // We directly restore with the keys
            let scan_sk = SecretKey::from_str(&scan_sk_hex)?;
            let spend_pk = PublicKey::from_str(&spend_pk_hex)?;
            sp_client = SpClient::new(
                label,
                scan_sk,
                SpendKey::Public(spend_pk),
                None,
                true
            )?;
        },
    }

    let mut sp_wallet = SpWallet::new(sp_client, None).unwrap();

    // Set the birthday and last_scan to prevent unnecessary scanning
    let outputs = sp_wallet.get_mut_outputs();
    outputs.set_birthday(birthday);
    outputs.update_last_scan(birthday);

    storage.save(&sp_wallet)?;

    Ok(())
}

/// Change wallet birthday
/// Reset the output list and last_scan
// #[flutter_rust_bridge::frb(sync)] 
pub async fn change_birthday(path: String, label: String, birthday: u32) -> Result<()> {
    let storage = JsonFile::new(&path, &label);
    debug!("{:?}", storage);
    match <JsonFile as Storage<SpWallet>>::load(&storage) {
        Ok(mut wallet) => {
            let outputs = wallet.get_mut_outputs();
            outputs.set_birthday(birthday);
            <JsonFile as Storage<SpWallet>>::save(&storage, &wallet)?;
            Ok(())
        },
        Err(e) => Err(Error::msg(format!("Failed to get the wallet on disk: {}", e)))
    }
}

/// Reset the last_scan of the wallet to its birthday, removing all outpoints
// #[flutter_rust_bridge::frb(sync)] 
pub async fn reset_wallet(path: String, label: String) -> Result<()> {
    let storage = JsonFile::new(&path, &label);
    if let Ok(mut wallet) = <JsonFile as Storage<SpWallet>>::load(&storage) {
        let outputs = wallet.get_mut_outputs();
        outputs.reset_to_birthday();
        <JsonFile as Storage<SpWallet>>::save(&storage, &wallet)?;
        Ok(())
    } else {
        Err(Error::msg("Failed to get the wallet on disk"))
    }
}

#[flutter_rust_bridge::frb(sync)] 
pub fn remove_wallet(path: String, label: String) -> Result<()> {
    let storage = JsonFile::new(&path, &label);
    <JsonFile as Storage<SpWallet>>::rm(storage).map(|_| ())
}

pub async fn sync_blockchain() -> Result<()> {
    blindbit::logic::sync_blockchain().await
}

pub async fn scan_to_tip(path: String, label: String) -> Result<()> {
    let storage = JsonFile::new(&path, &label);
    let mut wallet = <JsonFile as Storage<SpWallet>>::load(&storage)?;
    blindbit::logic::scan_blocks(0, &mut wallet).await
}

#[flutter_rust_bridge::frb(sync)] 
pub fn get_wallet_info(path: String, label: String) -> Result<WalletStatus> {
    let storage = JsonFile::new(&path, &label);
    if let Ok(wallet) = <JsonFile as Storage<SpWallet>>::load(&storage) {
        Ok(WalletStatus {
            amount: wallet.get_outputs().get_balance().to_sat(),
            birthday: wallet.get_outputs().get_birthday(),
            scan_height: wallet.get_outputs().get_last_scan(),
        })
    } else {
        Err(Error::msg("Failed to get the wallet on disk"))
    }
}

#[flutter_rust_bridge::frb(sync)] 
pub fn get_receiving_address(path: String, label: String) -> Result<String> {
    let storage = JsonFile::new(&path, &label);
    if let Ok(wallet) = <JsonFile as Storage<SpWallet>>::load(&storage) {
        Ok(wallet.get_client().get_receiving_address())
    } else {
        Err(Error::msg("Failed to get the wallet on disk"))
    }
}

#[flutter_rust_bridge::frb(sync)] 
pub fn get_spendable_outputs(
    path: String,
    label: String,
) -> Result<Vec<(String, OwnedOutput)>> {
    let storage = JsonFile::new(&path, &label);
    if let Ok(wallet) = <JsonFile as Storage<SpWallet>>::load(&storage) {
        Ok(wallet
            .get_outputs()
            .to_spendable_list()
            .into_iter()
            .map(|(outpoint, output)| (outpoint.to_string(), output.into()))
            .collect())
    } else {
        Err(Error::msg("Failed to get the wallet on disk"))
    }
}

#[flutter_rust_bridge::frb(sync)] 
pub fn get_outputs(path: String, label: String) -> Result<HashMap<String, OwnedOutput>> {
    let storage = JsonFile::new(&path, &label);
    if let Ok(wallet) = <JsonFile as Storage<SpWallet>>::load(&storage) {
        Ok(wallet
            .get_outputs()
            .to_outpoints_list()
            .into_iter()
            .map(|(outpoint, output)| (outpoint.to_string(), output.into()))
            .collect())
    } else {
        Err(Error::msg("Failed to get the wallet on disk"))
    }
}

#[flutter_rust_bridge::frb(sync)] 
pub fn create_new_psbt(
    label: String,
    path: String,
    inputs: HashMap<String, OwnedOutput>,
    recipients: Vec<Recipient>,
) -> Result<String> {
    // convert to spclient inputs
    let inputs = inputs
        .into_iter()
        .map(|(outpoint, output)| (OutPoint::from_str(&outpoint).unwrap(), output.into()))
        .collect();
    let recipients = recipients.into_iter().map(Into::into).collect();

    let storage = JsonFile::new(&path, &label);
    let wallet = <JsonFile as Storage<SpWallet>>::load(&storage)?;
    let psbt = wallet
        .get_client()
        .create_new_psbt(inputs, recipients, None)?;

    Ok(psbt.to_string())
}

// payer is an address, either Silent Payment or not
pub fn add_fee_for_fee_rate(psbt: String, fee_rate: u32, payer: String) -> Result<String> {
    let mut psbt = Psbt::from_str(&psbt)?;

    SpClient::set_fees(&mut psbt, Amount(fee_rate.into()).into(), payer)?;

    Ok(psbt.to_string())
}

pub fn fill_sp_outputs(path: String, label: String, psbt: String) -> Result<String> {
    let storage = JsonFile::new(&path, &label);
    let wallet = <JsonFile as Storage<SpWallet>>::load(&storage)?;
    let mut psbt = Psbt::from_str(&psbt)?;

    let partial_secret = wallet
        .get_client()
        .get_partial_secret_from_psbt(&psbt)?;

    wallet
        .get_client()
        .fill_sp_outputs(&mut psbt, partial_secret)?;

    Ok(psbt.to_string())
}

#[flutter_rust_bridge::frb(sync)] 
pub fn sign_psbt(
    path: String,
    label: String,
    psbt: String,
    finalize: bool,
) -> Result<String> {
    let storage = JsonFile::new(&path, &label);
    let wallet = <JsonFile as Storage<SpWallet>>::load(&storage)?;
    let psbt = Psbt::from_str(&psbt)?;

    let mut rng = sp_client::silentpayments::secp256k1::rand::thread_rng();
    let mut aux_rand = [0u8; 32];
    rng.fill_bytes(&mut aux_rand);

    let mut signed = wallet
        .get_client()
        .sign_psbt(psbt, &aux_rand)?;

    if finalize {
        SpClient::finalize_psbt(&mut signed)?;
    }

    Ok(signed.to_string())
}

pub fn extract_tx_from_psbt(psbt: String) -> Result<String> {
    let psbt = Psbt::from_str(&psbt)?;

    let final_tx = psbt.extract_tx()?;
    Ok(serialize_hex(&final_tx))
}

pub fn broadcast_tx(tx: String) -> Result<String> {
    let tx: pushtx::Transaction = tx.parse().unwrap();

    let txid = tx.txid();

    let opts = pushtx::Opts {
        network: pushtx::Network::Signet,
        ..Default::default()
    };

    let receiver = pushtx::broadcast(vec![tx], opts);

    loop {
        match receiver.recv().unwrap() {
            pushtx::Info::Done(Ok(report)) => {
                info!("broadcasted to {} peers", report.broadcasts);
                break;
            }
            pushtx::Info::Done(Err(err)) => return Err(anyhow!(err.to_string())),
            _ => {}
        }
    }

    Ok(txid.to_string())
}

#[flutter_rust_bridge::frb(sync)] 
pub fn mark_outpoint_spent(
    path: String,
    label: String,
    outpoint: String,
    txid: String,
) -> Result<()> {
    let storage = JsonFile::new(&path, &label);
    let mut wallet = <JsonFile as Storage<SpWallet>>::load(&storage)?;
    wallet
        .get_mut_outputs()
        .mark_spent(
            OutPoint::from_str(&outpoint)?,
            Txid::from_str(&txid)?,
            true,
        )?;

    Ok(())
}

pub fn show_mnemonic(path: String, label: String) -> Result<Option<String>> {
    let storage = JsonFile::new(&path, &label);
    let wallet = <JsonFile as Storage<SpWallet>>::load(&storage)?;
    let mnemonic = wallet.get_client().get_mnemonic();

    Ok(mnemonic)
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}
