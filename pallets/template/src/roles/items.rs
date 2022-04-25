pub use super::*;

pub type StorageIndex = u32;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;



#[scale_info(skip_type_params(T))]
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Contribution<T: Config> {
   pub amount: BalanceOf<T>,
   pub timestamp: BlockNumberOf<T>
}

#[scale_info(skip_type_params(T))]
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FundSharing<T: Config> {
    pub amount: BalanceOf<T>,
    pub share: u32
}

#[scale_info(skip_type_params(T))]
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Role {
    pub roles: Vec<u16>
}
impl Role {
    pub fn new() -> Self {
       Self {
          roles: Vec::<u16>::new()
       }
    }
 }


 #[scale_info(skip_type_params(T))]
 #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
 pub struct Ownership<T: Config> {
     pub id: u32,
     pub house_id: u32,
     pub account_id: AccountIdOf<T>,
     pub share: u32,
     pub timestamp: BlockNumberOf<T>,
     pub active: bool
 }
 impl<T: Config> Ownership<T> {
     pub fn new(id: u32, house_id: u32, account_id: AccountIdOf<T>, share: u32, timestamp: BlockNumberOf<T>, active: bool) -> Self {
         Self {
            id,
            house_id,
            account_id,
            share,
            timestamp,
            active,
         }
     }
 }


 #[scale_info(skip_type_params(T))]
 #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
 pub struct HouseMinted<T: Config, U> {
     pub id: StorageIndex,
     pub nft: U,
     pub timestamp: BlockNumberOf<T>,
     pub ownerships: Vec<StorageIndex>
 }
impl<T: Config, U> HouseMinted<T, U> {
    pub fn new(id: StorageIndex, nft: U, timestamp: BlockNumberOf<T>) -> Self {
        Self {
            id,
            nft,
            timestamp,
            ownerships: Vec::<StorageIndex>::new()
        }
    }
}

#[scale_info(skip_type_params(T))]
 #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
 pub struct Proposal<T: Config> {
     pub id: StorageIndex,
     pub house_id: StorageIndex,
     pub account_id: AccountIdOf<T>,
     pub valuation: u32,
     pub active: bool,
     pub funded: bool,
    //  pub votes: Vec<VoteBis<T>>,
     pub timestamp: BlockNumberOf<T>
 }
 impl<T: Config> Proposal<T> {
     pub fn new(id: StorageIndex, house_id: StorageIndex, account_id: AccountIdOf<T>, valuation: u32, timestamp: BlockNumberOf<T>, active:bool, funded: bool) -> Self {
         Self {
             id,
             house_id,
             account_id,
             valuation,
             active,
             funded,
             timestamp
            //  votes: Vec::new()
         }
     }

    //  pub fn add_vote(&mut self, vote: VoteBis<T>) {
    //     self.votes.push(vote);
    //  }
 }

 #[scale_info(skip_type_params(T))]
 #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
 pub struct Vote<T: Config> {
    pub id: StorageIndex,
    pub account_id: AccountIdOf<T>,
    pub status: bool,
    pub timestamp: BlockNumberOf<T>
 }
