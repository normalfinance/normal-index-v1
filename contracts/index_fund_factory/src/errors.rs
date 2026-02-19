use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum IndexFundFactoryError {
    IndexAlreadyExists = 401,
    IndexNotFound = 404,
    AlreadyInitialized = 405,
}
