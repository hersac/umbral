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
async fn rest_junta_argumentos_en_lista() {
    let codigo = r#"
        f: contar(...ns) { r: (ns.length); }
        v: n = contar(1, 2, 3);
    "#;
    let valor = ejecutar_y_obtener(codigo, "n").await;
    assert!(matches!(valor, Valor::Entero(3)));
}

#[tokio::test]
async fn rest_vacio_es_lista_vacia() {
    let codigo = r#"
        f: contar(...ns) { r: (ns.length); }
        v: n = contar();
    "#;
    let valor = ejecutar_y_obtener(codigo, "n").await;
    assert!(matches!(valor, Valor::Entero(0)));
}

#[tokio::test]
async fn rest_convive_con_parametros_fijos() {
    let codigo = r#"
        f: suma(a, b, ...resto) { r: (a + b + resto.length); }
        v: n = suma(10, 20, 1, 2, 3);
    "#;
    let valor = ejecutar_y_obtener(codigo, "n").await;
    assert!(matches!(valor, Valor::Entero(33)));
}

#[tokio::test]
async fn rest_tipado_incompatible_avisa_pero_enlaza() {
    let codigo = r#"
        f: contarInt(...ns->Int) { r: (ns.length); }
        v: n = contarInt(1, 'texto');
    "#;
    let valor = ejecutar_y_obtener(codigo, "n").await;
    assert!(matches!(valor, Valor::Entero(2)));
}

#[tokio::test]
async fn spread_en_llamada_expande_hacia_rest() {
    let codigo = r#"
        f: contar(...ns) { r: (ns.length); }
        v: l = {1, 2, 3, 4};
        v: n = contar(&l);
    "#;
    let valor = ejecutar_y_obtener(codigo, "n").await;
    assert!(matches!(valor, Valor::Entero(4)));
}

#[tokio::test]
async fn clase_generica_conserva_valor() {
    let codigo = r#"
        cs: Caja<T> {
            pr: valor->T;
            pu f: Caja(valor->T) { th.valor = valor; }
            pu f: obtener()->T { r: (th.valor); }
        }
        v: c->Caja<Int> = n: Caja(7);
        v: x = c.obtener();
    "#;
    let valor = ejecutar_y_obtener(codigo, "x").await;
    assert!(matches!(valor, Valor::Entero(7)));
}

#[tokio::test]
async fn interfaz_generica_implementada_resuelve_datos() {
    let codigo = r#"
        in: IPrueba<T> {
            pu: data->T;
            f: obtenerData()->T;
        }
        cs: Prueba {
            pr: valor->Int;
            pu f: Prueba(valor->Int) { th.valor = valor; }
        }
        cs: PruebaBase imp: IPrueba<Prueba> {
            pr: data->Prueba;
            pu f: PruebaBase(data->Prueba) { th.data = data; }
            pu f: obtenerData()->Prueba { r: (th.data); }
        }
        v: p = n: Prueba(42);
        v: b = n: PruebaBase(p);
        v: x = b.obtenerData().valor;
    "#;
    let valor = ejecutar_y_obtener(codigo, "x").await;
    assert!(matches!(valor, Valor::Entero(42)));
}

#[tokio::test]
async fn funcion_generica_retorna_mismo_valor() {
    let codigo = r#"
        f: id<T>(x->T)->T { r: (x); }
        v: a = id(99);
    "#;
    let valor = ejecutar_y_obtener(codigo, "a").await;
    assert!(matches!(valor, Valor::Entero(99)));
}
