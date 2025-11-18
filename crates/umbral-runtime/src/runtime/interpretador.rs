use crate::runtime::valores::{Valor, Funcion};
use crate::runtime::entorno::Entorno;
use crate::runtime::clases::{Clase, GestorClases};
use crate::runtime::funciones::GestorFunciones;
use umbral_parser::ast::*;
use std::collections::HashMap;

pub struct Interpretador {
    pub entorno_actual: Entorno,
    pub gestor_clases: GestorClases,
    pub gestor_funciones: GestorFunciones,
    pub valor_retorno: Option<Valor>,
}

impl Interpretador {
    pub fn nuevo() -> Self {
        Self {
            entorno_actual: Entorno::nuevo(None),
            gestor_clases: GestorClases::nuevo(),
            gestor_funciones: GestorFunciones::nuevo(),
            valor_retorno: None,
        }
    }

    pub fn ejecutar_sentencia(&mut self, sentencia: Sentencia) -> Option<Valor> {
        // Si ya hay un valor de retorno, no ejecutar más
        if self.valor_retorno.is_some() {
            return self.valor_retorno.clone();
        }

        match sentencia {
            Sentencia::DeclaracionVariable(decl) => {
                let valor = self.evaluar_expresion(decl.valor);
                self.entorno_actual.definir_variable(decl.nombre, valor.clone());
                None
            }
            Sentencia::DeclaracionConstante(decl) => {
                let valor = self.evaluar_expresion(decl.valor);
                self.entorno_actual.definir_constante(decl.nombre, valor.clone());
                None
            }
            Sentencia::Asignacion(asig) => {
                let valor = self.evaluar_expresion(asig.valor);
                if !self.entorno_actual.asignar(&asig.nombre, valor.clone()) {
                    self.entorno_actual.definir_variable(asig.nombre, valor);
                }
                None
            }
            Sentencia::LlamadoTPrint(lt) => {
                let valor = self.evaluar_expresion(lt.valor);
                self.tprint(valor);
                None
            }
            Sentencia::Return(expr) => {
                let valor = self.evaluar_expresion(expr);
                self.valor_retorno = Some(valor.clone());
                Some(valor)
            }
            Sentencia::If(if_stmt) => {
                self.ejecutar_if(if_stmt)
            }
            Sentencia::Switch(switch) => {
                self.ejecutar_switch(switch)
            }
            Sentencia::For(for_loop) => {
                self.ejecutar_for(for_loop)
            }
            Sentencia::ForEach(foreach) => {
                self.ejecutar_foreach(foreach)
            }
            Sentencia::While(while_loop) => {
                self.ejecutar_while(while_loop)
            }
            Sentencia::DoWhile(do_while) => {
                self.ejecutar_do_while(do_while)
            }
            Sentencia::Funcion(func) => {
                let parametros: Vec<String> = func.parametros.iter()
                    .map(|p| p.nombre.clone())
                    .collect();
                let funcion = Funcion::nueva(func.nombre.clone(), parametros, func.cuerpo);
                self.entorno_actual.definir_variable(func.nombre, Valor::Funcion(funcion));
                None
            }
            Sentencia::Clase(clase) => {
                let clase_obj = Clase::desde_declaracion(&clase);
                self.gestor_clases.registrar_clase(clase_obj);
                None
            }
            Sentencia::LlamadoFuncion(llamado) => {
                let valor = self.evaluar_llamado_funcion(&llamado);
                Some(valor)
            }
            Sentencia::Expresion(expr) => {
                self.evaluar_expresion(expr);
                None
            }
            _ => None,
        }
    }

