use std::{env,process};
use ponte::*;

//ok: Cliente: Separar a thread de envio stream TCP da função envio recursivo, criar ela antes do loop
//ok: criar um buffer de String para printar no terminal, uma thread separada limpa printa o buffer no terminal a cada 1 segundo
//ok: testar a escrita de arquivos do servidor sem criar uma nova thread, modo síncrono
//ok: Tratar o erro com um arquivo específico sem parar todo programa, apenas não enviar aquele arquivo se for o caso e exibir todos os arquivos não enviados no final
//ok: fazer o envio de todas pastas e subpastas de modo recursivo
//ok: melhorar a flag de final de envio, auguns arquivos reais já tem tamanho 0 e estão causando a finalização do envio de forma incorreta!!!!!!
fn main() {

	//extraindo argumentos
	let mut argumentos = env::args();
	argumentos.next();
	let modo = argumentos.next().unwrap_or_else(||{
		println!("{}: v{}",env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
		comandos_disponiveis();
		process::exit(0);
	});

	let auto_ip:bool = match argumentos.next(){
		Some(valor) => valor.as_str() == "--auto",
		None => false
	};

	//selecionando modo
	match modo.as_str(){
		"receber"=>{
			inicia_servidor(auto_ip);
		},
		"enviar"=>{
			inicia_cliente();
		},
		_=>{
			println!("Comando desconhecido");
			comandos_disponiveis();
		}
	}
}