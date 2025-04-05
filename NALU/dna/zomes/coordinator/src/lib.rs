use hdk::prelude::*;
use hdk::prelude::hash_type::AnyLinkable;
use nalu_coin_integrity::{EntryTypes, LinkTypes, NaluCoin, Transaction};

#[hdk_extern]
pub fn init(_: ()) -> ExternResult<InitCallbackResult> {
    // Create default account with 100 coins when the agent joins the network
    create_account(())?;
    Ok(InitCallbackResult::Pass)
}

#[hdk_extern]
pub fn create_account(_: ()) -> ExternResult<NaluCoin> {
    let agent_pubkey = agent_info()?.agent_latest_pubkey;
    
    // Define a path for easy retrieval of account
    let path = Path::from("accounts");
    create_path(&path)?;
    
    // Check if account already exists
    match get_account(agent_pubkey.clone())? {
        Some(_) => {
            return Err(wasm_error!(WasmErrorInner::Guest(
                "Account already exists for this agent".into()
            )));
        }
        None => {
            // Create account with initial balance of 100 coins
            let account = NaluCoin {
                owner: agent_pubkey.clone(),
                balance: 100,
            };
            
            let account_hash = create_entry(&EntryTypes::NaluCoin(account.clone()))?;
            
            // Create a link from the path to the account for easy retrieval
            let path_hash = path.path_entry_hash()?;
            create_link(
                path_hash,
                account_hash,
                LinkTypes::AccountLink,
                (),
            )?;
            
            Ok(account)
        }
    }
}

#[hdk_extern]
pub fn get_account(agent_pubkey: AgentPubKey) -> ExternResult<Option<NaluCoin>> {
    let path = Path::from("accounts");
    let path_hash = path.path_entry_hash()?;
    
    // Get all links from the accounts path
    let links_input = GetLinksInputBuilder::try_new(
        path_hash,
        LinkTypes::AccountLink,
    )?.build();
    
    let links = get_links(links_input)?;
    
    // Look for the account belonging to the specified agent
    for link in links {
        let record = get_link_target_record(link.target.clone())?;
        
        match record.entry().to_app_option::<NaluCoin>() {
            Ok(Some(account)) => {
                if account.owner.eq(&agent_pubkey) {
                    return Ok(Some(account));
                }
            },
            _ => continue,
        }
    }
    
    Ok(None)
}

#[hdk_extern]
pub fn get_balance(agent_pubkey: AgentPubKey) -> ExternResult<u64> {
    match get_account(agent_pubkey)? {
        Some(account) => Ok(account.balance),
        None => Ok(0),
    }
}

#[hdk_extern]
pub fn transfer_coins(input: (AgentPubKey, u64)) -> ExternResult<Transaction> {
    let (recipient, amount) = input;
    let sender = agent_info()?.agent_latest_pubkey;
    
    // Check if sender has enough balance
    let sender_account = get_account(sender.clone())?
        .ok_or(wasm_error!(WasmErrorInner::Guest("Sender account not found".into())))?;
    
    if sender_account.balance < amount {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Insufficient balance for transfer".into()
        )));
    }
    
    // Get recipient account
    let recipient_account = get_account(recipient.clone())?
        .ok_or(wasm_error!(WasmErrorInner::Guest(
            "Recipient account doesn't exist".into()
        )))?;
    
    // Create updated sender account
    let updated_sender_account = NaluCoin {
        owner: sender.clone(),
        balance: sender_account.balance - amount,
    };
    
    // Create updated recipient account
    let updated_recipient_account = NaluCoin {
        owner: recipient.clone(),
        balance: recipient_account.balance + amount,
    };
    
    // Record transaction
    let timestamp = sys_time()?.as_micros() as u64;
    let transaction = Transaction {
        sender: sender.clone(),
        receiver: recipient.clone(),
        amount,
        timestamp,
    };
    
    // Create transaction entry
    let _transaction_hash = create_entry(&EntryTypes::Transaction(transaction.clone()))?;
    
    // Update accounts
    let path = Path::from("accounts");
    let path_hash = path.path_entry_hash()?;
    
    // Update sender account
    let new_sender_hash = create_entry(&EntryTypes::NaluCoin(updated_sender_account))?;
    create_link(
        path_hash.clone(),
        new_sender_hash,
        LinkTypes::AccountLink,
        (),
    )?;
    
    // Update recipient account
    let new_recipient_hash = create_entry(&EntryTypes::NaluCoin(updated_recipient_account))?;
    create_link(
        path_hash,
        new_recipient_hash,
        LinkTypes::AccountLink,
        (),
    )?;
    
    Ok(transaction)
}

#[hdk_extern]
pub fn get_transactions(agent_pubkey: AgentPubKey) -> ExternResult<Vec<Transaction>> {
    let path = Path::from("accounts");
    let path_hash = path.path_entry_hash()?;
    
    // Get all links from the accounts path
    let links_input = GetLinksInputBuilder::try_new(
        path_hash,
        LinkTypes::AccountLink,
    )?.build();
    
    let links = get_links(links_input)?;
    
    let mut transactions = Vec::new();
    
    // Look for transactions
    for link in links {
        let record = get_link_target_record(link.target.clone())?;
        
        match record.entry().to_app_option::<Transaction>() {
            Ok(Some(transaction)) => {
                if transaction.sender.eq(&agent_pubkey) || transaction.receiver.eq(&agent_pubkey) {
                    transactions.push(transaction);
                }
            },
            _ => continue,
        }
    }
    
    // Sort transactions by timestamp, newest first
    transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Ok(transactions)
}

// Helper function to create a path
fn create_path(path: &Path) -> ExternResult<()> {
    path.path_entry_hash()?;
    Ok(())
}

// Helper function to get a record from a link target
fn get_link_target_record(link_target: HoloHash<AnyLinkable>) -> ExternResult<Record> {
    // Try to convert AnyLinkable to ActionHash
    let action_hash = match link_target.into_action_hash() {
        Some(hash) => hash,
        None => return Err(wasm_error!(WasmErrorInner::Guest("Could not convert link target to action hash".into()))),
    };
    
    // Get the record
    match get(action_hash, GetOptions::default())? {
        Some(record) => Ok(record),
        None => Err(wasm_error!(WasmErrorInner::Guest("Record not found".into()))),
    }
}