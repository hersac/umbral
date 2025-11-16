use std::fs;
use umbral_lexer::analizar;

fn main() {
    let ruta = "codigo-ejemplo/codigo.um";
    let contenido = fs::read_to_string(ruta).expect("No se pudo leer el archivo");
    let tokens = analizar(&contenido);
    println!("{:?}", tokens);
}
