use abi::ffi_store::FfiStore;
use microkelvin::Hamt;

struct Map<K, V> {
    wrapping: HAMT<K, V, (), FfiStore>,
}

impl<K, V> Map<K, V> {
    fn new() -> Self {
	Map(
	    wrapping: Hamt::new(),
	}
    }
}