    pub fn evaluar_expresion(&mut self, expr: Expresion) -> Valor {
        match expr {
            Expresion::LiteralEntero(i) => Valor::Entero(i),
            Expresion::LiteralFloat(f) => Valor::Flotante(f),
            Expresion::LiteralBool(b) => Valor::Booleano(b),
            Expresion::LiteralCadena(s) => {
                // Procesar interpolación
                self.interpolar_cadena(&s)
            }
            Expresion::LiteralNulo => Valor::Nulo,
            Expresion::Identificador(nombre) => {
                self.entorno_actual.obtener(&nombre)
                    .unwrap_or_else(|| {
                        eprintln!("Variable '{}' no encontrada", nombre);
                        Valor::Nulo
                    })
            }
            Expresion::Binaria { izquierda, operador, derecha } => {
                self.evaluar_binaria(*izquierda, &operador, *derecha)
            }
            Expresion::Unaria { operador, expresion } => {
                self.evaluar_unaria(&operador, *expresion)
            }
            Expresion::Agrupada(expr) => {
                self.evaluar_expresion(*expr)
            }
            Expresion::Array(items) => {
                let valores: Vec<Valor> = items.into_iter()
                    .map(|e| self.evaluar_expresion(e))
                    .collect();
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
        let valor = self.evaluar_expresion(expr);
        match op {
            "!" => Valor::Booleano(!valor.es_verdadero()),
            "-" => match valor {
                Valor::Entero(i) => Valor::Entero(-i),
                Valor::Flotante(f) => Valor::Flotante(-f),
                _ => Valor::Nulo,
            }
            _ => valor,
        }
    }

    fn sumar(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) => Valor::Entero(a + b),
            (Valor::Flotante(a), Valor::Flotante(b)) => Valor::Flotante(a + b),
            (Valor::Entero(a), Valor::Flotante(b)) => Valor::Flotante(a as f64 + b),
            (Valor::Flotante(a), Valor::Entero(b)) => Valor::Flotante(a + b as f64),
            (Valor::Texto(a), Valor::Texto(b)) => Valor::Texto(format!("{}{}", a, b)),
            _ => Valor::Nulo,
        }
    }

