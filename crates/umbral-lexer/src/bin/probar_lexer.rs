use umbral_lexer::analizar;

fn main() {
    let texto = "a + 1";
    let tokens = analizar(texto);
    println!("{:?}", tokens);
}
