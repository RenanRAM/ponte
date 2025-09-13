use std::net::{TcpListener, TcpStream,Ipv4Addr};
use std::io::{BufReader,BufRead,Read,Write,BufWriter};
use std::fs::{self,File};
use std::path::{PathBuf,Path};
use std::str::FromStr;
use std::process;
use std::io::Error;

use std::cmp::min;
use crate::{PORTA,BUFFBYTES,PASTARECEBE,FINAL_FLAG,SET_LEN_LIMIT,BufTerminal,pedir_ip};

use get_if_addrs::{IfAddr,get_if_addrs};

type Arcmb = BufReader<TcpStream>;
//struct para nome e tamanho de arquivo
struct Cabecalho{
	nome: String,
	tamanho: u64
}

impl Cabecalho {
	fn obter(reader: &mut Arcmb, buf_terminal: & BufTerminal)-> Option<Self>{
		let mut nome = String::new();
		//let mut reader = reader.lock().unwrap();
		let _ = reader.read_line(&mut nome).expect("Erro na leitura do nome");
		if nome.is_empty(){
			println!("O cliente desconectou");
			process::exit(1);
		}
		//eliminando a quebra de linha final
		nome = nome.trim_end().to_string();

		//obtendo o tamanho
		let mut buftamanho = [0u8;8];//u64
		reader.read_exact(&mut buftamanho).unwrap();
		let tamanho: u64 = u64::from_le_bytes(buftamanho);
		if tamanho == 0 && nome.as_str() == FINAL_FLAG{
			None
		}else{
			buf_terminal.escreve(format!("Nome completo: {nome}  Tamanho: {tamanho}b\r\n").as_str());
			Some(Self{
				nome,
				tamanho
			})
		}
	}
}

//inicia o servidor, função a ser chamada no main
pub fn iniciar(auto_ip: bool) -> std::io::Result<()> {

	let mut ipv4:Option<Ipv4Addr> = None;
	
	//condição de auto ip
	if auto_ip{
		ipv4 = obter_ip_unico();
	}

	//pedir ip caso ainda não tenha nada
	if ipv4.is_none(){
		println!("Digite o ip deste computador, ex: 127.0.0.1 para teste no localhost");
		ipv4 = pedir_ip();
	}

	//não foi obtido qualquer ip, abortar
	if ipv4.is_none(){
		return Err(Error::other("Erro: ip inválido"));
	}
		
	let ipv4 = ipv4.unwrap();
	let listener = TcpListener::bind((ipv4, PORTA))?;
	println!("Servidor da ponte escutando em {ipv4}, use este ip no cliente/enviar");
	let (stream , _) = listener.accept().unwrap();
	println!("Cliente conectado!");

	let mut reader = BufReader::new(stream);

	let buf_terminal = BufTerminal::new();


	//lendo stream e criando arquivos
	while let Some(cab) = Cabecalho::obter(&mut reader, &buf_terminal){
		if !criar_arquivo(cab, &mut reader){
			println!("Ocorreu um erro na transferência!");
			return Err(Error::other("Erro na transferência: envio incompleto"));
		}
	}
	Ok(())
}

fn criar_arquivo(cab: Cabecalho, reader: &mut Arcmb) -> bool{
	//agora o nome contem os diretórios
	let mut nome_com_caminho = PathBuf::from_str(PASTARECEBE).unwrap();

	//concatendando com a pasta PASTARECEBE
	nome_com_caminho.push(cab.nome.as_str());

	//criando diretórios caso não existam
	if let Some(path_pai) = nome_com_caminho.parent(){
		fs::create_dir_all(path_pai).expect("Não foi possível criar o diretório!");
	}
	
	//criando o arquivo
	let file = File::create_new(nome_com_caminho.as_path()).expect("Não foi possível criar um arquivo, abortando!");

	if cab.tamanho <= SET_LEN_LIMIT{
		//alocando o tamanho total do arquivo
		let _ = file.set_len(cab.tamanho);
	}

	// usando buffwritrer
	let mut file = BufWriter::with_capacity(8*BUFFBYTES,file);

	//contador do tamanho u64
	let mut tamanho_restante = cab.tamanho;

	//escrevendo no arquivo
	let mut limite: usize;
	let mut buffer8k = [0u8;BUFFBYTES];
	loop{
		limite = min(BUFFBYTES as u64, tamanho_restante) as usize;
		if let Ok(n) = reader.read(&mut buffer8k[..limite]){
			tamanho_restante -= n as u64;//Ok a contagem de tamanho está exatamente igual
			if let Err(_) = file.write_all(&buffer8k[..n]){
				drop(file);
				println!("Erro na escrita");
				anvancar_corrompido(nome_com_caminho.as_path(), tamanho_restante, reader);
				break;
			};
			if tamanho_restante == 0{
				break;
			}
		}else{
			return false;
		}
	}
	return true;
}

fn anvancar_corrompido(nome_antigo: &Path, mut tam_restante: u64, stream: &mut Arcmb){

	let filename: &str = nome_antigo.file_name().unwrap().to_str().unwrap();
	let novo_nome = nome_corrompido(filename);
	let novo_nome_completo = nome_antigo.with_file_name(&novo_nome);

	let _ = fs::rename(nome_antigo, novo_nome_completo);
	
	//avançar o ponteiro da stream
	let mut limite: usize;
	let mut buffer8k = [0u8;BUFFBYTES];
	loop{
		limite = min(BUFFBYTES as u64, tam_restante) as usize;
		if let Ok(n) = stream.read(&mut buffer8k[..limite]){
			tam_restante -= n as u64;
			if tam_restante == 0{
				break;
			}
		}else{
			panic!("Erro fatal")
		}
	}
}

fn obter_ip_unico() -> Option<Ipv4Addr> {
	let Ok(ifaces) = get_if_addrs()
	else{
		println!("Erro ao percorrer interfaces de ip");
		return None;
	};

	let mut retorno_unico: Option<Ipv4Addr> = None;

	for iface in ifaces{
		if iface.is_loopback() {continue;}
		if let IfAddr::V4(ipv4) = iface.addr{
			if retorno_unico.is_none() {
				retorno_unico = Some(ipv4.ip);
			}else{
				return None;
			}
		}
	}

	retorno_unico
}

fn nome_corrompido(nome: &str) -> String{

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
		assert_eq!(obter_ip_unico(), Some(Ipv4Addr::new(192,168,0,108)));
	}

	#[test]
	fn teste_nome_corrompido0() {
		let novo_nome = nome_corrompido("arquivo");
		assert_eq!(novo_nome.as_str(), "arquivo(corrompido)");
	}

	#[test]
	fn teste_nome_corrompido1() {
		let novo_nome = nome_corrompido("arquivo.txt");
		assert_eq!(novo_nome.as_str(), "arquivo(corrompido).txt");
	}

	#[test]
	fn teste_nome_corrompido2() {
		let novo_nome = nome_corrompido("arquivo.txt.ggt");
		assert_eq!(novo_nome.as_str(), "arquivo(corrompido).txt.ggt");
	}
}