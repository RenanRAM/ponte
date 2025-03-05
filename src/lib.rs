use std::path::Path;
use std::fs;
use std::process;
use std::thread::{self,JoinHandle};
use std::cell::OnceCell;
use std::time::Duration;
use std::sync::{Arc, Mutex};

mod servidor;
mod cliente;

//porta da comunicação
pub const PORTA: u16 = 7879;
pub const BUFFBYTES: usize = 8192;
pub const PASTAENVIO: &str = "enviar/";
pub const PASTARECEBE: &str = "receber/";
const PERIODO_THREAD: u64 = 295;

pub struct BufTerminal{
	thread: OnceCell<JoinHandle<()>>,
	texto_desligar: Arc<Mutex<(String,bool)>>,
}

impl BufTerminal{

	pub fn new()-> Self{
		let t_d = Arc::new(Mutex::new((String::new(), false)));
		let texto_desigar = t_d.clone();
		let thread1 = thread::spawn(move || {
		loop{
				let mut campos = t_d.lock().unwrap();
				print!("{}",campos.0);
				let desligado = campos.1;
				if desligado{
					println!("Desligando...");
					break;
				}
				campos.0.clear();
				drop(campos);
				thread::sleep(Duration::from_millis(PERIODO_THREAD));
			}
		});
		let cell = OnceCell::new();
		cell.set(thread1).unwrap();
		Self{
			thread: cell,
			texto_desligar: texto_desigar,
		}
	}

	fn escreve(& self, texto: &str){
		let mut campos = self.texto_desligar.lock().unwrap();
		campos.0.push_str(texto);
	}
}

//desligar a thread automaticamente no drop
impl Drop for BufTerminal {
	fn drop(&mut self) {
		let mut campos = self.texto_desligar.lock().unwrap();
		campos.1 = true;
		drop(campos);//precisa disso
		let status = self.thread.take().unwrap().join();
		if let Err(_) = status{
			println!("Erro na thread de BufTerminal");
		};
	}
}

//exibe os comandos disponíveis
pub fn comandos_disponiveis(){
	println!("Comandos disponíveis:");
	println!("enviar");
	println!("receber");
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

pub fn inicia_servidor(){
	verificar_pasta_recebe();
	println!("----Modo de recebimento----");
	let status = servidor::iniciar();
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
				println!("Erro, esvazie a pasta receber antes de executar o programa");
				process::exit(0);
			}
		},
		Ok(false)=>{
			if !criar_pasta(pasta_receber){
				println!("Erro, Não foi possível criar a pasta para receber os arquivos");
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
				println!("Erro, não há arquivos para enviar");
				process::exit(0);
			}
		},
		Ok(false)=>{
			println!("Atenção, pasta de envio não existe, criando...");
			if !criar_pasta(pasta_enviar){
				println!("Erro, não foi possível criar a pasta para enviar arquivos");
			}else{
				println!("Adicione arquivos na pasta e execute novamente");
			}
			process::exit(0);
		}
		Err(_)=>{
			panic!("Não foi possível verificar a pasta de envio");
		}
	}
}

//cria uma pasta
fn criar_pasta(pasta: &Path)->bool{
	let status = fs::create_dir_all(pasta);
	if let Err(erro) = status{
		println!("Erro ao criar pasta: {erro}");
		return false;
	}else{
		return true;
	}
}

//verifica se a pasta está vazia
fn esta_vazia(pasta: &Path) -> bool {
	let mut entries = fs::read_dir(pasta).unwrap();
	entries.next().is_none()// Retorna `true` se não houver arquivos
}