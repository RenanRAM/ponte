mod ips;

use ips::{interpretar_ips, completar_ips};
use ponte::{interpretar_ip, completar_ip, nome_corrompido};

fn assert_nome_corrompido(original: &str, esperado: &str) {
	let novo_nome = nome_corrompido(original);
	assert_eq!(novo_nome.as_str(), esperado);
}

#[test]
fn testar_nome_corrompido(){
	assert_nome_corrompido("arquivo","arquivo(corrompido)");
	assert_nome_corrompido("arquivo.txt","arquivo(corrompido).txt");
	assert_nome_corrompido("a.txt","a(corrompido).txt");
	assert_nome_corrompido("arquivo(1).txt","arquivo(1)(corrompido).txt");
	assert_nome_corrompido("arquivo.tx","arquivo(corrompido).tx");
	assert_nome_corrompido("arquivo.t","arquivo(corrompido).t");
	assert_nome_corrompido("arquivo.txt.w","arquivo(corrompido).txt.w");
	assert_nome_corrompido("arquivo.txt.wq","arquivo(corrompido).txt.wq");
	assert_nome_corrompido("arquivo.txt.ggt","arquivo(corrompido).txt.ggt");
	assert_nome_corrompido("arquivo 1.t","arquivo 1(corrompido).t");
	assert_nome_corrompido("arquivo 1.txt.ggt","arquivo 1(corrompido).txt.ggt");
}

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