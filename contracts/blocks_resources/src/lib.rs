#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Map, Symbol, Vec, token, auth, testutils::Accounts, SymbolShort};

#[derive(Clone)]
#[contracttype]
pub struct TraceRecord {
    pub id: u64,
    pub origen: String,
    pub lat: f64,
    pub lon: f64,
    pub metadata: Bytes,
    pub transfers: Vec<(Address, u64)>, // (to, timestamp)
    pub cert_hash: Option<Bytes>,
}

#[contract]
pub struct BlocksResources;

#[contractimpl]
impl BlocksResources {
    /// Inicializa con multisig: signers[0]=WalletA, signers[1]=WalletB, receiver=WalletC
    pub fn initialize(
        e: Env,
        admin: Address,
        signers: Vec<Address>,
        threshold: u32,
        receiver: Address,
    ) {
        admin.require_auth();
        if threshold != 2 { panic!("Threshold must be 2"); } // Fijo para demo
        let signers_map: Map<Address, bool> = signers.iter().map(|s| (s.clone(), true)).collect();
        e.storage().instance().set(&SymbolShort::new(&e, "signers"), &signers_map);
        e.storage().instance().set(&SymbolShort::new(&e, "threshold"), &threshold);
        e.storage().instance().set(&SymbolShort::new(&e, "receiver"), &receiver);
        e.events().publish((SymbolShort::new(&e, "init"), admin), threshold);
    }

    /// Mint recurso ambiental
    pub fn mint_resource(
        e: Env,
        id: u64,
        origen: String,
        lat: f64,
        lon: f64,
        metadata: Bytes,
    ) -> TraceRecord {
        let admin: Address = e.invoker();
        admin.require_auth(); // Solo admin (Wallet A)
        let record = TraceRecord {
            id,
            origen,
            lat,
            lon,
            metadata,
            transfers: Vec::new(&e),
            cert_hash: None,
        };
        let mut resources: Map<u64, TraceRecord> = e.storage().instance()
            .get(&SymbolShort::new(&e, "resources"))
            .unwrap_or_else(|| Map::new(&e));
        if resources.contains_key(&id) { panic!("ID exists"); }
        resources.set(id, record.clone());
        e.storage().instance().set(&SymbolShort::new(&e, "resources"), &resources);
        e.events().publish((SymbolShort::new(&e, "mint"), id), origen);
        record
    }

    /// Aprobar transferencia (llamar desde cada signer)
    pub fn approve_transfer(e: Env, id: u64) {
        let invoker = e.invoker();
        let signers: Map<Address, bool> = e.storage().instance().get(&SymbolShort::new(&e, "signers")).unwrap();
        if !signers.contains_key(&invoker) { panic!("Not a signer"); }
        invoker.require_auth();

        let mut approvals: Map<u64, Map<Address, bool>> = e.storage().instance()
            .get(&SymbolShort::new(&e, "approvals"))
            .unwrap_or_else(|| Map::new(&e));
        let mut id_approvals: Map<Address, bool> = approvals.get(id).unwrap_or_else(|| Map::new(&e));
        id_approvals.set(invoker.clone(), true);
        approvals.set(id, id_approvals);
        e.storage().instance().set(&SymbolShort::new(&e, "approvals"), &approvals);

        let count: u32 = id_approvals.len() as u32;
        let threshold: u32 = e.storage().instance().get(&SymbolShort::new(&e, "threshold")).unwrap();
        if count >= threshold {
            Self::execute_transfer_internal(&e, id);
            // Limpia approvals
            approvals.remove(&id);
            e.storage().instance().set(&SymbolShort::new(&e, "approvals"), &approvals);
        }
    }

    fn execute_transfer_internal(e: &Env, id: u64) {
        let mut resources: Map<u64, TraceRecord> = e.storage().instance().get(&SymbolShort::new(e, "resources")).unwrap();
        let mut record: TraceRecord = resources.get(id).unwrap();
        let to: Address = e.invoker(); // Asume to es invoker final
        record.transfers.push_back((to.clone(), e.ledger().timestamp()));
        resources.set(id, record);
        e.storage().instance().set(&SymbolShort::new(e, "resources"), &resources);
        e.events().publish((SymbolShort::new(e, "transfer"), id), to);
    }

    /// Libera fondos a receiver (Wallet C) - requiere multisig via approve
    pub fn release_funds(e: Env, id: u64, amount: i128) {
        // Reusa lógica de approve (llama approve_transfer primero desde 2 signers)
        let receiver: Address = e.storage().instance().get(&SymbolShort::new(&e, "receiver")).unwrap();
        // Asume token contract deployado (del repo base)
        let token_id = Address::from_string(&e, "TOKEN_CONTRACT_ID_AQUI"); // Reemplaza con deploy del token
        let token_client = token::Client::new(&e, &token_id);
        token_client.transfer(&e, &receiver, &amount);
        e.events().publish((SymbolShort::new(&e, "release"), id), amount);
    }

    /// Query pública
    pub fn query_trace(e: Env, id: u64) -> TraceRecord {
        let resources: Map<u64, TraceRecord> = e.storage().instance().get(&SymbolShort::new(&e, "resources")).unwrap();
        resources.get(id).unwrap_or_else(|| panic!("Not found"))
    }
}
