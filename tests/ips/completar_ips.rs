use std::net::Ipv4Addr;

//completar_ip(semi_ip: u32, ip_local: u32, mask: u32) -> Option<Ipv4Addr>
pub fn get_lista_teste1() -> Vec<(u32, u32, u32, Ipv4Addr)> {
	//semi, ip_local, mask, result
	return vec![
	(51380496u32, 3320771555u32 ,4109299939u32, Ipv4Addr::from_bits(3355374067u32))
	];
}