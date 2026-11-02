//período de escrita no terminal em milisegundos
const PERIODO_THREAD: u64 = 295;

use std::thread::{self,JoinHandle};
use std::cell::OnceCell;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::process;

pub struct BufTerminal{
	thread: OnceCell<JoinHandle<()>>,
	texto_desligar: Arc<Mutex<(String,bool)>>,
}

impl BufTerminal{

	pub fn new()-> Self{
		let t_d = Arc::new(Mutex::new((String::new(), false)));
		let texto_desligar = t_d.clone();
		let thread1 = Self::set_thread(t_d);
		let cell = OnceCell::new();
		cell.set(thread1).unwrap();
		Self{
			thread: cell,
			texto_desligar: texto_desligar,
		}
	}

	pub fn escreve(& self, texto: &str){
		let mut campos = self.texto_desligar.lock().unwrap();
		campos.0.push_str(texto);
	}

	fn set_thread(t_d: Arc<Mutex<(String, bool)>>) -> JoinHandle<()>{

		return thread::spawn(move || {
			loop{
				let mut campos = t_d.lock().unwrap_or_else(|_| {
					println!("Erro na execução, poison");
					process::exit(1);
				});
				print!("{}",campos.0);
				if campos.1{//true para desligar
					println!("Desligando...");
					break;
				}
				campos.0.clear();
				drop(campos);
				thread::sleep(Duration::from_millis(PERIODO_THREAD));
			}
		});
	}
}

//desligar a thread automaticamente no drop
impl Drop for BufTerminal {
	fn drop(&mut self) {
		let mut campos = self.texto_desligar.lock().unwrap();
		campos.1 = true;
		drop(campos);//precisa disso para liberar o lock
		let status = self.thread.take().unwrap().join();
		if let Err(_) = status{
			println!("Erro na thread de BufTerminal");
		};
	}
}