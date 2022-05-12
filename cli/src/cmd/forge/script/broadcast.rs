use crate::cmd::ScriptSequence;

use ethers::{
    prelude::{Provider, SignerMiddleware},
    providers::Middleware,
    signers::Signer,
    types::{
        transaction::eip2718::TypedTransaction, Address, Chain, Eip1559TransactionRequest,
        TransactionReceipt, TransactionRequest, U256,
    },
};

use foundry_utils::RuntimeOrHandle;
use std::collections::BTreeMap;

use super::*;

impl ScriptArgs {
    pub fn send_transactions(&self, deployment_sequence: &mut ScriptSequence) -> eyre::Result<()> {
        // The user wants to actually send the transactions
        let mut local_wallets = vec![];
        if let Some(wallets) = self.wallets.private_keys()? {
            wallets.into_iter().for_each(|wallet| local_wallets.push(wallet));
        }

        if let Some(wallets) = self.wallets.interactives()? {
            wallets.into_iter().for_each(|wallet| local_wallets.push(wallet));
        }

        if let Some(wallets) = self.wallets.mnemonics()? {
            wallets.into_iter().for_each(|wallet| local_wallets.push(wallet));
        }

        if let Some(wallets) = self.wallets.keystores()? {
            wallets.into_iter().for_each(|wallet| local_wallets.push(wallet));
        }

        // TODO: Add trezor and ledger support (supported in multiwallet, just need to
        // add derivation + SignerMiddleware creation logic)
        // foundry/cli/src/opts/mod.rs:110
        if local_wallets.is_empty() {
            panic!("Error accessing local wallet when trying to send onchain transaction, did you set a private key, mnemonic or keystore?")
        }

        let fork_url = self
            .evm_opts
            .fork_url
            .as_ref()
            .expect("You must provide an RPC URL (see --fork-url).")
            .clone();
        let provider = Provider::try_from(&fork_url).expect("Bad fork provider.");

        let rt = RuntimeOrHandle::new();
        let chain = rt.block_on(provider.get_chainid())?.as_u64();
        let is_legacy =
            self.legacy || Chain::try_from(chain).map(|x| Chain::is_legacy(&x)).unwrap_or_default();
        local_wallets =
            local_wallets.into_iter().map(|wallet| wallet.with_chain_id(chain)).collect();

        // in case of --force-resume, we forgive the first nonce disparity of each from
        let mut nonce_offset: BTreeMap<Address, U256> = BTreeMap::new();

        // Iterate through transactions, matching the `from` field with the associated
        // wallet. Then send the transaction. Panics if we find a unknown `from`
        deployment_sequence
            .clone()
            .transactions
            .range((deployment_sequence.index as usize)..)
            .map(|tx| {
                let from = into_legacy_ref(tx).from.expect("No sender for onchain transaction!");
                if let Some(wallet) =
                    local_wallets.iter().find(|wallet| (**wallet).address() == from)
                {
                    let signer = SignerMiddleware::new(provider.clone(), wallet.clone());
                    Ok((tx.clone(), signer))
                } else {
                    Err(eyre::eyre!(format!(
                        "No associated wallet for address: {:?}. Unlocked wallets: {:?}",
                        from,
                        local_wallets
                            .iter()
                            .map(|wallet| wallet.address())
                            .collect::<Vec<Address>>()
                    )))
                }
            })
            .for_each(|payload| {
                match payload {
                    Ok((tx, signer)) => {
                        let mut legacy_or_1559 = if is_legacy {
                            tx
                        } else {
                            TypedTransaction::Eip1559(into_1559(tx))
                        };
                        set_chain_id(&mut legacy_or_1559, chain);

                        let from = *legacy_or_1559.from().expect("no sender");
                        match foundry_utils::next_nonce(from, &fork_url, None) {
                            Ok(nonce) => {
                                let tx_nonce = *legacy_or_1559.nonce().expect("no nonce");
                                let offset = if self.force_resume {
                                    match nonce_offset.get(&from) {
                                        Some(offset) => *offset,
                                        None => {
                                            let offset = nonce - tx_nonce;
                                            nonce_offset.insert(from, offset);
                                            offset
                                        }
                                    }
                                } else {
                                    U256::from(0u32)
                                };

                                if nonce != tx_nonce + offset {
                                    deployment_sequence
                                        .save()
                                        .expect("not able to save deployment sequence");
                                    panic!("EOA nonce changed unexpectedly while sending transactions.");
                                } else if !offset.is_zero() {
                                    legacy_or_1559.set_nonce(tx_nonce + offset);
                                }
                            }
                            Err(_) => {
                                deployment_sequence.save().expect("not able to save deployment sequence");
                                panic!("Not able to query the EOA nonce.");
                            }
                        }

                        async fn send<T, U>(
                            signer: SignerMiddleware<T, U>,
                            legacy_or_1559: TypedTransaction,
                        ) -> eyre::Result<Option<TransactionReceipt>>
                        where
                            SignerMiddleware<T, U>: Middleware,
                        {
                            tracing::debug!("sending transaction: {:?}", legacy_or_1559);
                            match signer.send_transaction(legacy_or_1559, None).await {
                                Ok(pending) => pending.await.map_err(|e| eyre::eyre!(e)),
                                Err(e) => Err(eyre::eyre!(e.to_string())),
                            }
                        }

                        let receipt = match rt.block_on(send(signer, legacy_or_1559)) {
                            Ok(Some(res)) => {
                                let tx_str = serde_json::to_string_pretty(&res).expect("Bad serialization");
                                println!("{}", tx_str);
                                res
                            }

                            Ok(None) => {
                                // todo what if it has been actually sent
                                deployment_sequence.save().expect("not able to save deployment sequence");
                                panic!("Failed to get transaction receipt?")
                            }
                            Err(e) => {
                                deployment_sequence.save().expect("not able to save deployment sequence");
                                panic!("Aborting! A transaction failed to send: {:#?}", e)
                            }
                        };

                        deployment_sequence.add_receipt(receipt);
                        deployment_sequence.index += 1;
                    }
                    Err(e) => {
                        deployment_sequence.save().expect("not able to save deployment sequence");
                        panic!("{e}");
                    }
                }
            });

        deployment_sequence.save()?;

        println!("\n\n==========================");
        println!(
            "\nONCHAIN EXECUTION COMPLETE & SUCCESSFUL. Transaction receipts written to {:?}",
            deployment_sequence.path
        );
        Ok(())
    }
}