    fn restar(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) => Valor::Entero(a - b),
            (Valor::Flotante(a), Valor::Flotante(b)) => Valor::Flotante(a - b),
            (Valor::Entero(a), Valor::Flotante(b)) => Valor::Flotante(a as f64 - b),
            (Valor::Flotante(a), Valor::Entero(b)) => Valor::Flotante(a - b as f64),
            _ => Valor::Nulo,
        }
    }

    fn multiplicar(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) => Valor::Entero(a * b),
            (Valor::Flotante(a), Valor::Flotante(b)) => Valor::Flotante(a * b),
            (Valor::Entero(a), Valor::Flotante(b)) => Valor::Flotante(a as f64 * b),
            (Valor::Flotante(a), Valor::Entero(b)) => Valor::Flotante(a * b as f64),
            _ => Valor::Nulo,
        }
    }

    fn dividir(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) if b != 0 => Valor::Entero(a / b),
            (Valor::Flotante(a), Valor::Flotante(b)) if b != 0.0 => Valor::Flotante(a / b),
            (Valor::Entero(a), Valor::Flotante(b)) if b != 0.0 => Valor::Flotante(a as f64 / b),
            (Valor::Flotante(a), Valor::Entero(b)) if b != 0 => Valor::Flotante(a / b as f64),
            _ => {
                eprintln!("División por cero");
                Valor::Nulo
            }
        }
    }

    fn modulo(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) if b != 0 => Valor::Entero(a % b),
            _ => Valor::Nulo,
        }
    }

    fn son_iguales(&self, a: &Valor, b: &Valor) -> bool {
        match (a, b) {
            (Valor::Entero(x), Valor::Entero(y)) => x == y,
            (Valor::Flotante(x), Valor::Flotante(y)) => x == y,
            (Valor::Booleano(x), Valor::Booleano(y)) => x == y,
            (Valor::Texto(x), Valor::Texto(y)) => x == y,
            (Valor::Nulo, Valor::Nulo) => true,
            _ => false,
        }
    }

    fn comparar_menor(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) => Valor::Booleano(a < b),
            (Valor::Flotante(a), Valor::Flotante(b)) => Valor::Booleano(a < b),
            (Valor::Entero(a), Valor::Flotante(b)) => Valor::Booleano((a as f64) < b),
            (Valor::Flotante(a), Valor::Entero(b)) => Valor::Booleano(a < (b as f64)),
            _ => Valor::Booleano(false),
        }
    }

    fn comparar_mayor(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) => Valor::Booleano(a > b),
            (Valor::Flotante(a), Valor::Flotante(b)) => Valor::Booleano(a > b),
            (Valor::Entero(a), Valor::Flotante(b)) => Valor::Booleano((a as f64) > b),
            (Valor::Flotante(a), Valor::Entero(b)) => Valor::Booleano(a > (b as f64)),
            _ => Valor::Booleano(false),
        }
    }

    fn comparar_menor_igual(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) => Valor::Booleano(a <= b),
            (Valor::Flotante(a), Valor::Flotante(b)) => Valor::Booleano(a <= b),
            (Valor::Entero(a), Valor::Flotante(b)) => Valor::Booleano((a as f64) <= b),
            (Valor::Flotante(a), Valor::Entero(b)) => Valor::Booleano(a <= (b as f64)),
            _ => Valor::Booleano(false),
        }
    }

    fn comparar_mayor_igual(&self, izq: Valor, der: Valor) -> Valor {
        match (izq, der) {
            (Valor::Entero(a), Valor::Entero(b)) => Valor::Booleano(a >= b),
            (Valor::Flotante(a), Valor::Flotante(b)) => Valor::Booleano(a >= b),
            (Valor::Entero(a), Valor::Flotante(b)) => Valor::Booleano((a as f64) >= b),
            (Valor::Flotante(a), Valor::Entero(b)) => Valor::Booleano(a >= (b as f64)),
            _ => Valor::Booleano(false),
        }
    }

    fn ejecutar_if(&mut self, if_stmt: If) -> Option<Valor> {
        let condicion = self.evaluar_expresion(if_stmt.condicion);
        
        if condicion.es_verdadero() {
            for sentencia in if_stmt.bloque_entonces {
                if let Some(valor) = self.ejecutar_sentencia(sentencia) {
                    return Some(valor);
                }
            }
        } else {
            for else_if in if_stmt.else_ifs {
                let cond = self.evaluar_expresion(else_if.condicion);
                if cond.es_verdadero() {
                    for sentencia in else_if.bloque {
                        if let Some(valor) = self.ejecutar_sentencia(sentencia) {
                            return Some(valor);
                        }
                    }
                    return None;
                }
            }
            
            if let Some(bloque_else) = if_stmt.bloque_else {
                for sentencia in bloque_else {
                    if let Some(valor) = self.ejecutar_sentencia(sentencia) {
                        return Some(valor);
                    }
                }
            }
        }
        
        None
    }

    fn ejecutar_switch(&mut self, switch: Switch) -> Option<Valor> {
        let valor_switch = self.evaluar_expresion(switch.expresion);
        
        for caso in switch.casos {
            let valor_caso = self.evaluar_expresion(caso.valor);
            if self.son_iguales(&valor_switch, &valor_caso) {
                for sentencia in caso.bloque {
                    if let Some(valor) = self.ejecutar_sentencia(sentencia) {
                        return Some(valor);
                    }
                }
                return None;
            }
        }
        
        if let Some(default) = switch.default {
            for sentencia in default {
                if let Some(valor) = self.ejecutar_sentencia(sentencia) {
                    return Some(valor);
                }
            }
        }
        
        None
    }

    fn ejecutar_for(&mut self, for_loop: For) -> Option<Valor> {
        // Crear nuevo ámbito
        let parent = self.entorno_actual.clone();
        let entorno_anterior = std::mem::replace(
            &mut self.entorno_actual,
            Entorno::nuevo(Some(parent))
        );
        
        // Inicialización
        self.ejecutar_sentencia(*for_loop.inicializacion);
        
        // Bucle
        loop {
            let condicion = self.evaluar_expresion(for_loop.condicion.clone());
            if !condicion.es_verdadero() {
                break;
            }
            
            for sentencia in &for_loop.bloque {
                if let Some(valor) = self.ejecutar_sentencia(sentencia.clone()) {
                    self.entorno_actual = entorno_anterior;
                    return Some(valor);
                }
            }
            
            self.evaluar_expresion(for_loop.incremento.clone());
        }
        
        // Restaurar entorno
        self.entorno_actual = entorno_anterior;
        None
    }

    fn ejecutar_foreach(&mut self, foreach: ForEach) -> Option<Valor> {
        let iterable = self.evaluar_expresion(foreach.iterable);
        
        if let Valor::Lista(items) = iterable {
            let parent = self.entorno_actual.clone();
            let entorno_anterior = std::mem::replace(
                &mut self.entorno_actual,
                Entorno::nuevo(Some(parent))
            );
            
            for item in items {
                self.entorno_actual.definir_variable(foreach.variable.clone(), item);
                
                for sentencia in &foreach.bloque {
                    if let Some(valor) = self.ejecutar_sentencia(sentencia.clone()) {
                        self.entorno_actual = entorno_anterior;
                        return Some(valor);
                    }
                }
            }
            
            self.entorno_actual = entorno_anterior;
        }
        
        None
    }

    fn ejecutar_while(&mut self, while_loop: While) -> Option<Valor> {
        loop {
            let condicion = self.evaluar_expresion(while_loop.condicion.clone());
            if !condicion.es_verdadero() {
                break;
            }
            
            for sentencia in &while_loop.bloque {
                if let Some(valor) = self.ejecutar_sentencia(sentencia.clone()) {
                    return Some(valor);
                }
            }
        }
        
        None
    }

    fn ejecutar_do_while(&mut self, do_while: DoWhile) -> Option<Valor> {
        loop {
            for sentencia in &do_while.bloque {
                if let Some(valor) = self.ejecutar_sentencia(sentencia.clone()) {
                    return Some(valor);
                }
            }
            
            let condicion = self.evaluar_expresion(do_while.condicion.clone());
            if !condicion.es_verdadero() {
                break;
            }
        }
        
        None
    }

    fn evaluar_llamado_funcion(&mut self, llamado: &LlamadoFuncion) -> Valor {
        let argumentos: Vec<Valor> = llamado.argumentos.iter()
            .map(|arg| self.evaluar_expresion(arg.clone()))
            .collect();
        
        if let Some(Valor::Funcion(func)) = self.entorno_actual.obtener(&llamado.nombre) {
            self.valor_retorno = None;
            let resultado = GestorFunciones::ejecutar_funcion(&func, argumentos, self);
            self.valor_retorno = None;
            resultado
        } else {
            eprintln!("Función '{}' no encontrada", llamado.nombre);
            Valor::Nulo
        }
    }

    fn evaluar_instanciacion(&mut self, tipo: &str, argumentos: Vec<Expresion>) -> Valor {
        // Evaluar argumentos primero
        let mut args: Vec<Valor> = Vec::new();
        for arg in argumentos {
            args.push(self.evaluar_expresion(arg));
        }
        
        if let Some(instancia) = self.gestor_clases.crear_instancia(tipo) {
            // Si hay un constructor, ejecutarlo
            let constructor = self.gestor_clases.obtener_clase(tipo)
                .and_then(|c| c.constructor.clone());
            
            if let Some(constructor) = constructor {
                // Crear entorno con 'this'
                let parent = self.entorno_actual.clone();
                let entorno_anterior = std::mem::replace(
                    &mut self.entorno_actual,
                    Entorno::nuevo(Some(parent))
                );
                
                // Asignar parámetros
                for (i, param) in constructor.parametros.iter().enumerate() {
                    if let Some(valor) = args.get(i) {
                        self.entorno_actual.definir_variable(param.nombre.clone(), valor.clone());
                    }
                }
                
                // Definir 'th' (this) como una referencia mutable
                // Por ahora, simplemente ejecutar el cuerpo
                let cuerpo = constructor.cuerpo.clone();
                for sentencia in cuerpo {
                    self.ejecutar_sentencia(sentencia);
                }
                
                // Restaurar entorno
                self.entorno_actual = entorno_anterior;
            }
            
            Valor::Objeto(instancia)
        } else {
            eprintln!("Clase '{}' no encontrada", tipo);
            Valor::Nulo
        }
    }

    fn evaluar_acceso_propiedad(&mut self, objeto: Expresion, propiedad: &str) -> Valor {
        let obj_valor = self.evaluar_expresion(objeto);
        
        match obj_valor {
            Valor::Objeto(instancia) => {
                instancia.propiedades.get(propiedad)
                    .cloned()
                    .unwrap_or_else(|| {
                        eprintln!("Propiedad '{}' no encontrada", propiedad);
                        Valor::Nulo
                    })
            }
            Valor::Diccionario(mapa) => {
                mapa.get(propiedad)
                    .cloned()
                    .unwrap_or_else(|| {
                        eprintln!("Clave '{}' no encontrada", propiedad);
                        Valor::Nulo
                    })
            }
            Valor::Lista(ref items) if propiedad == "length" => {
                Valor::Entero(items.len() as i64)
            }
            _ => {
                eprintln!("No se puede acceder a la propiedad '{}' de {:?}", propiedad, obj_valor);
                Valor::Nulo
            }
        }
    }

    fn evaluar_acceso_indice(&mut self, objeto: Expresion, indice: Expresion) -> Valor {
        let obj_valor = self.evaluar_expresion(objeto);
        let indice_valor = self.evaluar_expresion(indice);
        
        match (obj_valor, indice_valor) {
            (Valor::Lista(items), Valor::Entero(i)) => {
                if i >= 0 && (i as usize) < items.len() {
                    items[i as usize].clone()
                } else {
                    eprintln!("Índice fuera de rango: {}", i);
                    Valor::Nulo
                }
            }
            _ => Valor::Nulo,
        }
    }

    fn interpolar_cadena(&mut self, s: &str) -> Valor {
        let mut resultado = String::new();
        let mut chars = s.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '&' {
                // Leer el nombre de la variable
                let mut var_name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                        var_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                
                if !var_name.is_empty() {
                    // Evaluar la variable o expresión
                    if var_name.contains('.') {
                        // Manejar acceso a propiedades
                        let partes: Vec<&str> = var_name.split('.').collect();
                        if let Some(valor) = self.entorno_actual.obtener(partes[0]) {
                            let mut valor_actual = valor;
                            for parte in &partes[1..] {
                                match valor_actual {
                                    Valor::Objeto(ref inst) => {
                                        valor_actual = inst.propiedades.get(*parte)
                                            .cloned()
                                            .unwrap_or(Valor::Nulo);
                                    }
                                    Valor::Diccionario(ref mapa) => {
                                        valor_actual = mapa.get(*parte)
                                            .cloned()
                                            .unwrap_or(Valor::Nulo);
                                    }
                                    _ => {
                                        valor_actual = Valor::Nulo;
                                        break;
                                    }
                                }
                            }
                            resultado.push_str(&format!("{}", valor_actual));
                        }
                    } else {
                        if let Some(valor) = self.entorno_actual.obtener(&var_name) {
                            resultado.push_str(&format!("{}", valor));
                        }
                    }
                }
            } else {
                resultado.push(c);
            }
        }
        
        Valor::Texto(resultado)
    }

    pub fn tprint(&self, valor: Valor) {
        println!("{}", valor);
    }
}
