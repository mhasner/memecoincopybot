use solana_sdk::{
    pubkey::Pubkey,
    signature::Signer,
    transaction::VersionedTransaction,
    message::VersionedMessage,
    system_instruction,
};

pub fn build_tip_only_tx(
    payer: &solana_sdk::signature::Keypair,
    to: &Pubkey,
    lamports: u64,
    rpc: &solana_client::rpc_client::RpcClient,
) -> anyhow::Result<VersionedTransaction> {
    let ix  = system_instruction::transfer(&payer.pubkey(), to, lamports);
    let bh  = rpc.get_latest_blockhash()?;
    let msg = VersionedMessage::Legacy(
        solana_sdk::message::Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &bh),
    );
    Ok(VersionedTransaction::try_new(msg, &[payer])?)
}
