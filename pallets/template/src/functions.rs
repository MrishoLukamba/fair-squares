
pub use crate::items::*;

use frame_support::{
    sp_runtime::traits::{Hash},
    storage::child
 };
use frame_support::inherent::Vec;
use scale_info::{ prelude::vec };


 
impl<T: Config> Pallet<T> {
   
      /// Each fund stores information about its contributors and their contributions in a child trie
      // This helper function calculates the id of the associated child trie.
      pub fn id_from_index() -> child::ChildInfo {
         let mut buf = Vec::new();
         buf.extend_from_slice(b"treasury");
         //buf.extend_from_slice(&index.to_le_bytes()[..]);

         child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
      }
      
      /// Record a contribution in the associated child trie.
      pub fn contribution_put( who: &T::AccountId, balance: &BalanceOf<T>) {
         let id = Self::id_from_index();
         who.using_encoded(|b| child::put(&id, b, &balance));
      }
   
      /// Lookup a contribution in the associated child trie.
      pub fn contribution_get(who: &T::AccountId) -> BalanceOf<T> {
         let id = Self::id_from_index();
         who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
      }
      
      /// Remove a contribution from an associated child trie.
      pub fn contribution_kill(who: &T::AccountId) {
         let id = Self::id_from_index();
         who.using_encoded(|b| child::kill(&id, b));
      }

      pub fn set_roles(account_id: AccountIdOf<T>, role: Role) {
         match role {
            Role::HOUSE_OWNER => {
               HouseOwner::<T>::create(account_id);
            },
            Role::INVESTOR => {
               Investor::<T>::create(account_id);
            },
            Role::TENANT => {
               Tenant::<T>::create(account_id);
            }
         }
      }

      pub fn create_account(account_id: AccountIdOf<T>, role: Role) -> bool {
         match role {
            Role::HOUSE_OWNER => {
               let result = HouseOwner::<T>::create(account_id);
               result
            },
            Role::INVESTOR => {
               let result = Investor::<T>::create(account_id);
               result
            },
            Role::TENANT => {
               let result = Tenant::<T>::create(account_id);
               result
            }
         }
      }

      pub fn validate_proposal_amount(amount: BalanceOf<T>) -> bool {
         true
      }
   }