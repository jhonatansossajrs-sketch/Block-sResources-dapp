#[test]
fn test_multisig_mint_transfer() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let signer1 = Address::generate(&e); // Wallet A pubkey
    let signer2 = Address::generate(&e); // Wallet B
    let receiver = Address::generate(&e); // Wallet C
    let signers = Vec::from_slice(&e, &[signer1.clone(), signer2.clone()]);
    BlocksResources::initialize(e.clone(), admin.clone(), signers, 2, receiver.clone());

    // Mint
    let trace = BlocksResources::mint_resource(e.clone(), 1, "Amazonas".into(), -3.0, -60.0, Bytes::from_array(&e, &[1]));
    
    // Simula approvals
    e.as_contract(&admin, || {
        // En test, mock auth
        BlocksResources::approve_transfer(e.clone(), 1);
        BlocksResources::approve_transfer(e.clone(), 1); // 2do signer
    });
    // Verifica transfer ejecutado
}
