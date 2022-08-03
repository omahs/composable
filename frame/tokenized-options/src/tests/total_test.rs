/* You should run this proptest in release mode:
cargo test -r --package pallet-tokenized-options --lib -- tests::total_test::total_proptest --exact --nocapture --ignored
*/

use super::{
	block_producer::{BlockProducer, BlocksConfig},
	random_initial_balances_simpl, random_option_config, OptionConfig, VaultInitializer, UNIT,
};
use crate::mocks::{
	accounts::{account_id_from_u64, AccountId},
	assets::{AssetId, ASSETS_WITH_USDC},
	runtime::{
		Assets, Balance, BlockNumber, ExtBuilder, MockRuntime, Moment, OptionId, Origin,
		TokenizedOptions,
	},
};
use composable_traits::tokenized_options::TokenizedOptions as TokenizedOptionsTrait;
use frame_support::{assert_ok, traits::fungibles::Inspect};
use proptest::prelude::{
	any, prop, prop::sample::Index, prop_compose, proptest, ProptestConfig, Strategy,
};
use sp_runtime::DispatchResult;
use std::{cmp::Reverse, collections::HashMap, ops::Range};

type EpochBlocks = [Vec<(BlockNumber, OptionId)>; 4];

const DEPOSIT_EPOCH_INDEX: usize = 0;
const PURCHASE_EPOCH_INDEX: usize = 1;
const EXERCISE_EPOCH_INDEX: usize = 2;
const END_EPOCH_INDEX: usize = 3;

fn balance_strategy(max: Balance) -> impl Strategy<Value = Balance> + Clone {
	(10..=max).prop_map(|b| b * UNIT)
}

fn random_extrinsic_type() -> impl Strategy<Value = TokenizedOptionsExtrinsicType> + Clone {
	(1..=21).prop_map(|v| match v {
		1..=10 => TokenizedOptionsExtrinsicType::SellOption,
		11..=20 => TokenizedOptionsExtrinsicType::BuyOption,
		_ => TokenizedOptionsExtrinsicType::DeleteSellOption,
	})
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1))]
	#[test]
	#[ignore]
	fn total_proptest(
		(balances, option_configs, extrinsics) in random_values(
			50_000..200_000,              // account_count_rng
			balance_strategy(1_000_000),  // account_balance_strategy
			10..15,                       // option_count_rng
			balance_strategy(1_000),      // option_balance_strategy
			1..100,                       // option_start_rng
			40..50,                       // option_duration_rng
			100_000..900_000,             // extrinsic_count_rng
			random_extrinsic_type(),      // extrinsic_type_strategy
			1..3,                         // option_amount_rng
		),
	) {
		let account_count = balances.len() / ASSETS_WITH_USDC.len();
		ExtBuilder::default()
			.initialize_balances_simpl(&balances)
			.build()
			.initialize_oracle_prices()
			.initialize_all_vaults()
			.execute_with(|| do_total_proptest(account_count, option_configs, extrinsics));
	}
}

prop_compose! {
	#[allow(clippy::too_many_arguments)]
	fn random_values(
		account_count_rng: Range<u64>,
		account_balance_strategy: impl Strategy<Value = Balance> + Clone,
		option_count_rng: Range<usize>,
		option_balance_strategy: impl Strategy<Value = Balance> + Clone,
		option_start_rng: Range<Moment>,
		option_duration_rng: Range<Moment>,
		extrinsic_count_rng: Range<usize>,
		extrinsic_type_strategy: impl Strategy<Value = TokenizedOptionsExtrinsicType> + Clone,
		option_amount_rng: Range<Balance>,
	)(
		account_count in account_count_rng,
		option_count in option_count_rng,
	)(
		balances in random_initial_balances_simpl(account_count, account_balance_strategy.clone()),
		option_configs in prop::collection::vec(random_option_config(option_balance_strategy.clone(), option_start_rng.clone(), option_duration_rng.clone()), option_count),
		extrinsics in prop::collection::vec(TokenizedOptionsExtrinsic::generate(extrinsic_type_strategy.clone(), 1..=account_count, option_amount_rng.clone()), extrinsic_count_rng.clone()),
	) -> (Vec<Balance>, Vec<OptionConfig<AssetId, Balance, Moment>>, Vec<TokenizedOptionsExtrinsic>) {
		(balances, option_configs, extrinsics)
	}
}

