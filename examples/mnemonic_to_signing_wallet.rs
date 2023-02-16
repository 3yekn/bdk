// Copyright (c) 2020-2021 Bitcoin Dev Kit Developers
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use bdk::blockchain::ElectrumBlockchain;
use electrum_client::Client;
use bdk::SyncOptions;
use bitcoin::Address;
use bdk::FeeRate;
use bdk::SignOptions;
use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::util::bip32::DerivationPath;
use miniscript::descriptor::DescriptorSecretKey;
use bdk::bitcoin::Network;
use bdk::descriptor;
use bdk::descriptor::IntoWalletDescriptor;
use bdk::keys::bip39::{Language, Mnemonic};
// use bdk::miniscript::Tap;
use std::error::Error;
use std::str::FromStr;
use bdk::Wallet;
use bdk::database::MemoryDatabase;

/// This example demonstrates how to generate a mnemonic phrase
/// using BDK and use that to generate a descriptor string.
fn main() -> Result<(), Box<dyn Error>> {
    let secp = Secp256k1::new();

    // In this example we are generating a 12 words mnemonic phrase
    // but it is also possible generate 15, 18, 21 and 24 words
    // using their respective `WordCount` variant.
    // let mnemonic: GeneratedKey<_, Tap> =
    //     Mnemonic::generate((WordCount::Words12, Language::English))
    //         .map_err(|_| BDK_Error::Generic("Mnemonic generation error".to_string()))?;

    const MNEMONIC: &str = "carry surface crater rude auction ritual banana elder shuffle much wonder decrease";
    const PASSPHRASE: &str = "oy+hB/qeJ1AasCCR";

    let mnemonic = Mnemonic::parse_in_normalized(Language::English, &MNEMONIC).unwrap();

    println!("Mnemonic phrase: {}", &mnemonic);
    let mnemonic_with_passphrase = (mnemonic, Some(PASSPHRASE.to_string()));

    // define external and internal derivation key path
    let external_path = DerivationPath::from_str("m/86h/0h/0h/0").unwrap();
    let internal_path = DerivationPath::from_str("m/86h/0h/0h/1").unwrap();

    // generate external and internal descriptor from mnemonic
    let (external_descriptor, ext_keymap) =
        descriptor!(tr((mnemonic_with_passphrase.clone(), external_path)))?
            .into_wallet_descriptor(&secp, Network::Testnet)?;

    let (internal_descriptor, int_keymap) =
        descriptor!(tr((mnemonic_with_passphrase, internal_path)))?
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

    let external_secret_xkey = DescriptorSecretKey::from_str(external_descriptor.to_string_with_secret(&ext_keymap).as_str()).unwrap();
    let internal_secret_xkey = DescriptorSecretKey::from_str(internal_descriptor.to_string_with_secret(&int_keymap).as_str()).unwrap();

    let signing_external_descriptor = descriptor!(tr(external_secret_xkey)).unwrap();
    let signing_internal_descriptor = descriptor!(tr(internal_secret_xkey)).unwrap();

    println!("Signing external descriptor   : \n{:#?}\n", signing_external_descriptor);

    // create signing wallet
    let signing_wallet: Wallet<MemoryDatabase> = Wallet::new(
        signing_external_descriptor,
        Some(signing_internal_descriptor),
        Network::Testnet,
        MemoryDatabase::default(),
    )?;

    // create client for Blockstream's testnet electrum server
    let blockchain =
        ElectrumBlockchain::from(Client::new("ssl://electrum.blockstream.info:60002")?);

    println!("Syncing signing wallet.");
    signing_wallet.sync(&blockchain, SyncOptions::default())?;

    println!("New address   {}", signing_wallet.get_address(bdk::wallet::AddressIndex::New).unwrap());

    // there are no signers and this is why the sign function below does nothing
    // and the PSBT is not finalized
    println!("\nNumber of signers in signing wallet   {}", signing_wallet.get_signers(bdk::KeychainKind::External).signers().len());

    for secret_key in signing_wallet
        .get_signers(bdk::KeychainKind::External)
        .signers()
        .iter()
        .filter_map(|s| s.descriptor_secret_key())
    {
        println!("secret_key: {}", secret_key);
    }

    let return_address = Address::from_str("tb1ql7w62elx9ucw4pj5lgw4l028hmuw80sndtntxt")?;
    let mut builder = signing_wallet.build_tx();
    builder
        .add_recipient(return_address.script_pubkey(), 300)
        .enable_rbf()
        .fee_rate(FeeRate::from_sat_per_vb(1.0));

    let (mut psbt, details) = builder.finish()?;
    println!("Transaction details: {:#?}", details);
    println!("\nUnsigned PSBT: \n{}", psbt);

    // Sign and finalize the PSBT with the signing wallet
    let finalized = signing_wallet.sign(&mut psbt, SignOptions::default())?;
    println!("\nAttempted to be signed PSBT: \n{}\n", psbt);

    assert!(finalized, "The PSBT was not finalized!");
    println!("The PSBT has been signed and finalized.");    

    Ok(())
}
