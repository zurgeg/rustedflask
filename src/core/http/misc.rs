/// Converts an HTTP version (`(i32, i32)`) to a `Vec<u8>`
///
/// In hindsight, this should've been
/// `impl From<i32, i32> for Vec<u8>`
///
/// Oh well.
pub fn httpver_to_vecu8(httpver: (i32, i32)) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(Vec::<u8>::from(httpver.0.to_string()));
    out.push(b'.');
    out.extend(Vec::<u8>::from(httpver.1.to_string()));
    out
}
