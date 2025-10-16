use std::net::Ipv4Addr;

//completar_ip(semi_ip: u32, ip_local: u32, mask: u32) -> Option<Ipv4Addr>
pub fn get_lista_teste1() -> Vec<(u32, u32, u32, Ipv4Addr)> {
	//semi, ip_local, mask, result
	return vec![
	(51380496u32, 3320771555u32, 4109299939u32, Ipv4Addr::from_bits(3355374067u32)),
	(Ipv4Addr::new(0,0,0,108).to_bits(),Ipv4Addr::new(192,168,0,104).to_bits(),Ipv4Addr::new(255,255,255,0).to_bits(), Ipv4Addr::new(192,168,0,108)),
	(Ipv4Addr::new(0,0,222,0).to_bits(),Ipv4Addr::new(192,168,58,3).to_bits(),Ipv4Addr::new(255,255,0,255).to_bits(), Ipv4Addr::new(192,168,222,3))
	];
}