# Nalu Coin - Holochain Community Currency

A simple community digital currency built on Holochain.

## Description

Nalu Coin enables users to:
- Create accounts (with 100 initial coins)
- Check account balances
- Transfer coins between users
- View transaction history

## Architecture

This app follows the latest Holochain architecture, using:
- Integrity zome: Defines entry types and validation rules
- Coordinator zome: Implements business logic and user-facing functions

### Entry Types

1. **NaluCoin** - Stores user balance information
   ```rust
   #[hdk_entry_helper]
   #[derive(Clone)]
   pub struct NaluCoin {
       pub owner: AgentPubKey,
       pub balance: u64,
   }
   ```

2. **Transaction** - Records transfer details
   ```rust
   #[hdk_entry_helper]
   #[derive(Clone)]
   pub struct Transaction {
       pub sender: AgentPubKey,
       pub receiver: AgentPubKey,
       pub amount: u64,
       pub timestamp: u64,
   }
   ```

### Zome Functions

1. **create_account()** - Creates a new account with 100 initial coins
   ```rust
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
   ```

2. **get_balance(agent_pubkey)** - Returns the balance for any agent
   ```rust
   #[hdk_extern]
   pub fn get_balance(agent_pubkey: AgentPubKey) -> ExternResult<u64> {
       match get_account(agent_pubkey)? {
           Some(account) => Ok(account.balance),
           None => Ok(0),
       }
   }
   ```

3. **transfer_coins(recipient, amount)** - Transfers coins between accounts
   ```rust
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
       
       // Create transaction entry and update accounts
       create_entry(&EntryTypes::Transaction(transaction.clone()))?;
       create_entry(&EntryTypes::NaluCoin(updated_sender_account))?;
       create_entry(&EntryTypes::NaluCoin(updated_recipient_account))?;
       
       Ok(transaction)
   }
   ```

4. **get_transactions(agent_pubkey)** - Retrieves transaction history
   ```rust
   #[hdk_extern]
   pub fn get_transactions(agent_pubkey: AgentPubKey) -> ExternResult<Vec<Transaction>> {
       // Retrieves all transactions that involve the specified agent
       // Returns a list of transactions sorted by timestamp (newest first)
   }
   ```

## Data Organization

The app uses Holochain's entry-based data model with:
- Entry types defined in the integrity zome
- Business logic implemented in the coordinator zome
- Links to associate accounts with agents
- Paths for easy data retrieval

## Building and Running

1. **Build the project**:
   ```
   cargo build --release --target wasm32-unknown-unknown
   ```

2. **Pack the DNA and hApp**:
   ```
   hc dna pack
   hc app pack
   ```

3. **Run in a development environment**:
   ```
   hc-sandbox run -n 2 workdir/nalu-coin/nalu-coin.happ
   ```

4. **Call zome functions** (Example with `hc-sandbox call`):
   ```
   # Create an account (happens automatically on initialization)
   hc-sandbox call nalu-coin coordinator create_account '{}'
   
   # Get your own balance
   hc-sandbox call nalu-coin coordinator get_balance '{agent_pubkey: <your-pubkey>}'
   
   # Transfer coins
   hc-sandbox call nalu-coin coordinator transfer_coins '[<recipient-pubkey>, 10]'
   
   # Get transaction history
   hc-sandbox call nalu-coin coordinator get_transactions '{agent_pubkey: <your-pubkey>}'
   ```

## License

MIT