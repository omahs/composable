use composable_support::validation::Validate;
use frame_support::{pallet_prelude::*, traits::Get};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, Percent};

#[derive(Debug, Decode)]
pub struct ValidProposal<U>{
    _market: PhantomData<U>,
}

impl<U> Copy for ValidProposal<U> {}

impl<U> Clone for ValidProposal<U> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<Proposal: PartialOrd, MinimumDeposit> Validate<Proposal, ValidProposal<MinimumDeposit>> for 
   ValidProposal<MinimumDeposit> where MinimumDeposit: Get<Proposal> {
       fn validate(input: Proposal) -> Result<Proposal, &'static str> {

           if input < MinimumDeposit::get() {
               return Err("PROPOSAL_VALUE_LOWER_THAN_MINIMUM")
           }
          Ok(input)
       }
   }

