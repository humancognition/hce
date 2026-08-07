use hce_core::{Hce, HceCase, HceError, HceMode, LanguageLevel, RecoveryResult, TsGranularity};
use hce_fpe::{build_cipher, CipherKind};
use rustler::{Binary, Encoder, Env, Error, NifResult, OwnedBinary, ResourceArc, Term};
use std::sync::Mutex;

struct HceRef { inner: Mutex<Hce> }

#[rustler::resource_impl]
impl rustler::Resource for HceRef {}

fn make_hce(key: &[u8], level: u8, mode: u8) -> Result<Hce, Error> {
    let l = LanguageLevel::from_byte(level).ok_or(Error::BadArg)?;
    let m = HceMode::from_byte(mode).ok_or(Error::BadArg)?;
    let k = if m == HceMode::Plain && key.is_empty() { None }
            else if key.is_empty() { return Err(Error::BadArg) }
            else { Some(key) };
    Ok(Hce::new(k, l, m))
}

#[rustler::nif]
fn hce_new(key: Binary, level: u8, mode: u8) -> NifResult<ResourceArc<HceRef>> {
    let h = make_hce(key.as_slice(), level, mode)?;
    Ok(ResourceArc::new(HceRef { inner: Mutex::new(h) }))
}

#[rustler::nif]
fn hce_encode(r: ResourceArc<HceRef>, data: Binary) -> NifResult<String> {
    Ok(r.inner.lock().map_err(|_| Error::BadArg)?.encode(data.as_slice()))
}

#[rustler::nif]
fn hce_decode<'a>(env: Env<'a>, r: ResourceArc<HceRef>, input: String) -> NifResult<Term<'a>> {
    let lock = r.inner.lock().map_err(|_| Error::BadArg)?;
    match lock.decode(&input) {
        Ok(bytes) => {
            let mut owned = OwnedBinary::new(bytes.len()).ok_or(Error::BadArg)?;
            owned.as_mut_slice().copy_from_slice(&bytes);
            Ok((atoms::ok(), owned.release(env)).encode(env))
        }
        Err(_) => Ok((atoms::error(), "decode failed").encode(env)),
    }
}

#[rustler::nif]
fn hce_recover<'a>(env: Env<'a>, r: ResourceArc<HceRef>, input: String) -> NifResult<Term<'a>> {
    let lock = r.inner.lock().map_err(|_| Error::BadArg)?;
    let result = match lock.recover(&input) {
        Ok(RecoveryResult::Ok) | Ok(RecoveryResult::Corrected(_)) => lock.decode(&input),
        _ => Err(HceError::IntegrityFailure),
    };
    drop(lock);
    match result {
        Ok(bytes) => {
            let mut owned = OwnedBinary::new(bytes.len()).ok_or(Error::BadArg)?;
            owned.as_mut_slice().copy_from_slice(&bytes);
            Ok((atoms::ok(), owned.release(env)).encode(env))
        }
        Err(_) => Ok((atoms::error(), "reject").encode(env)),
    }
}

#[rustler::nif]
fn hce_destroy<'a>(env: Env<'a>, _r: ResourceArc<HceRef>) -> NifResult<Term<'a>> {
    Ok(atoms::ok().encode(env))
}

#[rustler::nif] fn hce_with_bit_width(r: ResourceArc<HceRef>, v: u32) -> NifResult<()> { let mut h = r.inner.lock().map_err(|_| Error::BadArg)?; *h = h.clone().with_bit_width(v); Ok(()) }
#[rustler::nif] fn hce_with_domain_modulus(r: ResourceArc<HceRef>, hi: u64, lo: u64) -> NifResult<()> { let m = ((hi as u128) << 64) | (lo as u128); let mut h = r.inner.lock().map_err(|_| Error::BadArg)?; *h = h.clone().with_modulus(m); Ok(()) }
#[rustler::nif] fn hce_with_cipher_kind(r: ResourceArc<HceRef>, kind: u8, key: Binary, _key_len: usize) -> NifResult<()> { let ck = match kind { 0 => CipherKind::Feistel8, _ => CipherKind::Shuffle4 }; let mut h = r.inner.lock().map_err(|_| Error::BadArg)?; let ks = if key.is_empty() { None } else { Some(key.as_slice()) }; *h = h.clone().with_cipher(build_cipher(ck, ks, h.domain())); Ok(()) }
#[rustler::nif] fn hce_with_case(r: ResourceArc<HceRef>, v: u8) -> NifResult<()> { let mut h = r.inner.lock().map_err(|_| Error::BadArg)?; *h = h.clone().with_case(if v == 0 { HceCase::Lower } else { HceCase::Upper }); Ok(()) }
#[rustler::nif] fn hce_with_check_syllables(r: ResourceArc<HceRef>, n: usize) -> NifResult<()> { let mut h = r.inner.lock().map_err(|_| Error::BadArg)?; *h = h.clone().with_check_syllables(n); Ok(()) }
#[rustler::nif] fn hce_with_separator(r: ResourceArc<HceRef>, sep: u8) -> NifResult<()> { let mut h = r.inner.lock().map_err(|_| Error::BadArg)?; *h = h.clone().with_chunk_spec(hce_core::ChunkSpec::natural_with_separator(sep as char)); Ok(()) }
#[rustler::nif] fn hce_with_chunk_none(r: ResourceArc<HceRef>) -> NifResult<()> { let mut h = r.inner.lock().map_err(|_| Error::BadArg)?; *h = h.clone().with_chunk_spec(hce_core::ChunkSpec::none()); Ok(()) }
#[rustler::nif] fn hce_with_chunk_fixed(r: ResourceArc<HceRef>, n: usize) -> NifResult<()> { let mut h = r.inner.lock().map_err(|_| Error::BadArg)?; *h = h.clone().with_chunk_spec(hce_core::ChunkSpec::fixed(n)); Ok(()) }
#[rustler::nif] fn hce_with_chunk_pattern(r: ResourceArc<HceRef>, p: Vec<usize>) -> NifResult<()> { let mut h = r.inner.lock().map_err(|_| Error::BadArg)?; *h = h.clone().with_chunk_spec(hce_core::ChunkSpec::pattern(&p)); Ok(()) }
#[rustler::nif] fn hce_with_timestamp_config(r: ResourceArc<HceRef>, epoch: i64, gran: u8) -> NifResult<()> { let g = match gran { 0=>TsGranularity::Second,1=>TsGranularity::Minute,2=>TsGranularity::Hour,3=>TsGranularity::Day,4=>TsGranularity::Week,5=>TsGranularity::Month,_=>return Err(Error::BadArg) }; let mut h = r.inner.lock().map_err(|_| Error::BadArg)?; *h = h.clone().with_timestamp_config(epoch, g); Ok(()) }

mod atoms { rustler::atoms! { ok, error } }

rustler::init!("Elixir.Hce.NIF", [hce_new,hce_encode,hce_decode,hce_recover,hce_destroy,hce_with_bit_width,hce_with_domain_modulus,hce_with_cipher_kind,hce_with_case,hce_with_check_syllables,hce_with_separator,hce_with_chunk_none,hce_with_chunk_fixed,hce_with_chunk_pattern,hce_with_timestamp_config]);
