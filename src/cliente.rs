use std::io::{self,BufRead,Write,Read,ErrorKind};
use std::net::{TcpStream,Ipv4Addr,SocketAddrV4,SocketAddr};
use std::str::FromStr;
use std::path::Path;
use std::fs::File;
use std::thread;
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::time::{Instant,Duration};
use std::thread::sleep;
use std::process;
use crate::{PORTA,BUFFBYTES,PASTAENVIO,BufTerminal};

type MPSCDATA = ([u8; BUFFBYTES],usize);

//inicia o cliente, função a ser executada no main
pub fn iniciar() -> std::io::Result<()> {

	//vetor para armazenar os não enviados por qualquer erro
	let mut nao_enviados: Vec<String> = Vec::new();

	//input de texto do terminal
	let mut input = String::new();
	println!("Digite o ip do servidor, ex: 127.0.0.1");
	let mut stdin = io::stdin().lock();
	stdin.read_line(&mut input).unwrap();
	
	//interpretando como ipv4, usando trim para cortar quebras de linha e já retornando &str
	let Ok(ipv4) = Ipv4Addr::from_str(input.trim_end())
	else{
		println!("O IP digitado é inválido");
		process::exit(0);
	};

	let socaddrv4 = SocketAddr::from(SocketAddrV4::new(ipv4, PORTA));

	let mut stream = TcpStream::connect_timeout(&socaddrv4, Duration::from_millis(3200))?;
	println!("Conectado ao servidor!");

	//criando o canal:

	//criar o canal MPSC aqui, logo depois criar a thread separada de envio TCP
	let (tx1,rx1) = sync_channel::<MPSCDATA>(100);

	//envio TCP
	let thread1 = thread::spawn(move || {
		loop {
			if let Ok((dados,n)) = rx1.recv(){
				loop{
					//tentando escrever tudo
					match stream.write_all(&dados[..n]){
						Ok(_)=> {break;},
						Err(e)=>{
							match e.kind(){
								//erro recuperável
								ErrorKind::Interrupted=>{
									println!("Escrita interrompida, retomando...");
									sleep(Duration::from_millis(110));
								},
								_=>{
									panic!("Erro fatal no envio {e:?}");
								}
							}
						}
					}
				}
			}else{
				break;
			}
		}
	});

	//criando um BufTerminal para aumentar a velocidade
	let bufterminal = BufTerminal::new();

	//lendo a pasta de envio e enviando os dados para a thread1
	//medindo o tempo que leva
	let atual = Instant::now();
	let status = read_dir_recursive(PathBuf::from_str(PASTAENVIO).unwrap(), &mut nao_enviados, tx1.clone(), &bufterminal);
	drop(bufterminal);

	//sinalizando o fim para o servidor
	let (ultimobuff,tamanhototal) = singlebuff("FIM\r\n",0u64);
	tx1.send((ultimobuff,tamanhototal)).expect("Canal fechado antes do ultimo buffer");
	drop(tx1);//drop de tx1 encerra do loop da thread1
	thread1.join().expect("Erro na thread");

	//print do tempo de envio
	let duracao = atual.elapsed();
	println!("Tempo de envio: {},{}s", duracao.as_secs(), duracao.subsec_millis());
	if !status{
		println!("Ocorreu um erro no envio!");
	}

	//printar os arquivos não enviados se houver
	if !nao_enviados.is_empty(){
		println!("Os seguintes arquivos não puderam ser enviados");
		for arq in nao_enviados {
			println!("{arq}");
		}
	}

	Ok(())
}

//true = ok, false = erro, faz a leitura dos diretórios e envia todos os arquivos
fn read_dir_recursive(dir: PathBuf, vetor_erro: &mut Vec<String>, tx1: SyncSender<MPSCDATA>, bufterminal: & BufTerminal)->bool{//funcionando
	if let Ok(iterador) = dir.read_dir(){
		for entry in iterador{
			let entry = entry.expect("Erro na leitura do iterador de diretórios");
			let caminho_arquivo = entry.path();//PathBuf, é a única opção, se não usaria &Path
			if caminho_arquivo.is_file() {
				if !enviar(caminho_arquivo.as_path(), tx1.clone(), bufterminal){
					//não foi enviado, adicionar ao vetor de não enviados
					vetor_erro.push(String::from(caminho_arquivo.to_str().unwrap()));
				}
			}else if caminho_arquivo.is_dir(){
				if !read_dir_recursive(caminho_arquivo, vetor_erro, tx1.clone(), bufterminal){
					return false;
				}
			}
		}
	}else{
		return false;
	}
	return true;
}