fn do_total_proptest(
	account_count: usize,
	mut option_configs: Vec<OptionConfig<AssetId, Balance, Moment>>,
	mut extrinsics: Vec<TokenizedOptionsExtrinsic>,
) {
	dbg!(total_amount(account_count));
	let mut buyers: HashMap<OptionId, Vec<(Balance, AccountId)>> = Default::default();
	let mut sellers: Vec<AccountId> = Default::default();

	let mut moments = Vec::with_capacity(1 + 4 * option_configs.len());
	moments.push(0);
	for option_config in &option_configs {
		moments.push(option_config.epoch.deposit);
		moments.push(option_config.epoch.purchase);
		moments.push(option_config.epoch.exercise);
		moments.push(option_config.epoch.end);
	}
	moments.sort_unstable();
	moments.dedup();
	let mut block_producer = BlockProducer::<TokenizedOptionsBlocksConfig>::new(moments);

	let mut epoch_blocks: EpochBlocks = Default::default();
	let mut exercise_blocks: Vec<(BlockNumber, OptionId)> = Default::default();
	let mut successful = 0;
	let mut errors = 0;
	while block_producer.next_block() {
		if block_producer.is_init_block() {
			epoch_blocks = initialize_options(
				1 + block_producer.block_number(),
				block_producer.remaining_moments(),
				std::mem::take(&mut option_configs),
			);
			extrinsics.sort_by_key(|e| Reverse(e.block_number(&epoch_blocks)));
			exercise_blocks = get_exercise_blocks(&epoch_blocks);
		}
		let extrinsic_same_block = |e: &TokenizedOptionsExtrinsic| {
			e.block_number(&epoch_blocks) == block_producer.block_number()
		};
		while let Some(extrinsic) = pop_if(&mut extrinsics, extrinsic_same_block) {
			let result = extrinsic.run(&epoch_blocks);
			match result {
				Ok(_) => {
					successful += 1;
					if extrinsic.extrinsic_type == TokenizedOptionsExtrinsicType::BuyOption {
						let option_id = extrinsic.option_id(&epoch_blocks);
						let option_amount = extrinsic.option_amount;
						let account_id = extrinsic.account_id;
						buyers.entry(option_id).or_default().push((option_amount, account_id));
					}
				},
				Err(_err) => {
					errors += 1;
					// let option_id = extrinsic.option_id(&epoch_blocks);
					// let option = crate::OptionIdToOption::<MockRuntime>::get(option_id).unwrap();
					// use crate::types::OptionType;
					// let (asset_id, strike_price) = match option.option_type {
					// 	OptionType::Call => (option.base_asset_id, option.quote_asset_strike_price),
					// 	OptionType::Put => (option.quote_asset_id, option.base_asset_strike_price),
					// };
					// let account_balance = Assets::balance(asset_id, &extrinsic.account_id);
					// dbg!(asset_id, strike_price, extrinsic.option_amount, account_balance);
					// dbg!(_err);
				},
			}
		}
		let exercise_same_block =
			|(b, _id): &(BlockNumber, OptionId)| *b == block_producer.block_number();
		while let Some((_b, option_id)) = pop_if(&mut exercise_blocks, exercise_same_block) {
			for (option_amount, account_id) in buyers.get(&option_id).unwrap_or(&Vec::new()) {
				assert_ok!(TokenizedOptions::exercise_option(
					Origin::signed(*account_id),
					*option_amount,
					option_id,
				));
			}
			sellers.extend(crate::Sellers::<MockRuntime>::iter_key_prefix(option_id));
			for account_id in sellers.drain(..) {
				assert_ok!(TokenizedOptions::withdraw_collateral(
					Origin::signed(account_id),
					option_id
				));
			}
		}
	}
	dbg!(total_amount(account_count));
	dbg!((successful, errors));
}

fn total_amount(account_count: usize) -> [Balance; 5] {
	ASSETS_WITH_USDC.map(|asset_id| {
		(1..=account_count).fold(0, |accumulator, account_id| {
			accumulator + Assets::balance(asset_id, &account_id_from_u64(account_id as u64))
		})
	})
}

