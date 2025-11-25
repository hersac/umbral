use crate::runtime::clases::{Clase, GestorClases};
use crate::runtime::entorno::Entorno;
use crate::runtime::funciones::GestorFunciones;
use crate::runtime::valores::{Funcion, Valor};
use std::collections::HashMap;
use std::path::PathBuf;
use umbral_parser::ast::*;

pub struct Interpretador {
    pub entorno_actual: Entorno,
    pub gestor_clases: GestorClases,
    pub gestor_funciones: GestorFunciones,
    pub valor_retorno: Option<Valor>,
    pub exportaciones: HashMap<String, bool>,
    pub directorio_base: PathBuf,
}

impl Interpretador {
    pub fn nuevo() -> Self {
        Self {
            entorno_actual: Entorno::nuevo(None),
            gestor_clases: GestorClases::nuevo(),
            gestor_funciones: GestorFunciones::nuevo(),
            valor_retorno: None,
            exportaciones: HashMap::new(),
            directorio_base: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    pub fn establecer_directorio_base(&mut self, ruta: PathBuf) {
        self.directorio_base = ruta;
    }

    pub fn ejecutar_sentencia(&mut self, sentencia: Sentencia) -> Option<Valor> {
        if self.tiene_valor_retorno() {
            return self.valor_retorno.clone();
        }

        match sentencia {
            Sentencia::DeclaracionVariable(decl) => self.ejecutar_declaracion_variable(decl),
            Sentencia::DeclaracionConstante(decl) => self.ejecutar_declaracion_constante(decl),
            Sentencia::Asignacion(asig) => self.ejecutar_asignacion(asig),
            Sentencia::LlamadoTPrint(lt) => self.ejecutar_tprint(lt),
            Sentencia::Return(expr) => self.ejecutar_return(expr),
            Sentencia::If(if_stmt) => self.ejecutar_if(if_stmt),
            Sentencia::Switch(switch) => self.ejecutar_switch(switch),
            Sentencia::For(for_loop) => self.ejecutar_for(for_loop),
            Sentencia::ForEach(foreach) => self.ejecutar_foreach(foreach),
            Sentencia::While(while_loop) => self.ejecutar_while(while_loop),
            Sentencia::DoWhile(do_while) => self.ejecutar_do_while(do_while),
            Sentencia::Funcion(func) => self.registrar_funcion(func),
            Sentencia::Clase(clase) => self.registrar_clase(clase),
            Sentencia::LlamadoFuncion(llamado) => Some(self.evaluar_llamado_funcion(&llamado)),
            Sentencia::Importacion(imp) => self.ejecutar_importacion(imp),
            Sentencia::Expresion(expr) => {
                self.evaluar_expresion(expr);
                None
            }
            _ => None,
        }
    }

    fn tiene_valor_retorno(&self) -> bool {
        self.valor_retorno.is_some()
    }

    fn ejecutar_declaracion_variable(&mut self, decl: DeclaracionVariable) -> Option<Valor> {
        let valor = self.evaluar_expresion(decl.valor);

        if self.entorno_actual.existe(&decl.nombre) {
            eprintln!(
                "Advertencia: La variable '{}' ya existe en este ámbito o superior (shadowing).",
                decl.nombre
            );
        }

        self.entorno_actual
            .definir_variable(decl.nombre.clone(), valor);
        if decl.exportado {
            self.exportaciones.insert(decl.nombre, true);
        }
        None
    }

    fn ejecutar_declaracion_constante(&mut self, decl: DeclaracionConstante) -> Option<Valor> {
        let valor = self.evaluar_expresion(decl.valor);
        self.entorno_actual
            .definir_constante(decl.nombre.clone(), valor);
        if decl.exportado {
            self.exportaciones.insert(decl.nombre, true);
        }
        None
    }

    fn ejecutar_asignacion(&mut self, asig: Asignacion) -> Option<Valor> {
        let valor = self.evaluar_expresion(asig.valor);

        match asig.objetivo {
            umbral_parser::ast::ObjetivoAsignacion::Variable(nombre) => {
                if !self.entorno_actual.asignar(&nombre, valor.clone()) {
                    eprintln!(
                        "Error: Variable '{}' no definida. Use 'v:' para declarar.",
                        nombre
                    );
                }
            }
            umbral_parser::ast::ObjetivoAsignacion::Propiedad { objeto, propiedad } => {
                self.asignar_propiedad_objeto(*objeto, propiedad, valor);
            }
        }

        None
    }

    fn asignar_propiedad_objeto(
        &mut self,
        objeto_expr: Expresion,
        propiedad: String,
        valor: Valor,
    ) {
        let obj_valor = self.evaluar_expresion(objeto_expr.clone());

        match obj_valor {
            Valor::Objeto(mut instancia) => {
                instancia
                    .propiedades
                    .insert(propiedad.clone(), valor.clone());
                if matches!(objeto_expr, Expresion::This) {
                    if !self
                        .entorno_actual
                        .asignar("__this__", Valor::Objeto(instancia.clone()))
                    {
                        self.entorno_actual
                            .definir_variable("__this__".to_string(), Valor::Objeto(instancia));
                    }
                }
            }
            _ => {
                eprintln!("No se puede asignar propiedad a un valor que no es objeto");
            }
        }
    }

    fn ejecutar_tprint(&mut self, lt: LlamadoTPrint) -> Option<Valor> {
        let valor = self.evaluar_expresion(lt.valor);
        self.tprint(valor);
        None
    }

    fn ejecutar_return(&mut self, expr: Expresion) -> Option<Valor> {
        let valor = self.evaluar_expresion(expr);
        self.valor_retorno = Some(valor.clone());
        Some(valor)
    }

    fn registrar_funcion(&mut self, func: DeclaracionFuncion) -> Option<Valor> {
        let parametros: Vec<String> = func.parametros.iter().map(|p| p.nombre.clone()).collect();
        let funcion = Funcion::nueva(func.nombre.clone(), parametros, func.cuerpo);
        self.entorno_actual
            .definir_variable(func.nombre.clone(), Valor::Funcion(funcion));
        if func.exportado {
            self.exportaciones.insert(func.nombre, true);
        }
        None
    }

    fn registrar_clase(&mut self, clase: DeclaracionClase) -> Option<Valor> {
        let clase_obj = Clase::desde_declaracion(&clase);
        let nombre_clase = clase_obj.nombre.clone();
        self.gestor_clases.registrar_clase(clase_obj);
        if clase.exportado {
            self.exportaciones.insert(nombre_clase, true);
        }
        None
    }

    fn ejecutar_importacion(&mut self, imp: umbral_parser::ast::Importacion) -> Option<Valor> {
        use std::fs;

        let ruta_relativa = PathBuf::from(&imp.ruta);
        let ruta_archivo = self.directorio_base.join(&ruta_relativa);

        let contenido = match fs::read_to_string(&ruta_archivo) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error al leer archivo '{}': {}", ruta_archivo.display(), e);
                return None;
            }
        };

        let tokens = umbral_lexer::analizar(&contenido);
        let programa = match umbral_parser::parsear_programa(tokens) {
            Ok(p) => p,
            Err(e) => {
                eprintln!(
                    "Error al parsear archivo '{}': {:?}",
                    ruta_archivo.display(),
                    e
                );
                return None;
            }
        };

        let mut interprete_modulo = Interpretador::nuevo();
        if let Some(parent) = ruta_archivo.parent() {
            interprete_modulo.establecer_directorio_base(parent.to_path_buf());
        }

        for sentencia in programa.sentencias {
            interprete_modulo.ejecutar_sentencia(sentencia);
        }

        for item in imp.items {
            self.procesar_item_importacion(item, &interprete_modulo);
        }

        None
    }

