use soroban_sdk::{symbol_short, xdr::ToXdr, Address, Bytes, BytesN, Env, String};

/* Salt Methodology

Normal Index Funds implement a different salting method for public and private indexes:
- Public: these indexes are salted using the index token symbol exclusively - this is to avoid confusion by 3rd party investors.
- Private: these indexes are salted using both the index token symbol and the admin address - this allows users to create indexes with any symbol for personal use.

*/

pub(crate) fn get_index_salt(
    e: &Env,
    public: bool,
    admin: &Address,
    token_symbol: String,
) -> BytesN<32> {
    let mut salt = Bytes::new(e);

    salt.append(&symbol_short!("0x00").to_xdr(e));

    if public {
        salt.append(&token_symbol.to_xdr(e));
    } else {
        salt.append(&token_symbol.to_xdr(e));
        salt.append(&symbol_short!("0x00").to_xdr(e));
        salt.append(&admin.to_xdr(e));
    }
    salt.append(&symbol_short!("0x00").to_xdr(e));

    e.crypto().sha256(&salt).to_bytes()
}