//cria um cabeçalho em um buffer padrão
fn singlebuff(string: &str, tamanho: u64)-> MPSCDATA{
	let mut buff = [0u8;BUFFBYTES];
	let tamanhostring = string.len();
	buff[..tamanhostring].copy_from_slice(string.as_bytes());
	let buftamanho = tamanho.to_le_bytes();//[u8;8]
	let tamanhototal = tamanhostring+buftamanho.len();
	buff[tamanhostring..tamanhototal].copy_from_slice(&buftamanho);
	return (buff,tamanhototal);
}

//true se foi enviado, false se não
fn enviar(caminho: &Path, tx1: SyncSender<MPSCDATA>, bufterminal: & BufTerminal)-> bool{

	//testando a existência do arquivo
	if let Ok(tt) = caminho.try_exists(){
		if !tt{
			//erro tolerável
			return false;
		}
	}else{
		//erro tolerável
		println!("Erro ao verificar um arquivo");
		return false;
	}

	//obtendo uma &str
	let caminho_str = caminho.to_str().unwrap();

	//tentando abrir o arquivo
	let opt = match File::open(caminho){
		Ok(arquivo)=> {
			bufterminal.escreve(format!("Arquivo: {caminho_str} --> OK!").as_str());
			Some(arquivo)
		},
		Err(erro)=>{
			match erro.kind(){
				ErrorKind::PermissionDenied=>{
					println!("Sem permissão para abrir o arquivo: {caminho_str}");
				},
				ErrorKind::NotFound=>{
					println!("Não encontrado: {caminho_str}");
				},
				_=>(),
			}
			//erro tolerável
			return false;
		}
	};

	//extraindo o arquivo
	let mut file = opt.unwrap();

	//precisamos enviar a quebra de linha \r\n junto
	let nome_com_quebra = format!("{}\r\n",caminho_str.strip_prefix(PASTAENVIO).unwrap());//removendo "enviar/"

	//obtendo o tamanho do arquivo
	let metadata = file.metadata();
	if metadata.is_err(){
		bufterminal.escreve("Não foi possível ler o tamanho do arquivo\r\n");
		return false;
	}
	let metadata = metadata.expect("Não foi possível ler o tamanho do arquivo"); // Obtém as metainformações do arquivo
	let tamanho_arquivo: u64 = metadata.len(); // Retorna o tamanho do arquivo

	bufterminal.escreve("  Enviando...\r\n");

	//gerando um buffer de cabeçalho para o TCP
	let primeirobuffdata = singlebuff(nome_com_quebra.as_str(),tamanho_arquivo);
	tx1.send(primeirobuffdata).expect("Canal fechado antes do primeiro buffer");

	loop{
		let mut buffer1 = [0u8;BUFFBYTES];
		match file.read(&mut buffer1){
			Ok(n) =>{
				if n == 0 {break;}//fim do arquivo

				//enviando o buffer e o numero de bytes lidos
				tx1.send((buffer1,n)).expect("Canal fechado antes da hora");
			},
			Err(e)=>{
				//erro, tentando se recuperar
				match e.kind(){
					ErrorKind::Interrupted=>{
						println!("Leitura foi interrompida, retomando...");
						sleep(Duration::from_millis(71));
					},
					_=>{
						drop(file);
						panic!("Erro fatal na leitura");
					}
				}
			}	
		}
	}

	return true;
}














