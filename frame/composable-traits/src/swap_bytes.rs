pub trait SwapBytes: Sized {
	fn swap_bytes(self) -> Self;
}

impl SwapBytes for u64 {
	fn swap_bytes(self) -> u64 {
		u64::swap_bytes(self)
	}
}