    fn procesar_item_importacion(
        &mut self,
        item: umbral_parser::ast::ItemImportacion,
        modulo: &Interpretador,
    ) {
        use umbral_parser::ast::ItemImportacion;

        match item {
            ItemImportacion::Todo(alias) => {
                let alias_nombre = alias.unwrap_or_else(|| "mod".to_string());

                for (nombre, valor) in &modulo.entorno_actual.variables {
                    if modulo.exportaciones.get(nombre).copied().unwrap_or(false) {
                        let nombre_final = format!("{}_{}", alias_nombre, nombre);
                        self.entorno_actual
                            .definir_variable(nombre_final, valor.clone());
                    }
                }

                for (nombre, clase) in &modulo.gestor_clases.clases {
                    if modulo.exportaciones.get(nombre).copied().unwrap_or(false) {
                        let nombre_final = format!("{}_{}", alias_nombre, nombre);
                        self.gestor_clases
                            .clases
                            .insert(nombre_final, clase.clone());
                    }
                }
            }
            ItemImportacion::Nombre(nombre, alias) => {
                let nombre_final = alias.unwrap_or_else(|| nombre.clone());

                if !modulo.exportaciones.get(&nombre).copied().unwrap_or(false) {
                    eprintln!("Advertencia: '{}' no está exportado en el módulo", nombre);
                    return;
                }

                if let Some(valor) = modulo.entorno_actual.obtener(&nombre) {
                    self.entorno_actual.definir_variable(nombre_final, valor);
                    return;
                }

                if let Some(clase) = modulo.gestor_clases.clases.get(&nombre) {
                    self.gestor_clases
                        .clases
                        .insert(nombre_final, clase.clone());
                    return;
                }

                eprintln!("Advertencia: '{}' no encontrado en el módulo", nombre);
            }
            ItemImportacion::ListaNombres(items) => {
                for sub_item in items {
                    self.procesar_item_importacion(sub_item, modulo);
                }
            }
        }
    }

