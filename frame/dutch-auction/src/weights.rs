use frame_support::dispatch::Weight;

pub trait WeightInfo {
	fn ask() -> Weight;
	fn take() -> Weight;
	fn liquidate() -> Weight;
	fn known_overhead_for_on_finalize() -> Weight;
}

impl WeightInfo for () {
	fn ask() -> Weight {
		1
	}

	fn take() -> Weight {
		2
	}

	fn liquidate() -> Weight {
		2
	}

	fn known_overhead_for_on_finalize() -> Weight {
		1
	}
}
