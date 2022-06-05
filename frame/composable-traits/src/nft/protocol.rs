/// protocol nft is aware of NFT and protocols, so it manages to do complex intercation
pub trait ProtocolNft<AccountId> {
    fn split_into(
        instance: &Self::InstanceId,
        parts: BiBoundedVec<Permill, 1, 16>,
    ) -> Result<BiBoundedVec<Self::InstanceId, 1, 16>, DispatchError>;
}