    pub fn evaluar_expresion(&mut self, expr: Expresion) -> Valor {
        match expr {
            Expresion::LiteralEntero(i) => Valor::Entero(i),
            Expresion::LiteralFloat(f) => Valor::Flotante(f),
            Expresion::LiteralBool(b) => Valor::Booleano(b),
            Expresion::LiteralCadena(s) => self.interpolar_cadena(&s),
            Expresion::LiteralCadenaLiteral(s) => Valor::Texto(s),
            Expresion::LiteralNulo => Valor::Nulo,
            Expresion::Identificador(nombre) => {
                self.entorno_actual.obtener(&nombre).unwrap_or_else(|| {
                    eprintln!("Variable '{}' no encontrada", nombre);
                    Valor::Nulo
                })
            }
            Expresion::Binaria {
                izquierda,
                operador,
                derecha,
            } => self.evaluar_binaria(*izquierda, &operador, *derecha),
            Expresion::Unaria {
                operador,
                expresion,
            } => self.evaluar_unaria(&operador, *expresion),
            Expresion::Agrupada(expr) => self.evaluar_expresion(*expr),
            Expresion::This => {
                // Buscar la instancia actual en el entorno
                self.entorno_actual.obtener("__this__").unwrap_or_else(|| {
                    eprintln!("'th' solo puede usarse dentro de métodos o constructores de clase");
                    Valor::Nulo
                })
            }
            Expresion::Spread(expr) => self.evaluar_expresion(*expr),
            Expresion::Array(items) => {
                let mut valores: Vec<Valor> = Vec::new();
                for item in items {
                    match item {
                        Expresion::Spread(expr) => {
                            let valor = self.evaluar_expresion(*expr);
                            if let Valor::Lista(elementos) = valor {
                                valores.extend(elementos);
                            } else {
                                valores.push(valor);
                            }
                        }
                        _ => {
                            valores.push(self.evaluar_expresion(item));
                        }
                    }
                }
                Valor::Lista(valores)
            }
            Expresion::Objeto(pares) => {
                let mut mapa = HashMap::new();
                for (clave, valor_expr) in pares {
                    let valor = self.evaluar_expresion(valor_expr);
                    mapa.insert(clave, valor);
                }
                Valor::Diccionario(mapa)
            }
            Expresion::Instanciacion { tipo, argumentos } => {
                self.evaluar_instanciacion(&tipo, argumentos)
            }
            Expresion::AccesoPropiedad { objeto, propiedad } => {
                self.evaluar_acceso_propiedad(*objeto, &propiedad)
            }
            Expresion::AccesoIndice { objeto, indice } => {
                self.evaluar_acceso_indice(*objeto, *indice)
            }
            Expresion::LlamadoMetodo {
                objeto,
                metodo,
                argumentos,
            } => self.evaluar_llamado_metodo(*objeto, &metodo, argumentos),
            Expresion::LlamadoFuncion { nombre, argumentos } => {
                let args: Vec<Valor> = argumentos
                    .iter()
                    .map(|arg| self.evaluar_expresion(arg.clone()))
                    .collect();

                if self.es_funcion_builtin(&nombre) {
                    self.ejecutar_funcion_builtin(&nombre, args)
                } else {
                    self.ejecutar_funcion_usuario(&nombre, args)
                }
            }
        }
    }

    fn evaluar_binaria(&mut self, izq: Expresion, op: &str, der: Expresion) -> Valor {
        let izquierda = self.evaluar_expresion(izq);
        let derecha = self.evaluar_expresion(der);

        match op {
            "+" => self.sumar(izquierda, derecha),
            "-" => self.restar(izquierda, derecha),
            "*" => self.multiplicar(izquierda, derecha),
            "/" => self.dividir(izquierda, derecha),
            "%" => self.modulo(izquierda, derecha),
            "==" => Valor::Booleano(self.son_iguales(&izquierda, &derecha)),
            "!=" => Valor::Booleano(!self.son_iguales(&izquierda, &derecha)),
            "<" => self.comparar_menor(izquierda, derecha),
            ">" => self.comparar_mayor(izquierda, derecha),
            "<=" => self.comparar_menor_igual(izquierda, derecha),
            ">=" => self.comparar_mayor_igual(izquierda, derecha),
            "&&" => Valor::Booleano(izquierda.es_verdadero() && derecha.es_verdadero()),
            "||" => Valor::Booleano(izquierda.es_verdadero() || derecha.es_verdadero()),
            _ => {
                eprintln!("Operador binario desconocido: {}", op);
                Valor::Nulo
            }
        }
    }

    fn evaluar_unaria(&mut self, op: &str, expr: Expresion) -> Valor {
        match op {
            "!" => self.evaluar_negacion(expr),
            "-" => self.evaluar_negativo(expr),
            "++" => self.evaluar_incremento(expr),
            "--" => self.evaluar_decremento(expr),
            _ => self.evaluar_expresion(expr),
        }
    }

    fn evaluar_negacion(&mut self, expr: Expresion) -> Valor {
        let valor = self.evaluar_expresion(expr);
        Valor::Booleano(!valor.es_verdadero())
    }

    fn evaluar_negativo(&mut self, expr: Expresion) -> Valor {
        let valor = self.evaluar_expresion(expr);
        match valor {
            Valor::Entero(i) => Valor::Entero(-i),
            Valor::Flotante(f) => Valor::Flotante(-f),
            _ => Valor::Nulo,
        }
    }

    fn evaluar_incremento(&mut self, expr: Expresion) -> Valor {
        let Expresion::Identificador(nombre) = expr else {
            return Valor::Nulo;
        };

        let Some(valor) = self.entorno_actual.obtener(&nombre) else {
            return Valor::Nulo;
        };

        let nuevo_valor = self.incrementar_valor(valor);
        self.entorno_actual.asignar(&nombre, nuevo_valor.clone());
        nuevo_valor
    }

    fn evaluar_decremento(&mut self, expr: Expresion) -> Valor {
        let Expresion::Identificador(nombre) = expr else {
            return Valor::Nulo;
        };

        let Some(valor) = self.entorno_actual.obtener(&nombre) else {
            return Valor::Nulo;
        };

        let nuevo_valor = self.decrementar_valor(valor);
        self.entorno_actual.asignar(&nombre, nuevo_valor.clone());
        nuevo_valor
    }

    fn incrementar_valor(&self, valor: Valor) -> Valor {
        match valor {
            Valor::Entero(i) => Valor::Entero(i + 1),
            Valor::Flotante(f) => Valor::Flotante(f + 1.0),
            _ => valor,
        }
    }

    fn decrementar_valor(&self, valor: Valor) -> Valor {
        match valor {
            Valor::Entero(i) => Valor::Entero(i - 1),
            Valor::Flotante(f) => Valor::Flotante(f - 1.0),
            _ => valor,
        }
    }

