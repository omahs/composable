use super::*;
use frame_benchmarking::{
    benchmarks, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::ensure;
use sp_std::prelude::*;

use crate::mock::currency::sell_identity;
use crate::Pallet;

benchmarks! {
 sort_vector {
    let caller: T::AccountId = whitelisted_caller();
 	let x in 1 .. 10000;
 	let mut m = Vec::<u32>::new();
 	for i in (0..x).rev() {
 		m.push(i);
 	}
 }: {
 	m.sort();
 } verify {
 	ensure!(m[0] == 0, "You forgot to sort!")
 }
}

// ask {
//     let caller: T::AccountId = whitelisted_caller();
//     let sell = sell_identity();
// }: _(
//     RawOrigin::Signed(caller.clone()),
//     sell_identity,
// )
// verify {
//     ()
// }