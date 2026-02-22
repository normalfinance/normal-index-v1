use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum AdapterRegistryError {
    AlreadyInitialized = 1,
    AdapterNameNotFound = 2,
    AdapterAddressNotFound = 3,
    AdapterAddressAlreadyAssigned = 4,
}