/*antigo
pub fn iniciar() -> std::io::Result<()> {

	let mut input = String::new();
	println!("Digite o ip do servidor, ex: 127.0.0.1");
	let mut stdin = io::stdin().lock();
	stdin.read_line(&mut input).unwrap();
	input.truncate(input.len()-2);

	let ipv4 = Ipv4Addr::from_str(input.as_str()).expect("O ip digitado é inválido");
	let socaddrv4 = SocketAddr::from(SocketAddrV4::new(ipv4, 7879u16));

	let mut stream = TcpStream::connect_timeout(&socaddrv4, Duration::from_millis(3200))?;
	println!("Conectado ao servidor!");
	input.clear();//limpando a string


	println!("Digite o nome do arquivo com a extenção");
	stdin.read_line(&mut input).unwrap();
	enviar(&mut stream, input.as_str());
	drop(stream);
	sleep(Duration::from_secs(2));
	Ok(())
}

fn enviar(stream: &mut TcpStream, nome_com_quebra: &str){
	//lembrando que "nome" ainda tem a quebra de linha no final
	//testando a existência do arquivo
	let nome_sem_quebra = unsafe{nome_com_quebra.get_unchecked(..(nome_com_quebra.len()-2))};	
	if let Ok(tt) = Path::new(nome_sem_quebra).try_exists(){
		if !tt{
			panic!("Arquivo não existe");
		}
	}else{
		panic!("Erro ao verificar o arquivo");
	}

	//tentando abrir o arquivo
	let opt = match File::open(nome_sem_quebra){
		Ok(arquivo)=> {
			println!("Arquivo: {nome_sem_quebra} --> OK!");
			Some(arquivo)
		},
		Err(erro)=>{
			match erro.kind(){
				ErrorKind::PermissionDenied=>{
					println!("Sem permissão para abrir o arquivo");
				},
				ErrorKind::NotFound=>{
					println!("Não encontrado");
				},
				_=>(),
			}
			process::exit(1);
		}
	};
	//extraindo o arquivo
	let mut file = opt.unwrap();

	println!("Enviando arquivo...");

	//nome ainda tem a quebra de linha no final, precisamos enviar a quebra junto
	stream.write_all(nome_com_quebra.as_bytes()).unwrap();

	//enviando o tamanho do arquivo
	let metadata = file.metadata().expect("Não foi possível ler o tamanho do arquivo"); // Obtém as metainformações do arquivo
	let tamanho_arquivo: u64 = metadata.len(); // Retorna o tamanho do arquivo
	let buftamanho = tamanho_arquivo.to_le_bytes();//[u8;8]
	stream.write_all(&buftamanho).unwrap();
	//enviando agora
	stream.flush().unwrap();

	//criando o canal:
	//definindo o tamanho exato do buffer do canal para não sobrecarregar
	let tamanhobuff = mem::size_of::<([u8; BUFFBYTES],usize)>();

	//medindo o tempo que leva
	let atual = Instant::now();

	//criando o canal com buffer interno igual ao buffer1
	let (tx1,rx1) = sync_channel(tamanhobuff);

	//executando o envio
	thread::spawn(move || {
		loop{
			let mut buffer1 = [0u8;BUFFBYTES];
			match file.read(&mut buffer1){
				Ok(n) =>{
					if n == 0 {break;}//fim do arquivo

					//enviando o buffer o o numero de bytes lidos
					tx1.send((buffer1,n)).expect("Canal fechado antes da hora");
				},
				Err(e)=>{
					//erro, tentando se recuperar
					match e.kind(){
						ErrorKind::Interrupted=>{
							println!("Leitura foi interrompida, retomando...");
							sleep(Duration::from_millis(71));
						},
						_=>{
							drop(file);
							panic!("Erro fatal na leitura");
						}
					}
				}	
			}
		}
	});

	//
	loop {
		if let Ok((dados,n)) = rx1.recv(){
			loop{
				//tentando escrever tudo
				match stream.write_all(&dados[..n]){
					Ok(_)=> {break;},
					Err(e)=>{
						match e.kind(){
							//erro recuperável
							ErrorKind::Interrupted=>{
								sleep(Duration::from_millis(64));
							},
							_=>{
								panic!("Erro fatal no envio {e:?}");
							}
						}
					}
				}
			}
		}else{
			//enviando agora
			stream.flush().unwrap();
			break;
		}
	}


	println!("Tempo de envio {:?}", atual.elapsed());
}
*/