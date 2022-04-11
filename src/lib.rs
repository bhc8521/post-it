

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::thread::LocalKey;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base58CryptoHash, U128, U64};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::serde_json::{json, self};
use near_sdk::{env, near_bindgen, AccountId, log, bs58, PanicOnDefault, Promise, BlockHeight, CryptoHash, StorageUsage, Balance};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector, UnorderedSet};

pub mod view;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct PostIt {
    accounts: LookupMap<PosterId, Poster>,
    posts: UnorderedMap<Base58CryptoHash, Post>,
    public_posts: Vector<PublicPost>
}

#[derive(BorshDeserialize, BorshSerialize)]
#[derive(Debug)]
pub struct Poster {
    send_by_account: UnorderedMap<PosterId, Vector<Base58CryptoHash>>,
    receive_by_account: UnorderedMap<PosterId, Vector<Base58CryptoHash>>,
    send: UnorderedSet<Base58CryptoHash>,
    receive: UnorderedSet<Base58CryptoHash>
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Clone)]
pub enum PosterId {
    AccountId(AccountId),
    Base58CryptoHash(Base58CryptoHash)
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Clone)]
pub enum PosterIdWithHint {
    AccountId(AccountId),
    Base58CryptoHash(Base58CryptoHash, String)
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct Post {
    sender: PosterId,
    receiver: PosterIdWithHint,
    text: String,
    timestamp: U64,
    encrypted: bool
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct PublicPost {
    sender: AccountId,
    text: String,
    timestamp: U64,
}

#[near_bindgen]
impl PostIt {

    #[init]
    pub fn new() -> Self {
        Self {
            accounts: LookupMap::new(b'a'),
            posts: UnorderedMap::new(b'p'),
            public_posts: Vector::new(b'd')
        }
    }

    #[payable]
    pub fn post_to_public(&mut self, text: String) {
        let initial_storage_usage = env::storage_usage();
        let post = PublicPost {
            sender: env::predecessor_account_id(),
            text,
            timestamp: env::block_timestamp().into()
        };
        self.public_posts.push(&post);

        refund_extra_storage_deposit(env::storage_usage() - initial_storage_usage, 0);
    }

    #[payable]
    pub fn post(&mut self, text: String, account_id: PosterIdWithHint) {
        let initial_storage_usage = env::storage_usage();
        let (sender_id, sender_id_str) = (PosterId::AccountId(env::predecessor_account_id()), env::predecessor_account_id().to_string());
        let (receiver_id, receiver_id_str, encrypted) = match account_id.clone() {
            PosterIdWithHint::AccountId(v) => (PosterId::AccountId(v.clone()), v.to_string(), false),
            PosterIdWithHint::Base58CryptoHash(hash, _) => (PosterId::Base58CryptoHash(hash), String::from(&hash), true)
        };

        let mut sender = self.accounts.get(&sender_id).unwrap_or(Poster {
            send_by_account: UnorderedMap::new((sender_id_str.clone().to_string() + "send_by_account").as_bytes()),
            receive_by_account: UnorderedMap::new((sender_id_str.to_string() + "receive_by_account").as_bytes()),
            send: UnorderedSet::new((sender_id_str.clone().to_string() + "send").as_bytes()),
            receive: UnorderedSet::new((sender_id_str.clone().to_string() + "receive").as_bytes()),
        });
        let mut receiver = self.accounts.get(&receiver_id).unwrap_or(Poster {
            send_by_account: UnorderedMap::new((receiver_id_str.to_string() + "send_by_account").as_bytes()),
            receive_by_account: UnorderedMap::new((receiver_id_str.to_string() + "receive_by_account").as_bytes()),
            send: UnorderedSet::new((receiver_id_str.clone().to_string() + "send").as_bytes()),
            receive: UnorderedSet::new((receiver_id_str.clone().to_string() + "receive").as_bytes()),
        });
        let post = Post {
            sender: sender_id.clone(),
            receiver: account_id.clone(),
            text: text,
            timestamp: env::block_timestamp().into(),
            encrypted
        };
        let hash: CryptoHash = env::sha256(json!(post).to_string().as_bytes()).try_into().unwrap();
        let hash = Base58CryptoHash::from(hash);
        self.posts.insert(&hash, &post);

        let mut send_by_account = sender.send_by_account.get(&receiver_id).unwrap_or(Vector::new((sender_id_str.to_string() + &receiver_id_str).as_bytes()));
        send_by_account.push(&hash);
        sender.send_by_account.insert(&receiver_id, &send_by_account);
        sender.send.insert(&hash);

        let mut receive_by_account = receiver.receive_by_account.get(&sender_id).unwrap_or(Vector::new((sender_id_str.to_string() + &receiver_id_str).as_bytes()));
        receive_by_account.push(&hash);
        receiver.receive_by_account.insert(&sender_id, &receive_by_account);
        receiver.receive.insert(&hash);

        refund_extra_storage_deposit(env::storage_usage() - initial_storage_usage, 0);
    }
}

pub(crate) fn refund_extra_storage_deposit(storage_used: StorageUsage, used_balance: Balance) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit()
        .checked_sub(used_balance)
        .expect("not enough attached balance");

    assert!(
        required_cost <= attached_deposit,
        "not enough attached balance {}",
        required_cost,
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}



#[cfg(test)]
mod tests {


}