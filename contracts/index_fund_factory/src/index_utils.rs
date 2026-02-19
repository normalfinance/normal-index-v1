use soroban_sdk::{symbol_short, xdr::ToXdr, Bytes, BytesN, Env};

pub(crate) fn get_index_salt(e: &Env, sequence: &u32) -> BytesN<32> {
    let mut salt = Bytes::new(e);

    salt.append(&symbol_short!("0x00").to_xdr(e));
    salt.append(&sequence.to_xdr(e));

    e.crypto().sha256(&salt).to_bytes()
}
