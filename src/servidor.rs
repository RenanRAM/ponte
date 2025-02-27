use std::net::{TcpListener, TcpStream,Ipv4Addr};
use std::io::{self,BufReader,BufRead,Read,Write,BufWriter};
use std::fs::{self,File};
use std::str::FromStr;
use std::time::Duration;
use std::path::PathBuf;
//use std::time::Instant;
use std::thread::sleep;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::process;
use std::io::Error;
use std::sync::{Arc, Mutex};
use std::cmp::min;
use crate::{PORTA,BUFFBYTES,PASTARECEBE};

type Arcmb = Arc<Mutex<BufReader<TcpStream>>>;
//struct para nome e tamanho de arquivo
struct Cabecalho{
	nome: String,
	tamanho: u64
}

impl Cabecalho {
	fn obter(reader: Arcmb)-> Option<Self>{
		let mut nome = String::new();
		let mut reader = reader.lock().unwrap();
		let _ = reader.read_line(&mut nome).expect("Erro na leitura do nome");
		if nome.is_empty(){
			println!("O cliente desconectou");
			process::exit(1);
		}
		//eliminando a quebra de linha final
		nome = nome.trim_end().to_string();

		//obtendo o tamanho
		let mut buftamanho = [0u8;8];
		reader.read_exact(&mut buftamanho).unwrap();
		let tamanho: u64 = u64::from_le_bytes(buftamanho);
		if tamanho > 0{
			print!("Nome completo: {nome}");
			println!("  Tamanho: {tamanho}");
			Some(Self{
				nome,
				tamanho
			})
		}else{
			None
		}
	}
}

//inicia o servidor, função a ser chamada no main
pub fn iniciar() -> std::io::Result<()> {

	let mut input = String::new();
	println!("Digite o ip deste computador, ex: 127.0.0.1 para teste no localhost");

	let mut stdin = io::stdin().lock();
	stdin.read_line(&mut input).unwrap();

	let Ok(ipv4) = Ipv4Addr::from_str(input.trim_end())
	else{
		println!("O ip digitado não é válido");
		return Err(Error::other("Erro: ip inválido"));
	};
	let listener = TcpListener::bind((ipv4, PORTA))?;
	println!("Servidor da ponte escutando em {input}");
	println!("Use o ip acima no cliente");
	let (stream , _) = listener.accept().unwrap();
	println!("Cliente conectado!");

	let reader = Arc::new(Mutex::new(BufReader::new(stream)));

	//lendo stream e criando arquivos
	while let Some(cab) = Cabecalho::obter(reader.clone()){
		criar_arquivo(cab, reader.clone());
	}

	sleep(Duration::from_secs(1));
	Ok(())
}

fn criar_arquivo(cab: Cabecalho, reader: Arcmb){
	//agora o nome contem os diretórios
	let mut nome_com_caminho = PathBuf::from_str(PASTARECEBE).unwrap();

	//concatendando com a pasta PASTARECEBE
	nome_com_caminho.push(cab.nome.as_str());

	//criando diretórios caso não existam
	if let Some(path_pai) = nome_com_caminho.parent(){
		fs::create_dir_all(path_pai).expect("Não foi possível criar o diretório!");
	}
	
	//criando o arquivo
	let file = File::create_new(nome_com_caminho).expect("Não foi possível criar um arquivo, abortando!");
	// usando buffwritrer
	let mut file = BufWriter::with_capacity(2*BUFFBYTES,file);

	//criando canal
	let (tx1,rx1) = sync_channel(100);

	//contador do tamanho u64
	let mut tamanho_restante = cab.tamanho;

	//leitor tcp
	//TODO:testar sem criar uma nova thread, modo síncrono
	let thread1 = thread::spawn(move ||{
		let mut buffer8k = [0u8;BUFFBYTES];
		let mut reader = reader.lock().unwrap();
		loop{
			let limite = min(BUFFBYTES as u64, tamanho_restante) as usize;
			if let Ok(n) = reader.read(&mut buffer8k[..limite]){
				tx1.send((buffer8k,n)).expect("Canal fechado antes da hora");
				tamanho_restante -= n as u64;//Ok a contagem de tamanho está exatamente igual
				if tamanho_restante == 0{
					break;
				}
			}else{
				panic!("Erro na leitura");
			}
		}
	});

	//escrevendo no arquivo
	loop{
		if let Ok((buff,n)) = rx1.recv(){
			if n == 0{
				break;//fim da leitura
			}
			file.write_all(&buff[..n]).expect("Erro na escrita");
		}else{
			break;
		}
	}

	thread1.join().expect("Erro no leitor TCP");
}

/* Antigo, único arquivo
pub fn iniciar() -> std::io::Result<()> {

	let mut input = String::new();
	println!("Digite o ip deste computador, ex: 127.0.0.1 para localhost");

	let mut stdin = io::stdin().lock();
	stdin.read_line(&mut input).unwrap();
	input.truncate(input.len()-2);
	let Ok(ipv4) = Ipv4Addr::from_str(input.as_str())
	else{
		println!("O ip digitado não é válido");
		return Err(Error::other("Erro: ip inválido"));
	};
	let listener = TcpListener::bind((ipv4, 7879u16))?;
	println!("Servidor escutando em {input}");
	let (stream , _) = listener.accept().unwrap();
	println!("Cliente conectado!");

	//receber as informações
	let tempo = handle_client(stream);
	println!("Tempo de recebimento {:?}", tempo);
	sleep(Duration::from_secs(3));
	Ok(())
}

fn handle_client(stream: TcpStream)->Duration {
	let mut reader = BufReader::new(stream);
	let mut nome = String::new();
	let _ = reader.read_line(&mut nome).unwrap();
	if nome.is_empty(){
		println!("O cliente desconectou");
		process::exit(1);
	}

	nome.truncate(nome.len()-2);
	println!("Nome do arquivo: {nome}");

	//obtendo o tamanho
	let mut buftamanho = [0u8;8];
	reader.read(&mut buftamanho).unwrap();
	let tamanho: u64 = u64::from_le_bytes(buftamanho);
	println!("Tamanho do arquivo {tamanho}");

	let atual = Instant::now();

	let mut file = File::create_new(nome.as_str()).expect("Não foi possível criar o arquivo");
	let tamanhobuff = mem::size_of::<([u8; 8192],usize)>();
	let (tx1,rx1) = sync_channel(tamanhobuff);

	let mut conta_tamanho = 0u64; 
	//leitor ftp
	thread::spawn(move ||{
		let mut buffer8k = [0u8;8192];
		loop{
			if let Ok(n) = reader.read(&mut buffer8k){
				tx1.send((buffer8k,n)).expect("Canal fechado antes da hora");
				if n==0{
					break;
				}
			}else{
				panic!("Erro na leitura");
			}
		}
	});

	//criador de arquivo
	loop{
		if let Ok((buff,n)) = rx1.recv(){
			conta_tamanho += n as u64;//Ok a contagem de tamanho está exatamente igual
			if n == 0{
				break;//fim da leitura
			}
			file.write_all(&buff[..n]).expect("Erro na escrita");
		}else{
			panic!("Canal fechado devido a erro na leitura");
		}
	}

	println!("Tamanho contado {conta_tamanho}");
	if conta_tamanho == tamanho{
		println!("Tamanho confere");
	}else{
		println!("Tamanho diferente");
	}
	atual.elapsed()
}
*/