use ponte::servidor;
use ponte::cliente;
use ponte::{PASTARECEBE,obter_if_unico};

use std::fs;
use std::path::Path;
use std::io::Cursor;
use std::thread;
use std::time::Duration;

//verifica se a pasta está vazia
fn esta_vazia(pasta: &Path) -> bool{
	let mut entries = fs::read_dir(pasta).unwrap();
	entries.next().is_none()// Retorna `true` se não houver arquivos
}

fn remove_all_files(paste: &Path){
	let _ = fs::read_dir(paste).unwrap().for_each(|entry|{
		let entry = entry.unwrap();
		let file_type = entry.file_type().unwrap();
		if file_type.is_file() {
			fs::remove_file(entry.path()).unwrap();
		}else if file_type.is_dir(){
			remove_all_files(entry.path().as_path());
			fs::remove_dir(entry.path()).unwrap();
		}
	});
}

#[test]
fn transfer(){

	//setup servidor
	let recv_paste = Path::new(PASTARECEBE);//&Path
	match recv_paste.try_exists(){
		Ok(true)=>{
			if !esta_vazia(recv_paste){
				//delete all files
			    remove_all_files(recv_paste);
			}
		},
		Ok(false)=>{
			//doesn't exists
			let _ = fs::create_dir_all(recv_paste);
		}
		Err(_)=>{
			panic!("Não foi possível verificar a pasta de recebimento");
		}
	}

	//init servidor
	let thread_servidor = thread::spawn(|| {
		let status_servidor = servidor::iniciar(Cursor::new(obter_if_unico().unwrap().ip.to_string()), false);
		return status_servidor;
	});

	thread::sleep(Duration::from_millis(200));

	//init cliente
	let thread_cliente = thread::spawn(|| {
		let status_cliente = cliente::iniciar(Cursor::new(obter_if_unico().unwrap().ip.to_string()));
		return status_cliente;
	});

	let status_cliente = thread_cliente.join().unwrap();
	if let Err(_) = status_cliente{
		panic!("Erro no cliente");
	}
	let status_servidor = thread_servidor.join().unwrap();
	if let Err(_) = status_servidor{
		panic!("Erro no servidor");
	}
}