fn initialize_options(
	start_block_number: BlockNumber,
	moments: &[Moment],
	option_configs: Vec<OptionConfig<AssetId, Balance, Moment>>,
) -> EpochBlocks {
	let block_number = |p: BlockNumber| {
		let pp = moments.partition_point(|x| *x <= p);
		assert!(pp > 0);
		start_block_number + pp as BlockNumber - 1
	};
	let last_block_number = start_block_number + moments.len() as BlockNumber - 1;
	let capacity = 3 * option_configs.len();
	let mut deposits = Vec::with_capacity(capacity);
	let mut purchases = Vec::with_capacity(capacity);
	let mut exercises = Vec::with_capacity(capacity);
	let mut ends = Vec::with_capacity(capacity);
	for option_config in option_configs {
		let epoch = option_config.epoch;
		let r = <TokenizedOptions as TokenizedOptionsTrait>::create_option(option_config);
		if let Ok(option_id) = r {
			let deposit = block_number(epoch.deposit);
			let purchase = block_number(epoch.purchase);
			let exercise = block_number(epoch.exercise);
			let end = block_number(epoch.end);
			deposits.extend((deposit..purchase).map(|b| (b, option_id)));
			purchases.extend((purchase..exercise).map(|b| (b, option_id)));
			exercises.extend((exercise..end).map(|b| (b, option_id)));
			ends.extend((end..=last_block_number).map(|b| (b, option_id)));
		}
	}
	let mut result = [deposits, purchases, exercises, ends];
	result.iter_mut().for_each(|v| v.sort_by_key(|(b, _id)| *b));
	result
}

fn pop_if<T>(vec: &mut Vec<T>, cond: impl FnOnce(&T) -> bool) -> Option<T> {
	let elem = vec.pop()?;
	if cond(&elem) {
		Some(elem)
	} else {
		vec.push(elem);
		None
	}
}

fn get_exercise_blocks(epoch_blocks: &EpochBlocks) -> Vec<(BlockNumber, OptionId)> {
	let mut exercise_blocks = epoch_blocks[EXERCISE_EPOCH_INDEX].clone();
	exercise_blocks.sort_by_key(|(_b, option_id)| *option_id);
	exercise_blocks.dedup_by_key(|(_b, option_id)| *option_id);
	exercise_blocks.sort_by_key(|(b, _option_id)| Reverse(*b));
	exercise_blocks
}

enum TokenizedOptionsBlocksConfig {}

impl BlocksConfig for TokenizedOptionsBlocksConfig {
	type Runtime = MockRuntime;
	type Hooked = TokenizedOptions;
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TokenizedOptionsExtrinsicType {
	SellOption,
	DeleteSellOption,
	BuyOption,
}

#[derive(Clone, Debug)]
struct TokenizedOptionsExtrinsic {
	extrinsic_type: TokenizedOptionsExtrinsicType,
	index: Index,
	account_id: AccountId,
	option_amount: Balance,
}

impl TokenizedOptionsExtrinsic {
	fn generate(
		extrinsic_type_strategy: impl Strategy<Value = TokenizedOptionsExtrinsicType> + Clone,
		account_id_strategy: impl Strategy<Value = u64> + Clone,
		option_amount_rng: Range<Balance>,
	) -> impl Strategy<Value = Self> + Clone {
		(extrinsic_type_strategy, any::<Index>(), account_id_strategy, option_amount_rng).prop_map(
			move |(extrinsic_type, index, account_id, option_amount)| {
				let account_id = account_id_from_u64(account_id);
				TokenizedOptionsExtrinsic { extrinsic_type, index, account_id, option_amount }
			},
		)
	}
	fn epoch_index(&self) -> usize {
		match self.extrinsic_type {
			TokenizedOptionsExtrinsicType::SellOption
			| TokenizedOptionsExtrinsicType::DeleteSellOption => DEPOSIT_EPOCH_INDEX,
			TokenizedOptionsExtrinsicType::BuyOption => PURCHASE_EPOCH_INDEX,
		}
	}
	fn block_number(&self, epoch_blocks: &EpochBlocks) -> BlockNumber {
		self.index.get(&epoch_blocks[self.epoch_index()]).0
	}
	fn option_id(&self, epoch_blocks: &EpochBlocks) -> OptionId {
		self.index.get(&epoch_blocks[self.epoch_index()]).1
	}
	fn run(&self, epoch_blocks: &EpochBlocks) -> DispatchResult {
		let option_id = self.option_id(epoch_blocks);
		match self.extrinsic_type {
			TokenizedOptionsExtrinsicType::SellOption => TokenizedOptions::sell_option(
				Origin::signed(self.account_id),
				self.option_amount,
				option_id,
			),
			TokenizedOptionsExtrinsicType::DeleteSellOption => {
				TokenizedOptions::delete_sell_option(
					Origin::signed(self.account_id),
					self.option_amount,
					option_id,
				)
			},
			TokenizedOptionsExtrinsicType::BuyOption => TokenizedOptions::buy_option(
				Origin::signed(self.account_id),
				self.option_amount,
				option_id,
			),
		}
	}
}