    fn sumar(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Texto(a), Valor::Texto(b)) => Valor::Texto(format!("{}{}", a, b)),
            (Valor::Lista(mut a), Valor::Lista(b)) => {
                a.extend(b);
                Valor::Lista(a)
            }
            (a, b) => self.operar_numeros(a, b, |x, y| x + y, |x, y| x + y),
        }
    }

    fn restar(&self, izq: Valor, der: Valor) -> Valor {
        self.operar_numeros(izq, der, |x, y| x - y, |x, y| x - y)
    }

    fn multiplicar(&self, izq: Valor, der: Valor) -> Valor {
        self.operar_numeros(izq, der, |x, y| x * y, |x, y| x * y)
    }

    fn dividir(&self, izq: Valor, der: Valor) -> Valor {
        if self.es_division_por_cero(&der) {
            eprintln!("División por cero");
            return Valor::Nulo;
        }
        self.operar_numeros(izq, der, |x, y| x / y, |x, y| x / y)
    }

    fn modulo(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) if b != 0 => Valor::Entero(a % b),
            _ => Valor::Nulo,
        }
    }

    fn operar_numeros<F, G>(&self, izq: Valor, der: Valor, op_int: F, op_float: G) -> Valor
    where
        F: Fn(i64, i64) -> i64,
        G: Fn(f64, f64) -> f64,
    {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) => Valor::Entero(op_int(a, b)),
            (Valor::Flotante(a), Valor::Flotante(b)) => Valor::Flotante(op_float(a, b)),
            (Valor::Entero(a), Valor::Flotante(b)) => Valor::Flotante(op_float(a as f64, b)),
            (Valor::Flotante(a), Valor::Entero(b)) => Valor::Flotante(op_float(a, b as f64)),
            _ => Valor::Nulo,
        }
    }

    fn es_division_por_cero(&self, valor: &Valor) -> bool {
        match valor {
            Valor::Entero(0) => true,
            Valor::Flotante(f) if *f == 0.0 => true,
            _ => false,
        }
    }

    fn son_iguales(&self, a: &Valor, b: &Valor) -> bool {
        match (a, b) {
            (Valor::Entero(x), Valor::Entero(y)) => x == y,
            (Valor::Flotante(x), Valor::Flotante(y)) => (x - y).abs() < f64::EPSILON,
            (Valor::Entero(x), Valor::Flotante(y)) => (*x as f64 - y).abs() < f64::EPSILON,
            (Valor::Flotante(x), Valor::Entero(y)) => (x - *y as f64).abs() < f64::EPSILON,
            (Valor::Booleano(x), Valor::Booleano(y)) => x == y,
            (Valor::Texto(x), Valor::Texto(y)) => x == y,
            (Valor::Nulo, Valor::Nulo) => true,
            _ => false,
        }
    }

    fn comparar_menor(&self, izq: Valor, der: Valor) -> Valor {
        self.comparar_numeros(izq, der, |a, b| a < b)
    }

    fn comparar_mayor(&self, izq: Valor, der: Valor) -> Valor {
        self.comparar_numeros(izq, der, |a, b| a > b)
    }

    fn comparar_menor_igual(&self, izq: Valor, der: Valor) -> Valor {
        self.comparar_numeros(izq, der, |a, b| a <= b)
    }

    fn comparar_mayor_igual(&self, izq: Valor, der: Valor) -> Valor {
        self.comparar_numeros(izq, der, |a, b| a >= b)
    }

    fn comparar_numeros<F>(&self, izq: Valor, der: Valor, comparador: F) -> Valor
    where
        F: Fn(f64, f64) -> bool,
    {
        let resultado = match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) => comparador(a as f64, b as f64),
            (Valor::Flotante(a), Valor::Flotante(b)) => comparador(a, b),
            (Valor::Entero(a), Valor::Flotante(b)) => comparador(a as f64, b),
            (Valor::Flotante(a), Valor::Entero(b)) => comparador(a, b as f64),
            _ => false,
        };
        Valor::Booleano(resultado)
    }

    fn ejecutar_if(&mut self, if_stmt: If) -> Option<Valor> {
        let condicion = self.evaluar_expresion(if_stmt.condicion);

        if condicion.es_verdadero() {
            return self.ejecutar_bloque(if_stmt.bloque_entonces);
        }

        for else_if in if_stmt.else_ifs {
            let cond = self.evaluar_expresion(else_if.condicion);
            if cond.es_verdadero() {
                return self.ejecutar_bloque(else_if.bloque);
            }
        }

        if let Some(bloque_else) = if_stmt.bloque_else {
            return self.ejecutar_bloque(bloque_else);
        }

        None
    }

    fn ejecutar_bloque(&mut self, bloque: Vec<Sentencia>) -> Option<Valor> {
        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));

        let mut resultado = None;
        for sentencia in bloque {
            if let Some(valor) = self.ejecutar_sentencia(sentencia) {
                resultado = Some(valor);
                break;
            }
        }

        if let Some(parent) = self.entorno_actual.parent.take() {
            self.entorno_actual = *parent;
        }

        resultado
    }

    fn ejecutar_switch(&mut self, switch: Switch) -> Option<Valor> {
        let valor_switch = self.evaluar_expresion(switch.expresion);

        for caso in switch.casos {
            let valor_caso = self.evaluar_expresion(caso.valor);
            if self.son_iguales(&valor_switch, &valor_caso) {
                return self.ejecutar_bloque(caso.bloque);
            }
        }

        switch
            .default
            .and_then(|bloque| self.ejecutar_bloque(bloque))
    }

    fn ejecutar_for(&mut self, for_loop: For) -> Option<Valor> {
        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));

        self.ejecutar_sentencia(*for_loop.inicializacion);

        loop {
            let condicion = self.evaluar_expresion(for_loop.condicion.clone());
            if !condicion.es_verdadero() {
                break;
            }

            if let Some(valor) = self.ejecutar_bloque(for_loop.bloque.clone()) {
                if let Some(parent) = self.entorno_actual.parent.take() {
                    self.entorno_actual = *parent;
                }
                return Some(valor);
            }

            self.evaluar_expresion(for_loop.incremento.clone());
        }

        if let Some(parent) = self.entorno_actual.parent.take() {
            self.entorno_actual = *parent;
        }
        None
    }

    fn ejecutar_foreach(&mut self, foreach: ForEach) -> Option<Valor> {
        let Valor::Lista(items) = self.evaluar_expresion(foreach.iterable) else {
            return None;
        };

        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));

        for item in items {
            self.entorno_actual
                .definir_variable(foreach.variable.clone(), item);

            if let Some(valor) = self.ejecutar_bloque(foreach.bloque.clone()) {
                if let Some(parent) = self.entorno_actual.parent.take() {
                    self.entorno_actual = *parent;
                }
                return Some(valor);
            }
        }

        if let Some(parent) = self.entorno_actual.parent.take() {
            self.entorno_actual = *parent;
        }
        None
    }

    fn ejecutar_while(&mut self, while_loop: While) -> Option<Valor> {
        loop {
            let condicion = self.evaluar_expresion(while_loop.condicion.clone());
            if !condicion.es_verdadero() {
                break;
            }

            if let Some(valor) = self.ejecutar_bloque(while_loop.bloque.clone()) {
                return Some(valor);
            }
        }

        None
    }

    fn ejecutar_do_while(&mut self, do_while: DoWhile) -> Option<Valor> {
        loop {
            if let Some(valor) = self.ejecutar_bloque(do_while.bloque.clone()) {
                return Some(valor);
            }

            let condicion = self.evaluar_expresion(do_while.condicion.clone());
            if !condicion.es_verdadero() {
                break;
            }
        }

        None
    }

    fn evaluar_llamado_funcion(&mut self, llamado: &LlamadoFuncion) -> Valor {
        let argumentos: Vec<Valor> = llamado
            .argumentos
            .iter()
            .map(|arg| self.evaluar_expresion(arg.clone()))
            .collect();

        if self.es_funcion_builtin(&llamado.nombre) {
            return self.ejecutar_funcion_builtin(&llamado.nombre, argumentos);
        }

        self.ejecutar_funcion_usuario(&llamado.nombre, argumentos)
    }

    fn es_funcion_builtin(&self, nombre: &str) -> bool {
        nombre == "tprint"
    }

    fn ejecutar_funcion_builtin(&mut self, nombre: &str, argumentos: Vec<Valor>) -> Valor {
        match nombre {
            "tprint" => {
                for arg in argumentos {
                    self.tprint(arg);
                }
                Valor::Nulo
            }
            _ => Valor::Nulo,
        }
    }

    fn ejecutar_funcion_usuario(&mut self, nombre: &str, argumentos: Vec<Valor>) -> Valor {
        let Some(Valor::Funcion(func)) = self.entorno_actual.obtener(nombre) else {
            eprintln!("Función '{}' no encontrada", nombre);
            return Valor::Nulo;
        };

        self.valor_retorno = None;
        let resultado = GestorFunciones::ejecutar_funcion(&func, argumentos, self);
        self.valor_retorno = None;
        resultado
    }

    fn evaluar_instanciacion(&mut self, tipo: &str, argumentos: Vec<Expresion>) -> Valor {
        let args = self.evaluar_argumentos(argumentos);

        if self.es_funcion_builtin_instanciacion(tipo) {
            return self.ejecutar_builtin_instanciacion(tipo, args);
        }

        if let Some(resultado) = self.intentar_ejecutar_como_funcion(tipo, args.clone()) {
            return resultado;
        }

        self.crear_y_inicializar_instancia(tipo, args)
    }

    fn evaluar_argumentos(&mut self, argumentos: Vec<Expresion>) -> Vec<Valor> {
        argumentos
            .into_iter()
            .map(|arg| self.evaluar_expresion(arg))
            .collect()
    }

    fn es_funcion_builtin_instanciacion(&self, tipo: &str) -> bool {
        tipo == "tprint"
    }

    fn ejecutar_builtin_instanciacion(&mut self, tipo: &str, args: Vec<Valor>) -> Valor {
        match tipo {
            "tprint" => {
                args.into_iter().for_each(|arg| self.tprint(arg));
                Valor::Nulo
            }
            _ => Valor::Nulo,
        }
    }

    fn intentar_ejecutar_como_funcion(&mut self, tipo: &str, args: Vec<Valor>) -> Option<Valor> {
        let Valor::Funcion(func) = self.entorno_actual.obtener(tipo)? else {
            return None;
        };

        self.valor_retorno = None;
        let resultado = GestorFunciones::ejecutar_funcion(&func, args, self);
        self.valor_retorno = None;
        Some(resultado)
    }

    fn crear_y_inicializar_instancia(&mut self, tipo: &str, args: Vec<Valor>) -> Valor {
        let Some(mut instancia) = self.gestor_clases.crear_instancia(tipo) else {
            eprintln!("Clase '{}' no encontrada", tipo);
            return Valor::Nulo;
        };

        self.ejecutar_constructor_si_existe(tipo, &args, &mut instancia);
        Valor::Objeto(instancia)
    }

    fn ejecutar_constructor_si_existe(
        &mut self,
        tipo: &str,
        args: &[Valor],
        instancia: &mut crate::runtime::valores::Instancia,
    ) {
        let Some(constructor) = self.obtener_constructor(tipo) else {
            return;
        };

        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));

        // Vincular 'th' (__this__) con la instancia actual
        self.entorno_actual
            .definir_variable("__this__".to_string(), Valor::Objeto(instancia.clone()));

        self.vincular_parametros_constructor(&constructor.parametros, args);
        self.ejecutar_cuerpo_constructor(constructor.cuerpo, instancia);

        if let Some(parent) = self.entorno_actual.parent.take() {
            self.entorno_actual = *parent;
        }
    }

    fn obtener_constructor(&self, tipo: &str) -> Option<umbral_parser::ast::Metodo> {
        self.gestor_clases
            .obtener_clase(tipo)
            .and_then(|c| c.constructor.clone())
    }

    fn vincular_parametros_constructor(
        &mut self,
        parametros: &[umbral_parser::ast::Parametro],
        args: &[Valor],
    ) {
        for (i, param) in parametros.iter().enumerate() {
            if let Some(valor) = args.get(i) {
                self.entorno_actual
                    .definir_variable(param.nombre.clone(), valor.clone());
            }
        }
    }

    fn ejecutar_cuerpo_constructor(
        &mut self,
        cuerpo: Vec<umbral_parser::ast::Sentencia>,
        instancia: &mut crate::runtime::valores::Instancia,
    ) {
        for sentencia in cuerpo {
            self.ejecutar_sentencia(sentencia);
        }

        // Actualizar la instancia con los cambios realizados en el constructor
        if let Some(Valor::Objeto(inst_actualizada)) = self.entorno_actual.obtener("__this__") {
            *instancia = inst_actualizada;
        }
    }

    fn evaluar_acceso_propiedad(&mut self, objeto: Expresion, propiedad: &str) -> Valor {
        let obj_valor = self.evaluar_expresion(objeto.clone());

        match obj_valor {
            Valor::Objeto(ref instancia) => self.acceder_propiedad_objeto(instancia, propiedad),
            Valor::Diccionario(mapa) => self.acceder_clave_diccionario(mapa, propiedad),
            Valor::Lista(ref items) if propiedad == "length" => Valor::Entero(items.len() as i64),
            _ => self.error_acceso_propiedad_invalido(propiedad, &obj_valor),
        }
    }

    fn acceder_propiedad_objeto(
        &mut self,
        instancia: &crate::runtime::valores::Instancia,
        propiedad: &str,
    ) -> Valor {
        if let Some(valor) = instancia.propiedades.get(propiedad) {
            return valor.clone();
        }

        self.buscar_metodo_como_funcion(instancia, propiedad)
            .unwrap_or_else(|| self.error_propiedad_no_encontrada(propiedad))
    }

    fn buscar_metodo_como_funcion(
        &self,
        instancia: &crate::runtime::valores::Instancia,
        propiedad: &str,
    ) -> Option<Valor> {
        let clase = self.gestor_clases.obtener_clase(&instancia.clase)?;
        let metodo = clase.obtener_metodo(propiedad)?;

        let parametros: Vec<String> = metodo.parametros.iter().map(|p| p.nombre.clone()).collect();

        let funcion = Funcion::nueva(propiedad.to_string(), parametros, metodo.cuerpo.clone());

        Some(Valor::Funcion(funcion))
    }

    fn acceder_clave_diccionario(&self, mapa: HashMap<String, Valor>, propiedad: &str) -> Valor {
        mapa.get(propiedad).cloned().unwrap_or_else(|| {
            eprintln!("Clave '{}' no encontrada", propiedad);
            Valor::Nulo
        })
    }

    fn error_propiedad_no_encontrada(&self, propiedad: &str) -> Valor {
        eprintln!("Propiedad '{}' no encontrada", propiedad);
        Valor::Nulo
    }

    fn error_acceso_propiedad_invalido(&self, propiedad: &str, valor: &Valor) -> Valor {
        eprintln!(
            "No se puede acceder a la propiedad '{}' de {:?}",
            propiedad, valor
        );
        Valor::Nulo
    }

    fn evaluar_acceso_indice(&mut self, objeto: Expresion, indice: Expresion) -> Valor {
        let obj_valor = self.evaluar_expresion(objeto);
        let indice_valor = self.evaluar_expresion(indice);

        match (obj_valor, indice_valor) {
            (Valor::Lista(items), Valor::Entero(i)) => self.acceder_elemento_lista(items, i),
            _ => Valor::Nulo,
        }
    }

    fn acceder_elemento_lista(&self, items: Vec<Valor>, indice: i64) -> Valor {
        if !self.es_indice_valido(indice, items.len()) {
            eprintln!("Índice fuera de rango: {}", indice);
            return Valor::Nulo;
        }

        items[indice as usize].clone()
    }
    fn es_indice_valido(&self, indice: i64, longitud: usize) -> bool {
        indice >= 0 && (indice as usize) < longitud
    }

    fn evaluar_llamado_metodo(
        &mut self,
        objeto: Expresion,
        metodo: &str,
        argumentos: Vec<Expresion>,
    ) -> Valor {
        let obj_valor = self.evaluar_expresion(objeto);

        let instancia = match obj_valor {
            Valor::Objeto(inst) => inst,
            _ => {
                eprintln!(
                    "No se puede llamar método '{}' en un valor que no es objeto",
                    metodo
                );
                return Valor::Nulo;
            }
        };

        let clase = match self.gestor_clases.obtener_clase(&instancia.clase) {
            Some(c) => c,
            None => {
                eprintln!("Clase '{}' no encontrada", instancia.clase);
                return Valor::Nulo;
            }
        };

        let metodo_def = match clase.obtener_metodo(metodo) {
            Some(m) => m.clone(),
            None => {
                eprintln!(
                    "Método '{}' no encontrado en clase '{}'",
                    metodo, instancia.clase
                );
                return Valor::Nulo;
            }
        };

        let args: Vec<Valor> = argumentos
            .into_iter()
            .map(|arg| self.evaluar_expresion(arg))
            .collect();

        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));

        // Vincular 'th' (__this__) con la instancia actual
        self.entorno_actual
            .definir_variable("__this__".to_string(), Valor::Objeto(instancia.clone()));

        for (i, param) in metodo_def.parametros.iter().enumerate() {
            if let Some(valor) = args.get(i) {
                self.entorno_actual
                    .definir_variable(param.nombre.clone(), valor.clone());
            }
        }

        self.valor_retorno = None;
        for sentencia in metodo_def.cuerpo {
            if let Some(valor) = self.ejecutar_sentencia(sentencia) {
                if let Some(parent) = self.entorno_actual.parent.take() {
                    self.entorno_actual = *parent;
                }
                self.valor_retorno = None;
                return valor;
            }
        }

        if let Some(parent) = self.entorno_actual.parent.take() {
            self.entorno_actual = *parent;
        }
        self.valor_retorno = None;
        Valor::Nulo
    }

    fn interpolar_cadena(&mut self, s: &str) -> Valor {
        let mut resultado = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if c == '&' {
                let (var_name, nuevo_i) = self.leer_nombre_variable(&chars, i + 1);
                if !var_name.is_empty() {
                    let valor_interpolado = self.resolver_variable_interpolada(&var_name);
                    resultado.push_str(&format!("{}", valor_interpolado));
                    i = nuevo_i;
                    continue;
                }
            }
            resultado.push(c);
            i += 1;
        }

        Valor::Texto(resultado)
    }

    fn leer_nombre_variable(&self, chars: &[char], mut i: usize) -> (String, usize) {
        let mut nombre = String::new();

        // Leer primera parte
        while i < chars.len() {
            let ch = chars[i];
            if ch.is_alphanumeric() || ch == '_' {
                nombre.push(ch);
                i += 1;
            } else {
                break;
            }
        }

        if nombre.is_empty() {
            return (nombre, i);
        }

        // Leer propiedades encadenadas
        while i < chars.len() {
            if chars[i] == '.' {
                if i + 1 < chars.len() {
                    let next = chars[i + 1];
                    if next.is_alphanumeric() || next == '_' {
                        nombre.push('.');
                        nombre.push(next);
                        i += 2;

                        while i < chars.len() {
                            let ch = chars[i];
                            if ch.is_alphanumeric() || ch == '_' {
                                nombre.push(ch);
                                i += 1;
                            } else {
                                break;
                            }
                        }
                        continue;
                    }
                }
            }
            break;
        }

        (nombre, i)
    }

    fn resolver_variable_interpolada(&mut self, nombre: &str) -> Valor {
        if !nombre.contains('.') {
            let nombre_real = if nombre == "th" { "__this__" } else { nombre };
            return self
                .entorno_actual
                .obtener(nombre_real)
                .unwrap_or(Valor::Nulo);
        }

        let partes: Vec<&str> = nombre.split('.').collect();
        let primera_parte = if partes[0] == "th" {
            "__this__"
        } else {
            partes[0]
        };

        let Some(mut valor_actual) = self.entorno_actual.obtener(primera_parte) else {
            return Valor::Nulo;
        };

        for &parte in &partes[1..] {
            valor_actual = self.acceder_propiedad_o_metodo(valor_actual, parte);
            if matches!(valor_actual, Valor::Nulo) {
                break;
            }
        }

        valor_actual
    }

    fn acceder_propiedad_o_metodo(&mut self, valor: Valor, propiedad: &str) -> Valor {
        match valor {
            Valor::Objeto(ref inst) => self.acceder_objeto(inst, propiedad),
            Valor::Diccionario(ref mapa) => mapa.get(propiedad).cloned().unwrap_or(Valor::Nulo),
            _ => Valor::Nulo,
        }
    }

    fn acceder_objeto(
        &mut self,
        instancia: &crate::runtime::valores::Instancia,
        propiedad: &str,
    ) -> Valor {
        if let Some(prop_valor) = instancia.propiedades.get(propiedad) {
            return prop_valor.clone();
        }

        self.ejecutar_metodo_objeto(instancia, propiedad)
    }

    fn ejecutar_metodo_objeto(
        &mut self,
        instancia: &crate::runtime::valores::Instancia,
        nombre_metodo: &str,
    ) -> Valor {
        let Some(clase) = self.gestor_clases.obtener_clase(&instancia.clase) else {
            return Valor::Nulo;
        };

        let Some(metodo) = clase.obtener_metodo(nombre_metodo) else {
            return Valor::Nulo;
        };

        let parametros: Vec<String> = metodo.parametros.iter().map(|p| p.nombre.clone()).collect();

        let funcion = Funcion::nueva(nombre_metodo.to_string(), parametros, metodo.cuerpo.clone());

        self.valor_retorno = None;
        let resultado = GestorFunciones::ejecutar_funcion(&funcion, vec![], self);
        self.valor_retorno = None;

        resultado
    }

    fn tprint(&mut self, valor: Valor) {
        let salida = self.convertir_a_texto(valor);
        println!("{}", salida);
    }

    fn convertir_a_texto(&mut self, valor: Valor) -> String {
        match valor {
            Valor::Texto(t) => self.procesar_texto(t),
            Valor::Entero(e) => e.to_string(),
            Valor::Flotante(f) => f.to_string(),
            Valor::Booleano(b) => self.booleano_a_texto(b),
            Valor::Lista(l) => self.lista_a_texto(l),
            Valor::Diccionario(m) => self.diccionario_a_texto(m),
            Valor::Objeto(o) => o.to_string(),
            Valor::Nulo => "null".to_string(),
            _ => "<valor no imprimible>".to_string(),
        }
    }

    fn booleano_a_texto(&self, valor: bool) -> String {
        if valor { "true" } else { "false" }.to_string()
    }

    fn lista_a_texto(&mut self, items: Vec<Valor>) -> String {
        let elementos: Vec<String> = items
            .iter()
            .map(|v| self.convertir_a_texto(v.clone()))
            .collect();
        format!("[{}]", elementos.join(", "))
    }

    fn diccionario_a_texto(&mut self, mapa: HashMap<String, Valor>) -> String {
        let pares: Vec<String> = mapa
            .into_iter()
            .map(|(k, v)| format!("\"{}\": {}", k, self.convertir_a_texto(v)))
            .collect();
        format!("{{{}}}", pares.join(", "))
    }

    fn procesar_texto(&mut self, texto: String) -> String {
        if self.es_texto_multilinea(&texto) {
            return self.procesar_texto_multilinea(texto);
        }

        if self.es_texto_con_comillas_dobles(&texto) {
            return self.procesar_texto_comillas_dobles(texto);
        }

        texto.trim_matches('\'').to_string()
    }

    fn es_texto_multilinea(&self, texto: &str) -> bool {
        texto.starts_with("'''") && texto.ends_with("'''")
    }

    fn es_texto_con_comillas_dobles(&self, texto: &str) -> bool {
        texto.starts_with('"') && texto.ends_with('"')
    }

    fn procesar_texto_multilinea(&mut self, texto: String) -> String {
        let contenido = texto.trim_matches('\'').to_string();
        let normalizado = self.normalizar_multilinea(contenido);
        self.procesar_interpolaciones(normalizado)
    }

    fn procesar_texto_comillas_dobles(&mut self, texto: String) -> String {
        let contenido = texto.trim_matches('"').to_string();
        self.procesar_interpolaciones(contenido)
    }

    fn normalizar_multilinea(&self, texto: String) -> String {
        let lineas: Vec<&str> = texto.lines().collect();
        let minimo = lineas
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
            .min()
            .unwrap_or(0);

        lineas
            .iter()
            .map(|l| if l.len() > minimo { &l[minimo..] } else { *l })
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn procesar_interpolaciones(&mut self, texto: String) -> String {
        let mut salida = String::new();
        let mut chars = texto.chars().peekable();

        while let Some(c) = chars.next() {
            if c != '&' {
                salida.push(c);
                continue;
            }

            let expr = self.leer_expresion_interpolacion(&mut chars);
            let valor = self.evaluar_interpolacion(expr);
            salida.push_str(&valor);
        }

        salida
    }

    fn leer_expresion_interpolacion(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> String {
        let mut expr = String::new();
        while let Some(&ch) = chars.peek() {
            if !self.es_caracter_expresion(ch) {
                break;
            }
            expr.push(chars.next().unwrap());
        }
        expr
    }

    fn es_caracter_expresion(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '.'
    }

    fn evaluar_interpolacion(&mut self, expr: String) -> String {
        let valor = self.parsear_expresion_interpolacion(expr);
        self.convertir_a_texto(valor)
    }

    fn parsear_expresion_interpolacion(&mut self, expr: String) -> Valor {
        if self.es_literal_numerico(&expr) {
            return self.parsear_entero(&expr);
        }

        if expr.contains('.') {
            return self.resolver_acceso_encadenado(&expr);
        }

        self.entorno_actual.obtener(&expr).unwrap_or(Valor::Nulo)
    }

    fn es_literal_numerico(&self, expr: &str) -> bool {
        expr.chars().all(|c| c.is_digit(10))
    }

    fn parsear_entero(&self, expr: &str) -> Valor {
        Valor::Entero(expr.parse::<i64>().unwrap_or(0))
    }

    fn resolver_acceso_encadenado(&mut self, expr: &str) -> Valor {
        let partes: Vec<&str> = expr.split('.').collect();
        let Some(mut valor_actual) = self.entorno_actual.obtener(partes[0]) else {
            return Valor::Nulo;
        };

        for &parte in &partes[1..] {
            valor_actual = self.navegar_propiedad(valor_actual, parte);
            if matches!(valor_actual, Valor::Nulo) {
                break;
            }
        }

        valor_actual
    }

    fn navegar_propiedad(&self, valor: Valor, propiedad: &str) -> Valor {
        match valor {
            Valor::Objeto(ref inst) => inst
                .propiedades
                .get(propiedad)
                .cloned()
                .unwrap_or(Valor::Nulo),
            Valor::Diccionario(ref mapa) => mapa.get(propiedad).cloned().unwrap_or(Valor::Nulo),
            _ => Valor::Nulo,
        }
    }
}
