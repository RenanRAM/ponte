mod ips;

use ips::{interpretar_ips, completar_ips};
use ponte::{interpretar_ip, completar_ip};

#[test]
fn lista_ips_teste(){
	let lista: Vec<(&'static str, u32)> = interpretar_ips::get_lista_teste1();
	for (txt, num) in lista{
		let opt = interpretar_ip(txt);
		assert_eq!(opt.unwrap(), num);
	}
}

#[test]
fn lista_ips_falha() {
	let lista_falha: Vec<&'static str> = interpretar_ips::get_lista_falha1();
	for txt in lista_falha{
		let opt = interpretar_ip(txt);
		assert_eq!(opt.is_none(), true);
	}
}

#[test]
fn lista_completar_ip(){
	let lista = completar_ips::get_lista_teste1();
	for (semi, ip_local, mask, ipv4) in lista{
		let opt = completar_ip(semi, ip_local, mask);
		assert_eq!(opt.unwrap(), ipv4);
	}
}

