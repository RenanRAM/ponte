//Constantes
//porta da comunicação
const PORTA: u16 = 7879;
//tamanho do buffer de leitura ou escrita
const BUFFBYTES: usize = 8192;
//nome da pasta do cliente
const PASTAENVIO: &str = "enviar/";
//nome da pasta do servidor
const PASTARECEBE: &str = "receber/";
//TTL para o cliente
const TTL_CLIENT: u32 = 6;
//limite de set_len
const SET_LEN_LIMIT: u64 = 50000000;

//flag final de transmissão, a alteração dessa constante nunca deve ser feita em uma mesma versão, se alterar é preciso criar outra versão no git, pois os programas serão incompatíveis
const FINAL_FLAG: &str = "fimflag....";

use std::path::Path;
use std::fs;
use std::process;
use std::net::Ipv4Addr;
use std::io::{self,BufRead};
use std::str::FromStr;

use get_if_addrs::{IfAddr,Ifv4Addr,get_if_addrs};

mod servidor;
mod cliente;
mod buf_terminal;
//o comportamento de pub em funções é diferente, funções sem pub não são acessíveis fora do arquivo
// enquanto struc sem pub ainda pode ser acessada fora do arquivo e em qualquer lugar dentro da crate (src/) do projeto

//exibe os comandos disponíveis
pub fn comandos_disponiveis(){
	println!("Comandos disponíveis:");
	println!("enviar");
	println!("receber [--auto]");
}

//funções de início
pub fn inicia_cliente(){
	verificar_pasta_envia();
	println!("----Modo de envio----");
	let status = cliente::iniciar();
	match status{
		Ok(_)=>{
			println!("Envio finalizado com sucesso");
		},
		Err(e)=>{
			println!("Erro ao iniciar: {e}");
		}
	}
}

pub fn inicia_servidor(auto_ip: bool){
	verificar_pasta_recebe();
	println!("----Modo de recebimento----");
	let status = servidor::iniciar(auto_ip);
	match status{
		Ok(_)=>{
			println!("Recebimento finalizado com sucesso");
		},
		Err(e)=>{
			println!("Erro ao receber: {e}");
		}
	}
}

//verificadores de pasta
fn verificar_pasta_recebe(){
	let pasta_receber = Path::new(PASTARECEBE);//isso cria um &Path
	match pasta_receber.try_exists(){
		Ok(true)=>{
			if !esta_vazia(pasta_receber){
				println!("Erro, esvazie a pasta {PASTARECEBE} antes de executar o programa");
				process::exit(0);
			}
		},
		Ok(false)=>{
			if !criar_pasta(pasta_receber){
				println!("Erro, Não foi possível criar a pasta ({PASTARECEBE}) para receber os arquivos");
				process::exit(0);
			}
		}
		Err(_)=>{
			panic!("Não foi possível verificar a pasta de recebimento");
		}
	}
}

fn verificar_pasta_envia(){
	let pasta_enviar = Path::new(PASTAENVIO);
	match pasta_enviar.try_exists(){
		Ok(true)=>{
			if esta_vazia(pasta_enviar){
				println!("Não há arquivos para enviar");
				process::exit(0);
			}
		},
		Ok(false)=>{
			println!("Atenção, pasta de envio não existe, criando...");
			if !criar_pasta(pasta_enviar){
				println!("Erro, não foi possível criar a pasta ({PASTAENVIO}) para enviar arquivos");
			}else{
				println!("Adicione arquivos na pasta {PASTAENVIO} e execute novamente");
			}
			process::exit(0);
		}
		Err(_)=>{
			panic!("Não foi possível verificar a pasta de envio");
		}
	}
}

//cria uma pasta
fn criar_pasta(pasta: &Path) -> bool{
	let status = fs::create_dir_all(pasta);
	if let Err(erro) = status{
		println!("Erro ao criar pasta: {erro}");
		return false;
	}else{
		return true;
	}
}

//verifica se a pasta está vazia
fn esta_vazia(pasta: &Path) -> bool{
	let mut entries = fs::read_dir(pasta).unwrap();
	entries.next().is_none()// Retorna `true` se não houver arquivos
}

