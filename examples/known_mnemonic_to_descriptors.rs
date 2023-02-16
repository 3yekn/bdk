// Copyright (c) 2020-2021 Bitcoin Dev Kit Developers
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use bdk::blockchain::{ElectrumBlockchain};
use bdk::{descriptor, SyncOptions};
use bdk::database::MemoryDatabase;
use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::util::bip32::DerivationPath;
use bdk::bitcoin::Network;
use bdk::descriptor::IntoWalletDescriptor;
use bdk::keys::bip39::{Language, Mnemonic};
use std::error::Error;
use std::str::FromStr;
use bdk::Wallet;
use electrum_client::Client;

/// This example demonstrates how to generate a mnemonic phrase
/// using BDK and use that to generate a descriptor string.
fn main() -> Result<(), Box<dyn Error>> {
    let secp = Secp256k1::new();

    // Coinstr Alice
    const MNEMONIC: &str = "carry surface crater rude auction ritual banana elder shuffle much wonder decrease";
    const PASSPHRASE: &str = "oy+hB/qeJ1AasCCR";
    
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, &MNEMONIC).unwrap();

    println!("Mnemonic phrase   : {}", &mnemonic);
    println!("Passphrase        : {}", &PASSPHRASE);
    let mnemonic_with_passphrase = (mnemonic, Some(PASSPHRASE.to_string()));

    // define external and internal derivation key path
    let external_path = DerivationPath::from_str("m/86h/0h/0h/0").unwrap();
    let internal_path = DerivationPath::from_str("m/86h/0h/0h/1").unwrap();

    // generate external and internal descriptor from mnemonic
    let (external_descriptor, ext_keymap) =
        descriptor!(tr((mnemonic_with_passphrase.clone(), external_path)))?
            .into_wallet_descriptor(&secp, Network::Testnet)?;

    let (internal_descriptor, int_keymap) =
        descriptor!(tr((mnemonic_with_passphrase.clone(), internal_path)))?
            .into_wallet_descriptor(&secp, Network::Testnet)?;

    println!("tpub external descriptor: {}", external_descriptor);
    println!("tpub internal descriptor: {}", internal_descriptor);
    println!(
        "tprv external descriptor: {}",
        external_descriptor.to_string_with_secret(&ext_keymap)
    );
    println!(
        "tprv internal descriptor: {}",
        internal_descriptor.to_string_with_secret(&int_keymap)
    );

    // create client for Blockstream's testnet electrum server
    let blockchain =
        ElectrumBlockchain::from(Client::new("ssl://electrum.blockstream.info:60002")?);

    // create signing wallet
    let signing_wallet: Wallet<MemoryDatabase> = Wallet::new(
        external_descriptor.clone(),
        Some(internal_descriptor),
        Network::Testnet,
        MemoryDatabase::default(),
    )?;

    println!("Syncing wallet.");
    signing_wallet.sync(&blockchain, SyncOptions::default())?;

    let balance = signing_wallet.get_balance()?;
    println!("wallet balances in SATs: {}", balance);

    // why no signers? 
    println!("\nNumber of signers in signing wallet   {}", signing_wallet.get_signers(bdk::KeychainKind::External).signers().len());

    println!("{:#?}", external_descriptor);

    Ok(())
}
