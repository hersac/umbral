use umbral_lexer::analizar;
use umbral_parser::Parser;
use umbral_runtime::Runtime;
use umbral_runtime::runtime::valores::Valor;

async fn ejecutar_y_obtener(codigo: &str, variable: &str) -> Valor {
    let tokens = analizar(codigo);
    let mut parser = Parser::nuevo(tokens);
    let programa = parser.parsear_programa().unwrap();

    let mut runtime = Runtime::nuevo();
    runtime.ejecutar(programa).await;
    runtime
        .interpretador
        .entorno_actual
        .obtener(variable)
        .unwrap_or(Valor::Nulo)
}

#[tokio::test]
async fn variable_sin_tipo_puede_reasignarse_con_otro_tipo() {
    let codigo = r#"
        v: nombre = 'Heriberto';
        nombre = 32;
    "#;
    let valor = ejecutar_y_obtener(codigo, "nombre").await;
    assert!(matches!(valor, Valor::Entero(32)));
}

#[tokio::test]
async fn variable_tipada_rechaza_reasignacion_incompatible() {
    let codigo = r#"
        v: nombre->Str = 'Heriberto';
        nombre = 32;
    "#;
    let valor = ejecutar_y_obtener(codigo, "nombre").await;
    assert!(matches!(valor, Valor::Texto(s) if s == "Heriberto"));
}

#[tokio::test]
async fn variable_tipada_acepta_reasignacion_compatible() {
    let codigo = r#"
        v: nombre->Str = 'Heriberto';
        nombre = 'Ana';
    "#;
    let valor = ejecutar_y_obtener(codigo, "nombre").await;
    assert!(matches!(valor, Valor::Texto(s) if s == "Ana"));
}

#[tokio::test]
async fn variable_tipada_rechaza_valor_inicial_incompatible() {
    let codigo = r#"
        v: numero->Int = 'texto';
    "#;
    let valor = ejecutar_y_obtener(codigo, "numero").await;
    assert!(matches!(valor, Valor::Nulo));
}

#[tokio::test]
async fn int_es_compatible_con_float() {
    let codigo = r#"
        v: precio->Flo = 10;
    "#;
    let valor = ejecutar_y_obtener(codigo, "precio").await;
    assert!(matches!(valor, Valor::Entero(10)));
}