pub fn set_chain_id(tx: &mut TypedTransaction, chain_id: u64) {
    match tx {
        TypedTransaction::Legacy(tx) => tx.chain_id = Some(chain_id.into()),
        TypedTransaction::Eip1559(tx) => tx.chain_id = Some(chain_id.into()),
        _ => panic!("Wrong transaction type for expected output"),
    }
}

pub fn into_legacy(tx: TypedTransaction) -> TransactionRequest {
    match tx {
        TypedTransaction::Legacy(tx) => tx,
        _ => panic!("Wrong transaction type for expected output"),
    }
}

pub fn into_legacy_ref(tx: &TypedTransaction) -> &TransactionRequest {
    match tx {
        TypedTransaction::Legacy(ref tx) => tx,
        _ => panic!("Wrong transaction type for expected output"),
    }
}

pub fn into_1559(tx: TypedTransaction) -> Eip1559TransactionRequest {
    match tx {
        TypedTransaction::Legacy(tx) => Eip1559TransactionRequest {
            from: tx.from,
            to: tx.to,
            value: tx.value,
            data: tx.data,
            nonce: tx.nonce,
            ..Default::default()
        },
        TypedTransaction::Eip1559(tx) => tx,
        _ => panic!("Wrong transaction type for expected output"),
    }
}
