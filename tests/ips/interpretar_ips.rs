use std::net::Ipv4Addr;

pub fn get_lista_teste1() -> Vec<(&'static str, u32)>{
	return vec![
	("123.111", Ipv4Addr::new(0, 0, 123, 111).to_bits()),
	("212.123.111", Ipv4Addr::new(0, 212, 123, 111).to_bits()),
	("111", Ipv4Addr::new(0, 0, 0, 111).to_bits()),
	("7", Ipv4Addr::new(0, 0, 0, 7).to_bits()),
	("6.7", Ipv4Addr::new(0, 0, 6, 7).to_bits())];
}

//usat para testar se a interpretação resulta em none em todos estes casos
pub fn get_lista_falha1() -> Vec<&'static str> {
	return vec![
	"256.7", "g.25.7", "2.5.7.1", "43.12.54.11.6", ".2.6.", "...", "78."
	];
}