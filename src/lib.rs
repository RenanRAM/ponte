use std::path::Path;
use std::fs;
use std::process;

mod servidor;
mod cliente;

//porta da comunicação
pub const PORTA: u16 = 7879;
pub const BUFFBYTES: usize = 8192;
pub const PASTAENVIO: &str = "enviar/";
pub const PASTARECEBE: &str = "receber/";

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