fn pedir_ip() -> Option<Ipv4Addr> {
	let mut input = String::new();
	let mut stdin = io::stdin().lock();
	stdin.read_line(&mut input).unwrap();
	let input_trim = input.trim_end();
	let Ok(ipv4) = Ipv4Addr::from_str(input_trim)
	else{
		//interpretando entrada
		let opt_ipv4 = interpretar_ip(input_trim);
		if opt_ipv4.is_none(){
			return None;
		}
		let semi_ipv4_u32 = opt_ipv4.unwrap();

		//obter ip deste pc
		let interface = obter_if_unico();
		if interface.is_none(){
			return None;
		}
		let iface = interface.unwrap();

		//tentando completar
		let opt_ipv4_completo = completar_ip(semi_ipv4_u32, iface.ip.to_bits(), iface.netmask.to_bits());

		if opt_ipv4_completo.is_none(){
			println!("O ip digitado não é válido");
		}

		return opt_ipv4_completo;
	};
	Some(ipv4)
}

pub fn interpretar_ip(entrada: &str) -> Option<u32>{
	let pontos = entrada.matches('.').count();
	if pontos > 2{
		return None;
	}
	let separado = entrada.split('.');
	//tentar converter para u8
	let ipv4 = separado.map(|e| e.parse::<u8>())
	.rev()//invertendo pois a entrada deve ser lida de tras para frente
	.enumerate()//obtendo indexes
	.try_fold(0u32, |acc, (i, item)| {//se falhar: ipv4 = None
		if item.is_err(){
			return None;
		}
		if i > 2{// apenas 0, 1 e 2
			return None;
		}
		let s = 8 * i;
		let result: u32 = acc | ((item.unwrap() as u32) << s);
		Some(result)
	});

	return ipv4;
}

fn obter_if_unico() -> Option<Ifv4Addr> {
	let Ok(ifaces) = get_if_addrs()
	else{
		println!("Erro ao percorrer interfaces de ip");
		return None;
	};

	let mut retorno_unico: Option<Ifv4Addr> = None;

	for iface in ifaces{
		if iface.is_loopback() {continue;}
		if let IfAddr::V4(ifv4) = iface.addr{
			if retorno_unico.is_none() {
				retorno_unico = Some(ifv4);
			}else{
				return None;
			}
		}
	}

	retorno_unico
}

pub fn completar_ip(semi_ip: u32, ip_local: u32, mask: u32) -> Option<Ipv4Addr> {

	if (semi_ip & !mask) != semi_ip{
		//semi ip não pertence à rede
		return None;
	}
	let ip_rede: u32 = ip_local & mask;
	let ip_completo: u32 = semi_ip | ip_rede;
	return Some(Ipv4Addr::from_bits(ip_completo));
}

pub fn nome_corrompido(nome: &str) -> String{

	let mut novo_nome = String::new();
	let (nome_split, ext) = if nome.contains("."){
		//separa exatamente o nome antes do . a extenão com . antes
		nome.split_at(nome.find(".").unwrap())
	}else{
		(nome, "")
	};
	novo_nome.push_str(nome_split);
	novo_nome.push_str("(corrompido)");
	novo_nome.push_str(ext);
	return novo_nome;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn teste_de_ip() {
		//coloque o ip do computador antes de testar
		let ifv4 = obter_if_unico().unwrap();
		assert_eq!(ifv4.ip, Ipv4Addr::new(192,168,0,108));
	}

	#[test]
	fn teste_completar_ip_pc(){
		//caso hipotético
		let semi_ip = Ipv4Addr::new(0, 0, 0, 108).to_bits();
		let ifv4 = obter_if_unico().unwrap();
		let mask = ifv4.netmask.to_bits();
		let ip_local = ifv4.ip.to_bits();
		let ip_completo = completar_ip(semi_ip, ip_local, mask);
		assert_eq!(ip_completo, Some(Ipv4Addr::new(192,168,0,108)));
	}
}