use std::net::{TcpListener, TcpStream,Ipv4Addr,SocketAddr};
use std::io::{BufReader,BufRead,Read,Write,BufWriter};
use std::fs::{self,File};
use std::path::{PathBuf,Path};
use std::str::FromStr;
use std::process;
use std::io::Error;
use std::mem;
use std::cmp::min;
use std::time::Duration;
use std::thread;
use crate::{PORTA,BUFFBYTES,PASTARECEBE,FINAL_FLAG,SET_LEN_LIMIT,pedir_ip,obter_if_unico,nome_corrompido};
use crate::buf_terminal::BufTerminal;

use get_if_addrs::Ifv4Addr;

type Arcmb = BufReader<TcpStream>;
//struct para nome e tamanho de arquivo
struct Cabecalho{
	nome: String,
	tamanho: u64
}

impl Cabecalho {
	fn obter(reader: &mut Arcmb)-> Option<Self>{
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
		let mut buftamanho = [0u8;mem::size_of::<u64>()];//u64
		
		reader.read_exact(&mut buftamanho).unwrap();
		let tamanho: u64 = u64::from_le_bytes(buftamanho);
		if tamanho == 0 && nome.as_str() == FINAL_FLAG{
			None
		}else{
			Some(Self{
				nome,
				tamanho
			})
		}
	}
}

//inicia o servidor, função a ser chamada no main
pub fn iniciar<R : Read + BufRead>(mut stdin: R, auto_ip: bool) -> std::io::Result<()> {

	let ipv4:Option<Ipv4Addr>;
	let mut ifv4:Option<Ifv4Addr> = None;
	//condição de auto ip
	if auto_ip{
		ifv4 = obter_if_unico();
	}

	//pedir ip caso ainda não tenha nada

	if ifv4.is_none(){
		println!("Digite o ip deste computador, ex: 127.0.0.1 para teste no localhost");
		ipv4 = pedir_ip(&mut stdin);
	}else{
		ipv4 = Some(ifv4.unwrap().ip);
	}

	//não foi obtido qualquer ip, abortar
	if ipv4.is_none(){
		return Err(Error::other("Erro: ip inválido"));
	}
		
	let ipv4 = ipv4.unwrap();
	let listener = TcpListener::bind((ipv4, PORTA))?;
	println!("Servidor da ponte escutando em {ipv4}, use este ip no cliente/enviar");
	let (stream , soc) = listener.accept().unwrap();

	print!("Cliente conectado! ");
	match soc{
		SocketAddr::V4(socv4) =>{
			println!("ip(v4): {}", socv4.ip());
		},
		SocketAddr::V6(socv6) =>{
			println!("ip(v6): {}", socv6.ip());
		},
	}
	thread::sleep(Duration::from_millis(900));

	let mut reader = BufReader::new(stream);

	let buf_terminal = BufTerminal::new();


	//lendo stream e criando arquivos
	while let Some(cab) = Cabecalho::obter(&mut reader){

		buf_terminal.escreve(format!("{}  Tamanho: {}b\r\n", cab.nome, cab.tamanho).as_str());
		if !criar_arquivo(cab, &mut reader){
			println!("Ocorreu um erro na transferência!");
			return Err(Error::other("Erro na transferência: envio incompleto"));
		}
	}
	Ok(())
}

//retorna false em caso de erro fatal
fn criar_arquivo(cab: Cabecalho, reader: &mut Arcmb) -> bool{
	//agora o nome contem os diretórios
	let mut nome_com_caminho = PathBuf::from_str(PASTARECEBE).unwrap();

	//concatendando com a pasta PASTARECEBE
	nome_com_caminho.push(cab.nome.as_str());

	//criando diretórios caso não existam
	if let Some(path_pai) = nome_com_caminho.parent(){
		if let Err(_) = fs::create_dir_all(path_pai){
			println!("Não foi possível criar um diretório");
			return false;
		}
	}
	
	//criando o arquivo
	let Ok(file) = File::create_new(nome_com_caminho.as_path())
	else{
		println!("Não foi possível criar um arquivo, abortando!");
		return false;
	};

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

	//TODO: fazer um print caso não consiga renomear
	if let Err(_) = fs::rename(nome_antigo, novo_nome_completo){
		println!("Não foi possível retomar a transferência após erro na escrita do arquivo {}", filename);
		process::exit(1);
	};
	
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