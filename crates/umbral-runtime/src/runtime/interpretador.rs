use crate::runtime::clases::{Clase, GestorClases};
use crate::runtime::entorno::Entorno;
use crate::runtime::enums::GestorEnums;
use crate::runtime::funciones::GestorFunciones;
use crate::runtime::interfaces::{GestorInterfaces, Interfaz};
use crate::runtime::valores::{Funcion, SharedPromesa, Valor};
use async_recursion::async_recursion;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use umbral_parser::ast::*;

#[derive(Clone)]
pub struct Interpretador {
    pub entorno_actual: Entorno,
    pub gestor_clases: GestorClases,
    pub gestor_funciones: GestorFunciones,
    pub gestor_interfaces: GestorInterfaces,
    pub gestor_enums: GestorEnums,
    pub valor_retorno: Option<Valor>,
    pub estado_excepcion: Option<Valor>,
    pub exportaciones: HashMap<String, bool>,
    pub directorio_base: PathBuf,
}

impl Interpretador {
    pub fn nuevo() -> Self {
        let mut inter = Self {
            entorno_actual: Entorno::nuevo(None),
            gestor_clases: GestorClases::nuevo(),
            gestor_funciones: GestorFunciones::nuevo(),
            gestor_interfaces: GestorInterfaces::nuevo(),
            gestor_enums: GestorEnums::nuevo(),
            valor_retorno: None,
            estado_excepcion: None,
            exportaciones: HashMap::new(),
            directorio_base: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        };

        crate::runtime::stdlib::registrar_stdlib(&mut inter);
        inter
    }

    pub fn establecer_directorio_base(&mut self, ruta: PathBuf) {
        self.directorio_base = ruta;
    }

    #[async_recursion]
    pub async fn ejecutar_sentencia(&mut self, sentencia: Sentencia) -> Option<Valor> {
        if self.tiene_valor_retorno() {
            return self.valor_retorno.clone();
        }

        match sentencia {
            Sentencia::DeclaracionVariable(decl) => self.ejecutar_declaracion_variable(decl).await,
            Sentencia::DeclaracionConstante(decl) => {
                self.ejecutar_declaracion_constante(decl).await
            }
            Sentencia::Asignacion(asig) => self.ejecutar_asignacion(asig).await,
            Sentencia::LlamadoTPrint(lt) => self.ejecutar_tprint(lt).await,
            Sentencia::Return(expr) => self.ejecutar_return(expr).await,
            Sentencia::If(if_stmt) => self.ejecutar_if(if_stmt).await,
            Sentencia::Switch(switch) => self.ejecutar_switch(switch).await,
            Sentencia::For(for_loop) => self.ejecutar_for(for_loop).await,
            Sentencia::ForEach(foreach) => self.ejecutar_foreach(foreach).await,
            Sentencia::While(while_loop) => self.ejecutar_while(while_loop).await,
            Sentencia::DoWhile(do_while) => self.ejecutar_do_while(do_while).await,
            Sentencia::Funcion(func) => {
                self.registrar_funcion(func);
                None
            }
            Sentencia::Clase(clase) => {
                self.registrar_clase(clase);
                None
            }
            Sentencia::Interfaz(interfaz) => {
                self.registrar_interfaz(interfaz);
                None
            }
            Sentencia::Enum(decl_enum) => {
                self.registrar_enum(decl_enum).await;
                None
            }
            Sentencia::LlamadoFuncion(llamado) => {
                Some(self.evaluar_llamado_funcion(&llamado).await)
            }
            Sentencia::Importacion(imp) => self.ejecutar_importacion(imp).await,
            Sentencia::TryCatch(stmt) => self.ejecutar_try_catch(stmt).await,
            Sentencia::Throw(stmt) => self.ejecutar_throw(stmt).await,
            Sentencia::Exportacion(nombre) => {
                self.exportaciones.insert(nombre, true);
                None
            }
            Sentencia::Expresion(expr) => {
                self.evaluar_expresion(expr).await;
                None
            }
        }
    }

    fn tiene_valor_retorno(&self) -> bool {
        self.valor_retorno.is_some()
    }

    async fn ejecutar_declaracion_variable(&mut self, decl: DeclaracionVariable) -> Option<Valor> {
        let valor = self.evaluar_expresion(decl.valor).await;

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

    async fn ejecutar_declaracion_constante(
        &mut self,
        decl: DeclaracionConstante,
    ) -> Option<Valor> {
        let valor = self.evaluar_expresion(decl.valor).await;
        self.entorno_actual
            .definir_constante(decl.nombre.clone(), valor);
        if decl.exportado {
            self.exportaciones.insert(decl.nombre, true);
        }
        None
    }

    async fn ejecutar_asignacion(&mut self, asig: Asignacion) -> Option<Valor> {
        let valor = self.evaluar_expresion(asig.valor).await;

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
                self.asignar_propiedad_objeto(*objeto, propiedad, valor)
                    .await;
            }
        }

        None
    }

    async fn asignar_propiedad_objeto(
        &mut self,
        objeto_expr: Expresion,
        propiedad: String,
        valor: Valor,
    ) {
        let obj_valor = self.evaluar_expresion(objeto_expr.clone()).await;

        let Valor::Objeto(instancia) = obj_valor else {
            eprintln!("No se puede asignar propiedad a un valor que no es objeto");
            return;
        };

        if let Ok(mut props) = instancia.propiedades.lock() {
            props.insert(propiedad.clone(), valor.clone());
        }

        if !matches!(objeto_expr, Expresion::This) {
            return;
        }

        if !self
            .entorno_actual
            .asignar("__this__", Valor::Objeto(instancia.clone()))
        {
            self.entorno_actual
                .definir_variable("__this__".to_string(), Valor::Objeto(instancia));
        }
    }

    async fn ejecutar_tprint(&mut self, lt: LlamadoTPrint) -> Option<Valor> {
        let valor = self.evaluar_expresion(lt.valor).await;
        self.tprint(valor).await;
        None
    }

    async fn ejecutar_return(&mut self, expr: Expresion) -> Option<Valor> {
        let valor = self.evaluar_expresion(expr).await;
        self.valor_retorno = Some(valor.clone());
        Some(valor)
    }

    fn registrar_funcion(&mut self, func: DeclaracionFuncion) -> Option<Valor> {
        let parametros: Vec<String> = func.parametros.iter().map(|p| p.nombre.clone()).collect();
        let funcion = Funcion::nueva(func.nombre.clone(), parametros, func.cuerpo, func.es_async);
        self.entorno_actual
            .definir_variable(func.nombre.clone(), Valor::Funcion(funcion));
        if func.exportado {
            self.exportaciones.insert(func.nombre, true);
        }
        None
    }

    fn registrar_interfaz(&mut self, interfaz: DeclaracionInterfaz) -> Option<Valor> {
        let interfaz_obj = Interfaz::desde_declaracion(&interfaz);
        let nombre = interfaz_obj.nombre.clone();
        self.gestor_interfaces.registrar(interfaz_obj);
        if interfaz.exportado {
            self.exportaciones.insert(nombre, true);
        }
        None
    }

    async fn registrar_enum(&mut self, decl_enum: DeclaracionEnum) -> Option<Valor> {
        let nombre_enum = decl_enum.nombre.clone();
        let enum_obj = crate::runtime::enums::Enum::desde_declaracion(&decl_enum);

        self.gestor_enums.registrar(enum_obj);

        let mut dict_variantes = HashMap::new();

        for (indice, variante_enum) in decl_enum.variantes.iter().enumerate() {
            let nombre_variante = variante_enum.nombre.clone();

            let valor_asociado = if let Some(ref expr_valor) = variante_enum.valor {
                self.evaluar_expresion(expr_valor.clone()).await
            } else {
                Valor::Entero(indice as i64)
            };

            dict_variantes.insert(nombre_variante, valor_asociado);
        }

        self.entorno_actual
            .definir_variable(nombre_enum.clone(), Valor::Diccionario(dict_variantes));

        if decl_enum.exportado {
            self.exportaciones.insert(nombre_enum, true);
        }
        None
    }

    fn registrar_clase(&mut self, clase: DeclaracionClase) -> Option<Valor> {
        self.validar_implementaciones(&clase);

        let clase_obj = Clase::desde_declaracion(&clase);
        let nombre_clase = clase_obj.nombre.clone();
        self.gestor_clases.registrar_clase(clase_obj);
        if clase.exportado {
            self.exportaciones.insert(nombre_clase, true);
        }
        None
    }

    fn validar_implementaciones(&self, clase: &DeclaracionClase) {
        for nombre_interfaz in &clase.implementaciones {
            self.validar_implementacion_interfaz(clase, nombre_interfaz);
        }
    }

    fn validar_implementacion_interfaz(&self, clase: &DeclaracionClase, nombre_interfaz: &str) {
        let interfaz_opt = self.gestor_interfaces.obtener(nombre_interfaz);

        if interfaz_opt.is_none() {
            eprintln!("Error: La interfaz '{}' no está definida.", nombre_interfaz);
            return;
        }

        let interfaz = interfaz_opt.unwrap();
        for (nombre_metodo, metodo_interfaz) in &interfaz.metodos {
            self.validar_metodo_interfaz(clase, nombre_interfaz, nombre_metodo, metodo_interfaz);
        }
    }

    fn validar_metodo_interfaz(
        &self,
        clase: &DeclaracionClase,
        nombre_interfaz: &str,
        nombre_metodo: &str,
        metodo_interfaz: &Metodo,
    ) {
        let metodo_clase_opt = clase.metodos.iter().find(|m| m.nombre == *nombre_metodo);

        if metodo_clase_opt.is_none() {
            eprintln!(
                "Error: La clase '{}' no implementa el método '{}' de la interfaz '{}'.",
                clase.nombre, nombre_metodo, nombre_interfaz
            );
            return;
        }

        let metodo_clase = metodo_clase_opt.unwrap();
        if metodo_clase.parametros.len() != metodo_interfaz.parametros.len() {
            eprintln!(
                "Error: La clase '{}' implementa incorrectamente el método '{}' de la interfaz '{}'. Diferente número de parámetros.",
                clase.nombre, nombre_metodo, nombre_interfaz
            );
        }
    }

    async fn ejecutar_importacion(
        &mut self,
        imp: umbral_parser::ast::Importacion,
    ) -> Option<Valor> {
        let (contenido, ruta_encontrada) = self.buscar_modulo(&imp.ruta)?;
        let programa = self.parsear_modulo(&contenido, &ruta_encontrada)?;
        let interprete_modulo = self.ejecutar_modulo(programa, &ruta_encontrada).await;
        self.importar_items(imp.items, &interprete_modulo);
        None
    }

    fn buscar_modulo(&self, ruta: &str) -> Option<(String, PathBuf)> {
        let es_ruta_relativa = self.es_ruta_relativa(ruta);

        if es_ruta_relativa {
            return self.buscar_ruta_relativa(ruta);
        }

        self.buscar_modulo_ump(ruta)
    }

    fn es_ruta_relativa(&self, ruta: &str) -> bool {
        ruta.contains('/')
            || ruta.starts_with("./")
            || ruta.starts_with("../")
            || ruta.ends_with(".um")
    }

    fn buscar_ruta_relativa(&self, ruta: &str) -> Option<(String, PathBuf)> {
        let ruta_original = PathBuf::from(ruta);
        let rutas_posibles = self.construir_rutas_relativas(&ruta_original);

        self.intentar_leer_archivos(rutas_posibles, ruta)
    }

    fn construir_rutas_relativas(&self, ruta: &PathBuf) -> Vec<PathBuf> {
        vec![
            self.directorio_base.join(ruta),
            self.directorio_base.join("modules_ump").join(ruta),
            self.directorio_base
                .join("modules_ump")
                .join(ruta)
                .join("main.um"),
            self.directorio_base
                .join("modules_ump")
                .join(ruta)
                .join("index.um"),
        ]
    }

    fn buscar_modulo_ump(&self, nombre_modulo: &str) -> Option<(String, PathBuf)> {
        let mut dir_actual = self.directorio_base.clone();

        loop {
            let resultado = self.buscar_en_directorio(&dir_actual, nombre_modulo);
            if resultado.is_some() {
                return resultado;
            }

            if !dir_actual.pop() {
                break;
            }
        }

        self.reportar_modulo_no_encontrado(nombre_modulo);
        None
    }

    fn buscar_en_directorio(
        &self,
        dir: &PathBuf,
        nombre_modulo: &str,
    ) -> Option<(String, PathBuf)> {
        use std::fs;
        let modules_ump = dir.join("modules_ump");

        if !modules_ump.exists() || !modules_ump.is_dir() {
            return None;
        }

        let rutas = vec![
            modules_ump.join(nombre_modulo).join("src").join("main.um"),
            modules_ump.join(nombre_modulo).join("main.um"),
            modules_ump.join(nombre_modulo).join("index.um"),
        ];

        for ruta in rutas {
            if let Ok(contenido) = fs::read_to_string(&ruta) {
                return Some((contenido, ruta));
            }
        }

        None
    }

    fn intentar_leer_archivos(
        &self,
        rutas: Vec<PathBuf>,
        ruta_original: &str,
    ) -> Option<(String, PathBuf)> {
        use std::fs;

        for ruta in &rutas {
            if let Ok(contenido) = fs::read_to_string(ruta) {
                return Some((contenido, ruta.clone()));
            }
        }

        self.reportar_ruta_no_encontrada(ruta_original, &rutas);
        None
    }

    fn reportar_ruta_no_encontrada(&self, ruta: &str, rutas_intentadas: &[PathBuf]) {
        eprintln!(
            "Error: No se pudo encontrar el módulo '{}'. Se buscaron las siguientes rutas:",
            ruta
        );
        for ruta_intentada in rutas_intentadas.iter().take(2) {
            eprintln!(" - {}", ruta_intentada.display());
        }
    }

    fn reportar_modulo_no_encontrado(&self, nombre: &str) {
        eprintln!(
            "Error: No se pudo encontrar el módulo UMP '{}' en modules_ump.",
            nombre
        );
        eprintln!(
            "Asegúrate de que el módulo esté instalado con 'ump add {}'.",
            nombre
        );
    }

    fn parsear_modulo(
        &self,
        contenido: &str,
        ruta: &PathBuf,
    ) -> Option<umbral_parser::ast::Programa> {
        let tokens = umbral_lexer::analizar(contenido);
        match umbral_parser::parsear_programa(tokens) {
            Ok(programa) => Some(programa),
            Err(e) => {
                eprintln!("Error al parsear archivo '{}': {:?}", ruta.display(), e);
                None
            }
        }
    }

    async fn ejecutar_modulo(
        &self,
        programa: umbral_parser::ast::Programa,
        ruta: &PathBuf,
    ) -> Interpretador {
        let mut interprete = Interpretador::nuevo();

        if let Some(parent) = ruta.parent() {
            interprete.establecer_directorio_base(parent.to_path_buf());
        }

        for sentencia in programa.sentencias {
            interprete.ejecutar_sentencia(sentencia).await;
        }

        interprete
    }

    fn importar_items(
        &mut self,
        items: Vec<umbral_parser::ast::ItemImportacion>,
        modulo: &Interpretador,
    ) {
        for item in items {
            self.procesar_item_importacion(item, modulo);
        }
    }

    fn procesar_item_importacion(
        &mut self,
        item: umbral_parser::ast::ItemImportacion,
        modulo: &Interpretador,
    ) {
        use umbral_parser::ast::ItemImportacion;

        match item {
            ItemImportacion::Todo(alias) => self.importar_todo(alias, modulo),
            ItemImportacion::Nombre(nombre, alias) => self.importar_nombre(nombre, alias, modulo),
            ItemImportacion::Modulo(nombre_var) => {
                self.importar_modulo_como_objeto(nombre_var, modulo)
            }
            ItemImportacion::ListaNombres(items) => {
                for sub_item in items {
                    self.procesar_item_importacion(sub_item, modulo);
                }
            }
        }
    }

    fn importar_modulo_como_objeto(&mut self, nombre_var: String, modulo: &Interpretador) {
        let mut mapa_exportaciones = HashMap::new();

        for (nombre, valor) in &modulo.entorno_actual.variables {
            if modulo.exportaciones.get(nombre).copied().unwrap_or(false) {
                mapa_exportaciones.insert(nombre.clone(), valor.clone());
            }
        }

        for (nombre, valor) in &modulo.entorno_actual.constantes {
            if modulo.exportaciones.get(nombre).copied().unwrap_or(false) {
                mapa_exportaciones.insert(nombre.clone(), valor.clone());
            }
        }

        for (nombre, clase) in &modulo.gestor_clases.clases {
            if modulo.exportaciones.get(nombre).copied().unwrap_or(false) {
                let nombre_unico = format!("__modulo_{}_{}", nombre_var, nombre);

                let mut clase_copia = clase.clone();
                clase_copia.nombre = nombre_unico.clone();
                self.gestor_clases
                    .clases
                    .insert(nombre_unico.clone(), clase_copia);

                mapa_exportaciones.insert(nombre.clone(), Valor::Clase(nombre_unico));
            }
        }

        self.entorno_actual
            .definir_variable(nombre_var, Valor::Diccionario(mapa_exportaciones));
    }

    fn importar_todo(&mut self, alias: Option<String>, modulo: &Interpretador) {
        let alias_nombre = alias.unwrap_or_else(|| "mod".to_string());

        self.importar_variables_exportadas(&alias_nombre, modulo);
        self.importar_clases_exportadas(&alias_nombre, modulo);
    }

    fn importar_variables_exportadas(&mut self, alias: &str, modulo: &Interpretador) {
        for (nombre, valor) in &modulo.entorno_actual.variables {
            if !modulo.exportaciones.get(nombre).copied().unwrap_or(false) {
                continue;
            }

            let nombre_final = format!("{}_{}", alias, nombre);
            self.entorno_actual
                .definir_variable(nombre_final, valor.clone());
        }
    }

    fn importar_clases_exportadas(&mut self, alias: &str, modulo: &Interpretador) {
        for (nombre, clase) in &modulo.gestor_clases.clases {
            if !modulo.exportaciones.get(nombre).copied().unwrap_or(false) {
                continue;
            }

            let nombre_final = format!("{}_{}", alias, nombre);
            self.gestor_clases
                .clases
                .insert(nombre_final, clase.clone());
        }
    }

    fn importar_nombre(&mut self, nombre: String, alias: Option<String>, modulo: &Interpretador) {
        if !modulo.exportaciones.get(&nombre).copied().unwrap_or(false) {
            eprintln!("Advertencia: '{}' no está exportado en el módulo", nombre);
            return;
        }

        let nombre_final = alias.unwrap_or_else(|| nombre.clone());

        if self.intentar_importar_variable(&nombre, &nombre_final, modulo) {
            return;
        }

        if self.intentar_importar_clase(&nombre, &nombre_final, modulo) {
            return;
        }

        eprintln!("Advertencia: '{}' no encontrado en el módulo", nombre);
    }

    fn intentar_importar_variable(
        &mut self,
        nombre: &str,
        nombre_final: &str,
        modulo: &Interpretador,
    ) -> bool {
        if let Some(valor) = modulo.entorno_actual.obtener(nombre) {
            self.entorno_actual
                .definir_variable(nombre_final.to_string(), valor);
            return true;
        }
        false
    }

    fn intentar_importar_clase(
        &mut self,
        nombre: &str,
        nombre_final: &str,
        modulo: &Interpretador,
    ) -> bool {
        if let Some(clase) = modulo.gestor_clases.clases.get(nombre) {
            self.gestor_clases
                .clases
                .insert(nombre_final.to_string(), clase.clone());
            return true;
        }
        false
    }

    #[async_recursion]
    pub async fn evaluar_expresion(&mut self, expr: Expresion) -> Valor {
        match expr {
            Expresion::LiteralEntero(i) => Valor::Entero(i),
            Expresion::LiteralFloat(f) => Valor::Flotante(f),
            Expresion::LiteralBool(b) => Valor::Booleano(b),
            Expresion::LiteralCadena(s) => self.evaluar_literal_cadena(s).await,
            Expresion::LiteralCadenaLiteral(s) => Valor::Texto(s),
            Expresion::LiteralNulo => Valor::Nulo,
            Expresion::Identificador(nombre) => self.evaluar_identificador(&nombre),
            Expresion::Binaria {
                izquierda,
                operador,
                derecha,
            } => self.evaluar_binaria(*izquierda, &operador, *derecha).await,
            Expresion::Unaria {
                operador,
                expresion,
            } => self.evaluar_unaria(&operador, *expresion).await,
            Expresion::Agrupada(expr) => self.evaluar_expresion(*expr).await,
            Expresion::This => self.evaluar_this(),
            Expresion::Spread(expr) => self.evaluar_expresion(*expr).await,
            Expresion::Array(items) => self.evaluar_array(items).await,
            Expresion::Objeto(pares) => self.evaluar_objeto(pares).await,
            Expresion::Instanciacion { tipo, argumentos } => {
                self.evaluar_instanciacion(&tipo, argumentos).await
            }
            Expresion::AccesoPropiedad { objeto, propiedad } => {
                self.evaluar_acceso_propiedad(*objeto, &propiedad).await
            }
            Expresion::AccesoIndice { objeto, indice } => {
                self.evaluar_acceso_indice(*objeto, *indice).await
            }
            Expresion::LlamadoMetodo {
                objeto,
                metodo,
                argumentos,
            } => {
                self.evaluar_llamado_metodo(*objeto, &metodo, argumentos)
                    .await
            }
            Expresion::LlamadoFuncion { nombre, argumentos } => {
                let mut args = Vec::new();
                for arg in argumentos {
                    args.push(self.evaluar_expresion(arg).await);
                }

                if self.es_funcion_builtin(&nombre) {
                    return self.ejecutar_funcion_builtin(&nombre, args).await;
                }

                self.ejecutar_funcion_usuario(&nombre, args).await
            }
            Expresion::Await(expr) => {
                let valor = self.evaluar_expresion(*expr).await;
                if let Valor::Promesa(SharedPromesa(arc_mutex)) = valor {
                    let handle_opt = {
                        let mut guard = arc_mutex.lock().unwrap();
                        guard.take()
                    };

                    if let Some(handle) = handle_opt {
                        match handle.await {
                            Ok(v) => v,
                            Err(e) => {
                                eprintln!("Error en tarea asincrona: {:?}", e);
                                Valor::Nulo
                            }
                        }
                    } else {
                        Valor::Nulo
                    }
                } else {
                    valor
                }
            }
        }
    }

    #[async_recursion]
    async fn evaluar_literal_cadena(&mut self, s: String) -> Valor {
        let contenido = s.trim_matches('"').to_string();
        let interpolado = self.procesar_interpolaciones(contenido).await;
        Valor::Texto(interpolado)
    }

    fn evaluar_identificador(&self, nombre: &str) -> Valor {
        self.entorno_actual.obtener(nombre).unwrap_or_else(|| {
            eprintln!("Variable '{}' no encontrada", nombre);
            Valor::Nulo
        })
    }

    fn evaluar_this(&self) -> Valor {
        self.entorno_actual.obtener("__this__").unwrap_or_else(|| {
            eprintln!("'th' solo puede usarse dentro de métodos o constructores de clase");
            Valor::Nulo
        })
    }

    #[async_recursion]
    async fn evaluar_array(&mut self, items: Vec<Expresion>) -> Valor {
        let mut valores = Vec::new();
        for item in items {
            let expandido = self.expandir_item_array(item).await;
            valores.extend(expandido);
        }
        Valor::Lista(valores)
    }

    #[async_recursion]
    async fn expandir_item_array(&mut self, item: Expresion) -> Vec<Valor> {
        match item {
            Expresion::Spread(expr) => {
                let valor = self.evaluar_expresion(*expr).await;
                match valor {
                    Valor::Lista(elementos) => elementos,
                    otro => vec![otro],
                }
            }
            _ => vec![self.evaluar_expresion(item).await],
        }
    }

    #[async_recursion]
    async fn evaluar_objeto(&mut self, pares: Vec<(String, Expresion)>) -> Valor {
        let mut mapa = HashMap::new();
        for (clave, valor_expr) in pares {
            let valor = self.evaluar_expresion(valor_expr).await;
            mapa.insert(clave, valor);
        }
        Valor::Diccionario(mapa)
    }

    #[async_recursion]
    async fn evaluar_binaria(&mut self, izq: Expresion, op: &str, der: Expresion) -> Valor {
        let izquierda = self.evaluar_expresion(izq).await;
        // Logical operators short-circuit
        if op == "&&" {
            return Valor::Booleano(
                izquierda.es_verdadero() && self.evaluar_expresion(der).await.es_verdadero(),
            );
        }
        if op == "||" {
            return Valor::Booleano(
                izquierda.es_verdadero() || self.evaluar_expresion(der).await.es_verdadero(),
            );
        }

        let derecha = self.evaluar_expresion(der).await;

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
            _ => match op {
                "&&" => Valor::Booleano(izquierda.es_verdadero() && derecha.es_verdadero()),
                "||" => Valor::Booleano(izquierda.es_verdadero() || derecha.es_verdadero()),
                _ => {
                    eprintln!("Operador binario desconocido: {}", op);
                    Valor::Nulo
                }
            },
        }
    }

    #[async_recursion]
    async fn evaluar_unaria(&mut self, op: &str, expr: Expresion) -> Valor {
        match op {
            "!" => self.evaluar_negacion(expr).await,
            "-" => self.evaluar_negativo(expr).await,
            "++" => self.evaluar_incremento(expr).await,
            "--" => self.evaluar_decremento(expr).await,
            _ => self.evaluar_expresion(expr).await,
        }
    }

    #[async_recursion]
    async fn evaluar_negacion(&mut self, expr: Expresion) -> Valor {
        let valor = self.evaluar_expresion(expr).await;
        Valor::Booleano(!valor.es_verdadero())
    }

    #[async_recursion]
    async fn evaluar_negativo(&mut self, expr: Expresion) -> Valor {
        let valor = self.evaluar_expresion(expr).await;
        match valor {
            Valor::Entero(i) => Valor::Entero(-i),
            Valor::Flotante(f) => Valor::Flotante(-f),
            _ => Valor::Nulo,
        }
    }

    #[async_recursion]
    async fn evaluar_incremento(&mut self, expr: Expresion) -> Valor {
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

    #[async_recursion]
    async fn evaluar_decremento(&mut self, expr: Expresion) -> Valor {
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

    #[async_recursion]
    async fn ejecutar_if(&mut self, if_stmt: If) -> Option<Valor> {
        let condicion = self.evaluar_expresion(if_stmt.condicion).await;

        if condicion.es_verdadero() {
            return self.ejecutar_bloque(if_stmt.bloque_entonces).await;
        }

        let resultado_elseif = self.ejecutar_elseifs(if_stmt.else_ifs).await;
        if resultado_elseif.is_some() {
            return resultado_elseif;
        }

        if let Some(bloque) = if_stmt.bloque_else {
            self.ejecutar_bloque(bloque).await
        } else {
            None
        }
    }

    #[async_recursion]
    async fn ejecutar_elseifs(&mut self, else_ifs: Vec<ElseIf>) -> Option<Valor> {
        for else_if in else_ifs {
            let cond = self.evaluar_expresion(else_if.condicion).await;
            if cond.es_verdadero() {
                return self.ejecutar_bloque(else_if.bloque).await;
            }
        }
        None
    }

    #[async_recursion]
    async fn ejecutar_bloque(&mut self, bloque: Vec<Sentencia>) -> Option<Valor> {
        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));

        let mut resultado = None;
        for sentencia in bloque {
            if self.estado_excepcion.is_some() {
                break;
            }

            if let Some(valor) = self.ejecutar_sentencia(sentencia).await {
                resultado = Some(valor);
                break;
            }

            if self.estado_excepcion.is_some() {
                break;
            }
        }

        if let Some(parent) = self.entorno_actual.parent.take() {
            self.entorno_actual = *parent;
        }

        resultado
    }

    async fn ejecutar_switch(&mut self, switch: Switch) -> Option<Valor> {
        let valor_switch = self.evaluar_expresion(switch.expresion).await;

        let resultado_caso = self.ejecutar_casos(&valor_switch, switch.casos).await;
        if resultado_caso.is_some() {
            return resultado_caso;
        }

        if let Some(bloque) = switch.default {
            self.ejecutar_bloque(bloque).await
        } else {
            None
        }
    }

    async fn ejecutar_casos(&mut self, valor_switch: &Valor, casos: Vec<Case>) -> Option<Valor> {
        for caso in casos {
            let valor_caso = self.evaluar_expresion(caso.valor).await;

            if self.son_iguales(valor_switch, &valor_caso) {
                return self.ejecutar_bloque(caso.bloque).await;
            }
        }
        None
    }

    #[async_recursion]
    async fn ejecutar_for(&mut self, for_loop: For) -> Option<Valor> {
        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));

        self.ejecutar_sentencia(*for_loop.inicializacion).await;

        loop {
            if !self
                .evaluar_expresion(for_loop.condicion.clone())
                .await
                .es_verdadero()
            {
                break;
            }

            if let Some(valor) = self.ejecutar_bloque(for_loop.bloque.clone()).await {
                if let Some(parent) = self.entorno_actual.parent.take() {
                    self.entorno_actual = *parent;
                }
                return Some(valor);
            }

            self.evaluar_expresion(for_loop.incremento.clone()).await;
        }

        if let Some(parent) = self.entorno_actual.parent.take() {
            self.entorno_actual = *parent;
        }
        None
    }

    #[async_recursion]
    async fn ejecutar_foreach(&mut self, foreach: ForEach) -> Option<Valor> {
        let Valor::Lista(items) = self.evaluar_expresion(foreach.iterable).await else {
            return None;
        };

        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));

        for item in items {
            self.entorno_actual
                .definir_variable(foreach.variable.clone(), item);

            if let Some(valor) = self.ejecutar_bloque(foreach.bloque.clone()).await {
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

    #[async_recursion]
    async fn ejecutar_while(&mut self, while_loop: While) -> Option<Valor> {
        loop {
            let condicion = self.evaluar_expresion(while_loop.condicion.clone()).await;
            if !condicion.es_verdadero() {
                break;
            }

            if let Some(valor) = self.ejecutar_bloque(while_loop.bloque.clone()).await {
                return Some(valor);
            }
        }

        None
    }

    #[async_recursion]
    async fn ejecutar_do_while(&mut self, do_while: DoWhile) -> Option<Valor> {
        loop {
            if let Some(valor) = self.ejecutar_bloque(do_while.bloque.clone()).await {
                return Some(valor);
            }

            let condicion = self.evaluar_expresion(do_while.condicion.clone()).await;
            if !condicion.es_verdadero() {
                break;
            }
        }

        None
    }

    async fn evaluar_llamado_funcion(&mut self, llamado: &LlamadoFuncion) -> Valor {
        let mut argumentos = Vec::new();
        for arg in &llamado.argumentos {
            argumentos.push(self.evaluar_expresion(arg.clone()).await);
        }

        if self.es_funcion_builtin(&llamado.nombre) {
            return self
                .ejecutar_funcion_builtin(&llamado.nombre, argumentos)
                .await;
        }

        self.ejecutar_funcion_usuario(&llamado.nombre, argumentos)
            .await
    }

    fn es_funcion_builtin(&self, nombre: &str) -> bool {
        nombre == "tprint"
    }

    async fn ejecutar_funcion_builtin(&mut self, nombre: &str, argumentos: Vec<Valor>) -> Valor {
        match nombre {
            "tprint" => {
                for arg in argumentos {
                    self.tprint(arg).await;
                }
                Valor::Nulo
            }
            _ => Valor::Nulo,
        }
    }

    async fn ejecutar_funcion_usuario(&mut self, nombre: &str, argumentos: Vec<Valor>) -> Valor {
        let Some(valor_funcion) = self.entorno_actual.obtener(nombre) else {
            eprintln!("Función '{}' no encontrada", nombre);
            return Valor::Nulo;
        };

        match valor_funcion {
            Valor::Funcion(func) => {
                if func.es_async {
                    let mut interpreter_clone = self.clone();
                    let func_clone = func.clone();
                    let args_clone = argumentos.clone();

                    let handle = tokio::spawn(async move {
                        GestorFunciones::ejecutar_funcion(
                            &func_clone,
                            args_clone,
                            &mut interpreter_clone,
                        )
                        .await
                    });

                    let promesa =
                        crate::runtime::valores::SharedPromesa(Arc::new(Mutex::new(Some(handle))));
                    Valor::Promesa(promesa)
                } else {
                    self.valor_retorno = None;
                    let resultado =
                        GestorFunciones::ejecutar_funcion(&func, argumentos, self).await;
                    self.valor_retorno = None;
                    resultado
                }
            }
            Valor::FuncionNativa(_, native_fn) => native_fn(argumentos),
            _ => {
                eprintln!("'{}' no es una función", nombre);
                Valor::Nulo
            }
        }
    }

    async fn evaluar_instanciacion(&mut self, tipo: &str, argumentos: Vec<Expresion>) -> Valor {
        let args = self.evaluar_argumentos(argumentos).await;

        if self.es_funcion_builtin_instanciacion(tipo) {
            return self.ejecutar_builtin_instanciacion(tipo, args).await;
        }

        if let Some(resultado) = self
            .intentar_ejecutar_como_funcion(tipo, args.clone())
            .await
        {
            return resultado;
        }

        self.crear_y_inicializar_instancia(tipo, args).await
    }

    async fn evaluar_argumentos(&mut self, argumentos: Vec<Expresion>) -> Vec<Valor> {
        let mut valores = Vec::new();
        for arg in argumentos {
            valores.push(self.evaluar_expresion(arg).await);
        }
        valores
    }

    fn es_funcion_builtin_instanciacion(&self, tipo: &str) -> bool {
        tipo == "tprint"
    }

    async fn ejecutar_builtin_instanciacion(&mut self, tipo: &str, args: Vec<Valor>) -> Valor {
        match tipo {
            "tprint" => {
                for arg in args {
                    self.tprint(arg).await;
                }
                Valor::Nulo
            }
            _ => Valor::Nulo,
        }
    }

    async fn intentar_ejecutar_como_funcion(
        &mut self,
        tipo: &str,
        args: Vec<Valor>,
    ) -> Option<Valor> {
        let Valor::Funcion(func) = self.entorno_actual.obtener(tipo)? else {
            return None;
        };

        self.valor_retorno = None;
        let resultado = GestorFunciones::ejecutar_funcion(&func, args, self).await;
        self.valor_retorno = None;
        Some(resultado)
    }

    async fn crear_y_inicializar_instancia(&mut self, tipo: &str, args: Vec<Valor>) -> Valor {
        let Some(mut instancia) = self.gestor_clases.crear_instancia(tipo) else {
            eprintln!("Clase '{}' no encontrada", tipo);
            return Valor::Nulo;
        };

        self.ejecutar_constructor_si_existe(tipo, &args, &mut instancia)
            .await;
        Valor::Objeto(instancia)
    }

    async fn ejecutar_constructor_si_existe(
        &mut self,
        tipo: &str,
        args: &[Valor],
        instancia: &mut crate::runtime::valores::Instancia,
    ) {
        let Some(constructor) = self.obtener_constructor(tipo) else {
            return;
        };

        self.ejecutar_constructor(constructor, args, instancia)
            .await;
    }

    async fn ejecutar_constructor(
        &mut self,
        constructor: umbral_parser::ast::Metodo,
        args: &[Valor],
        instancia: &mut crate::runtime::valores::Instancia,
    ) {
        self.crear_entorno_constructor(instancia);
        self.vincular_parametros_constructor(&constructor.parametros, args);
        self.ejecutar_cuerpo_constructor(constructor.cuerpo, instancia)
            .await;
        self.restaurar_entorno();
    }

    fn crear_entorno_constructor(&mut self, instancia: &crate::runtime::valores::Instancia) {
        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));
        self.entorno_actual
            .definir_variable("__this__".to_string(), Valor::Objeto(instancia.clone()));
    }

    fn restaurar_entorno(&mut self) {
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

    async fn ejecutar_cuerpo_constructor(
        &mut self,
        cuerpo: Vec<umbral_parser::ast::Sentencia>,
        instancia: &mut crate::runtime::valores::Instancia,
    ) {
        for sentencia in cuerpo {
            self.ejecutar_sentencia(sentencia).await;
        }

        if let Some(Valor::Objeto(inst_actualizada)) = self.entorno_actual.obtener("__this__") {
            *instancia = inst_actualizada;
        }
    }

    fn acceder_propiedad_objeto(
        &mut self,
        instancia: &crate::runtime::valores::Instancia,
        propiedad: &str,
    ) -> Valor {
        if let Ok(props) = instancia.propiedades.lock() {
            if let Some(valor) = props.get(propiedad) {
                return valor.clone();
            }
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

        let funcion = Funcion::nueva(
            propiedad.to_string(),
            parametros,
            metodo.cuerpo.clone(),
            metodo.es_async,
        );

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

    async fn evaluar_acceso_propiedad(&mut self, objeto: Expresion, propiedad: &str) -> Valor {
        let obj_valor = self.evaluar_expresion(objeto.clone()).await;

        match obj_valor {
            Valor::Objeto(ref instancia) => self.acceder_propiedad_objeto(instancia, propiedad),
            Valor::Diccionario(mapa) => self.acceder_clave_diccionario(mapa, propiedad),
            Valor::Lista(ref items) if propiedad == "length" => Valor::Entero(items.len() as i64),
            _ => self.error_acceso_propiedad_invalido(propiedad, &obj_valor),
        }
    }

    async fn evaluar_acceso_indice(&mut self, objeto: Expresion, indice: Expresion) -> Valor {
        let obj_valor = self.evaluar_expresion(objeto).await;
        let indice_valor = self.evaluar_expresion(indice).await;

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

    #[async_recursion]
    async fn evaluar_llamado_metodo(
        &mut self,
        objeto: Expresion,
        metodo: &str,
        argumentos: Vec<Expresion>,
    ) -> Valor {
        let obj_valor = self.evaluar_expresion(objeto).await;

        if let Valor::Lista(ref items) = obj_valor {
            match metodo {
                "push" => {
                    if argumentos.is_empty() {
                        eprintln!("push() requiere al menos un argumento");
                        return Valor::Nulo;
                    }
                    let mut nueva_lista = items.clone();
                    for arg in argumentos {
                        let valor = self.evaluar_expresion(arg).await;
                        nueva_lista.push(valor);
                    }
                    return Valor::Lista(nueva_lista);
                }
                "pop" => {
                    if !items.is_empty() {
                        let mut nueva_lista = items.clone();
                        nueva_lista.pop();
                        return Valor::Lista(nueva_lista);
                    }
                    return Valor::Lista(vec![]);
                }
                "len" => {
                    return Valor::Entero(items.len() as i64);
                }
                _ => {
                    eprintln!("Método '{}' no existe para arreglos", metodo);
                    return Valor::Nulo;
                }
            }
        }

        if let Valor::Diccionario(mapa) = obj_valor {
            if let Some(funcion_val) = mapa.get(metodo) {
                let mut args = Vec::new();
                for arg in argumentos {
                    args.push(self.evaluar_expresion(arg).await);
                }

                return match funcion_val {
                    Valor::FuncionNativa(_, native_fn) => native_fn(args),
                    Valor::Clase(nombre_clase) => {
                        self.crear_y_inicializar_instancia(nombre_clase, args).await
                    }
                    Valor::Funcion(func) => {
                        self.valor_retorno = None;
                        let resultado = GestorFunciones::ejecutar_funcion(func, args, self).await;
                        self.valor_retorno = None;
                        resultado
                    }
                    _ => {
                        eprintln!("'{}' no es una función", metodo);
                        Valor::Nulo
                    }
                };
            } else {
                eprintln!("Método '{}' no encontrado en el diccionario", metodo);
                return Valor::Nulo;
            }
        }

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

        self.ejecutar_metodo_instancia_impl(instancia, metodo, argumentos)
            .await
    }

    #[async_recursion]
    async fn ejecutar_metodo_instancia_impl(
        &mut self,
        instancia: crate::runtime::valores::Instancia,
        metodo: &str,
        argumentos: Vec<Expresion>,
    ) -> Valor {
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

        let args = self.evaluar_argumentos(argumentos).await;

        if metodo_def.es_async {
            let mut interpreter_clone = self.clone();
            let metodo_clone = metodo_def.clone();
            let instancia_clone = instancia.clone();
            let args_clone = args.clone();

            let handle = tokio::spawn(async move {
                interpreter_clone
                    .ejecutar_metodo_clase(metodo_clone, instancia_clone, args_clone)
                    .await
            });

            let promesa =
                crate::runtime::valores::SharedPromesa(Arc::new(Mutex::new(Some(handle))));
            Valor::Promesa(promesa)
        } else {
            self.ejecutar_metodo_clase(metodo_def, instancia, args)
                .await
        }
    }

    async fn ejecutar_metodo_clase(
        &mut self,
        metodo_def: umbral_parser::ast::Metodo,
        instancia: crate::runtime::valores::Instancia,
        args: Vec<Valor>,
    ) -> Valor {
        self.preparar_entorno_metodo(&instancia, &metodo_def.parametros, &args);
        let resultado = self.ejecutar_cuerpo_metodo(metodo_def.cuerpo).await;
        self.restaurar_entorno();
        self.valor_retorno = None;
        resultado
    }

    fn preparar_entorno_metodo(
        &mut self,
        instancia: &crate::runtime::valores::Instancia,
        parametros: &[umbral_parser::ast::Parametro],
        args: &[Valor],
    ) {
        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));
        self.entorno_actual
            .definir_variable("__this__".to_string(), Valor::Objeto(instancia.clone()));

        for (i, param) in parametros.iter().enumerate() {
            if let Some(valor) = args.get(i) {
                self.entorno_actual
                    .definir_variable(param.nombre.clone(), valor.clone());
            }
        }
        self.valor_retorno = None;
    }

    async fn ejecutar_cuerpo_metodo(
        &mut self,
        cuerpo: Vec<umbral_parser::ast::Sentencia>,
    ) -> Valor {
        for sentencia in cuerpo {
            if let Some(valor) = self.ejecutar_sentencia(sentencia).await {
                return valor;
            }
        }
        Valor::Nulo
    }

    async fn tprint(&mut self, valor: Valor) {
        let salida = self.convertir_a_texto(valor).await;
        println!("{}", salida);
    }

    #[async_recursion]
    async fn convertir_a_texto(&mut self, valor: Valor) -> String {
        match valor {
            Valor::Texto(t) => self.procesar_texto(t).await,
            Valor::Entero(e) => e.to_string(),
            Valor::Flotante(f) => f.to_string(),
            Valor::Booleano(b) => self.booleano_a_texto(b),
            Valor::Lista(l) => self.lista_a_texto(l).await,
            Valor::Diccionario(m) => self.diccionario_a_texto(m).await,
            Valor::Objeto(o) => o.to_string(),
            Valor::Nulo => "null".to_string(),
            _ => "<valor no imprimible>".to_string(),
        }
    }

    fn booleano_a_texto(&self, valor: bool) -> String {
        if valor { "true" } else { "false" }.to_string()
    }

    #[async_recursion]
    async fn lista_a_texto(&mut self, items: Vec<Valor>) -> String {
        let mut elementos = Vec::new();
        for v in items {
            elementos.push(self.convertir_a_texto(v).await);
        }
        format!("[{}]", elementos.join(", "))
    }

    #[async_recursion]
    async fn diccionario_a_texto(&mut self, mapa: HashMap<String, Valor>) -> String {
        let mut pares = Vec::new();
        for (k, v) in mapa {
            let val_str = self.convertir_a_texto(v).await;
            pares.push(format!("\"{}\": {}", k, val_str));
        }
        format!("{{{}}}", pares.join(", "))
    }

    async fn procesar_texto(&mut self, texto: String) -> String {
        if self.es_texto_multilinea(&texto) {
            return self.procesar_texto_multilinea(texto).await;
        }

        if self.es_texto_con_comillas_dobles(&texto) {
            return self.procesar_texto_comillas_dobles(texto).await;
        }

        texto.trim_matches('\'').to_string()
    }

    fn es_texto_multilinea(&self, texto: &str) -> bool {
        texto.starts_with("'''") && texto.ends_with("'''")
    }

    fn es_texto_con_comillas_dobles(&self, texto: &str) -> bool {
        texto.starts_with('"') && texto.ends_with('"')
    }

    async fn procesar_texto_multilinea(&mut self, texto: String) -> String {
        let contenido = texto.trim_matches('\'').to_string();
        let normalizado = self.normalizar_multilinea(contenido);
        self.procesar_interpolaciones(normalizado).await
    }

    async fn procesar_texto_comillas_dobles(&mut self, texto: String) -> String {
        let contenido = texto.trim_matches('"').to_string();
        self.procesar_interpolaciones(contenido).await
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

    #[async_recursion]
    async fn procesar_interpolaciones(&mut self, texto: String) -> String {
        let mut salida = String::new();
        let mut chars = texto.chars().peekable();

        while let Some(c) = chars.next() {
            self.procesar_caracter_interpolacion(c, &mut chars, &mut salida)
                .await;
        }

        salida
    }

    #[async_recursion]
    async fn procesar_caracter_interpolacion(
        &mut self,
        caracter: char,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        salida: &mut String,
    ) {
        if self.es_escape_interpolacion(caracter, chars) {
            self.agregar_escape_interpolacion(chars, salida);
            return;
        }

        if caracter != '&' {
            salida.push(caracter);
            return;
        }

        self.evaluar_y_agregar_interpolacion(chars, salida).await;
    }

    fn es_escape_interpolacion(
        &self,
        caracter: char,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> bool {
        caracter == '\\' && chars.peek() == Some(&'&')
    }

    fn agregar_escape_interpolacion(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        salida: &mut String,
    ) {
        chars.next();
        salida.push('&');
    }

    #[async_recursion]
    async fn evaluar_y_agregar_interpolacion(
        &mut self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        salida: &mut String,
    ) {
        let expr = self.leer_expresion_interpolacion(chars);

        if expr.is_empty() {
            salida.push('&');
            return;
        }

        let valor = self.evaluar_interpolacion(expr).await;
        salida.push_str(&valor);
    }

    fn leer_expresion_interpolacion(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> String {
        let mut expr = String::new();
        let mut nivel_parentesis = 0;
        let mut nivel_brackets = 0;

        while let Some(&ch) = chars.peek() {
            if self.procesar_parentesis_apertura(ch, &mut nivel_parentesis, &mut expr, chars) {
                continue;
            }

            if ch == ')' {
                if self.procesar_parentesis_cierre(ch, &mut nivel_parentesis, &mut expr, chars) {
                    if nivel_brackets == 0 {
                        break;
                    }
                }
                continue;
            }

            if ch == '[' {
                nivel_brackets += 1;
                expr.push(chars.next().unwrap());
                continue;
            }

            if ch == ']' {
                if nivel_brackets > 0 {
                    nivel_brackets -= 1;
                    expr.push(chars.next().unwrap());
                    continue;
                }
                break;
            }

            if nivel_parentesis > 0 {
                expr.push(chars.next().unwrap());
                continue;
            }

            if self.procesar_punto_acceso(ch, chars, &mut expr) {
                continue;
            }

            if !self.es_caracter_expresion_basico(ch) {
                break;
            }

            expr.push(chars.next().unwrap());
        }
        expr
    }

    fn procesar_parentesis_apertura(
        &self,
        caracter: char,
        nivel: &mut i32,
        expr: &mut String,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> bool {
        if caracter != '(' {
            return false;
        }

        *nivel += 1;
        expr.push(chars.next().unwrap());
        true
    }

    fn procesar_parentesis_cierre(
        &self,
        caracter: char,
        nivel: &mut i32,
        expr: &mut String,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> bool {
        if caracter != ')' {
            return false;
        }

        if *nivel == 0 {
            return true;
        }

        *nivel -= 1;
        expr.push(chars.next().unwrap());

        if *nivel == 0 && chars.peek() != Some(&'.') {
            return true;
        }

        false
    }

    fn procesar_punto_acceso(
        &self,
        caracter: char,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        expr: &mut String,
    ) -> bool {
        if caracter != '.' {
            return false;
        }

        let mut temp_chars = chars.clone();
        temp_chars.next();

        let es_acceso_valido = temp_chars
            .peek()
            .map(|&ch| ch.is_alphanumeric() || ch == '_')
            .unwrap_or(false);

        if !es_acceso_valido {
            return false;
        }

        expr.push(chars.next().unwrap());
        true
    }

    fn es_caracter_expresion_basico(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    #[async_recursion]
    async fn evaluar_interpolacion(&mut self, expr: String) -> String {
        let valor = self.parsear_expresion_interpolacion(expr).await;
        self.convertir_a_texto(valor).await
    }

    async fn parsear_expresion_interpolacion(&mut self, expr: String) -> Valor {
        if self.es_literal_numerico(&expr) {
            return self.parsear_entero(&expr);
        }

        if expr.contains('(') {
            return self.resolver_llamada_metodo_interpolacion(&expr).await;
        }

        if expr.contains('.') || expr.contains('[') {
            return self.resolver_acceso_encadenado(&expr).await;
        }

        let nombre_variable = if expr == "th" { "__this__" } else { &expr };

        self.entorno_actual
            .obtener(nombre_variable)
            .unwrap_or(Valor::Nulo)
    }

    #[async_recursion]
    async fn ejecutar_try_catch(&mut self, stmt: TryCatch) -> Option<Valor> {
        self.ejecutar_bloque(stmt.bloque_try).await;

        if let Some(error) = self.estado_excepcion.take() {
            if let Some(catch) = stmt.bloque_catch {
                let coincide = if let Some(ref tipo_error) = catch.tipo {
                    match &error {
                        Valor::Objeto(inst) => inst.clase == *tipo_error,
                        _ => false,
                    }
                } else {
                    true
                };

                if coincide {
                    let anterior =
                        std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
                    self.entorno_actual = Entorno::nuevo(Some(anterior));

                    self.entorno_actual.definir_variable(catch.variable, error);

                    let mut resultado = None;
                    for sentencia in catch.bloque {
                        if self.estado_excepcion.is_some() {
                            break;
                        }
                        if let Some(valor) = self.ejecutar_sentencia(sentencia).await {
                            resultado = Some(valor);
                            break;
                        }
                        if self.estado_excepcion.is_some() {
                            break;
                        }
                    }

                    if let Some(parent) = self.entorno_actual.parent.take() {
                        self.entorno_actual = *parent;
                    }

                    if resultado.is_some() {
                        self.valor_retorno = resultado.clone();
                    }
                } else {
                    self.estado_excepcion = Some(error);
                }
            } else {
                self.estado_excepcion = Some(error);
            }
        }

        if let Some(finally_block) = stmt.bloque_finally {
            let excepcion_pendiente = self.estado_excepcion.take();
            let retorno_pendiente = self.valor_retorno.take();

            self.ejecutar_bloque(finally_block).await;

            if self.estado_excepcion.is_none() {
                self.estado_excepcion = excepcion_pendiente;
            }
            if self.valor_retorno.is_none() {
                self.valor_retorno = retorno_pendiente;
            }
        }

        None
    }

    #[async_recursion]
    async fn ejecutar_throw(&mut self, stmt: Throw) -> Option<Valor> {
        let valor = self.evaluar_expresion(stmt.valor).await;
        self.estado_excepcion = Some(valor);
        None
    }

    fn es_literal_numerico(&self, expr: &str) -> bool {
        expr.chars().all(|c| c.is_digit(10))
    }

    fn parsear_entero(&self, expr: &str) -> Valor {
        Valor::Entero(expr.parse::<i64>().unwrap_or(0))
    }

    #[async_recursion]
    async fn resolver_llamada_metodo_interpolacion(&mut self, expr: &str) -> Valor {
        let partes: Vec<&str> = expr.split('.').collect();
        let primer_parte = partes[0];

        let mut valor_actual = if primer_parte.contains('(') {
            let (nombre_func, args_str) = self.extraer_metodo_argumentos(primer_parte);
            let args = self.parsear_argumentos_interpolacion(args_str);

            if let Some(res) = self.intentar_ejecutar_como_funcion(nombre_func, args).await {
                res
            } else {
                return Valor::Nulo;
            }
        } else {
            let primer_elemento = self.obtener_nombre_inicial(primer_parte);
            match self.entorno_actual.obtener(primer_elemento) {
                Some(v) => v,
                None => return Valor::Nulo,
            }
        };

        for &parte in &partes[1..] {
            valor_actual = self.procesar_parte_cadena(valor_actual, parte).await;

            if matches!(valor_actual, Valor::Nulo) {
                break;
            }
        }

        valor_actual
    }

    fn obtener_nombre_inicial<'a>(&self, parte: &'a str) -> &'a str {
        if parte == "th" {
            "__this__"
        } else {
            parte
        }
    }

    #[async_recursion]
    async fn procesar_parte_cadena(&mut self, valor: Valor, parte: &str) -> Valor {
        if !parte.contains('(') {
            return self.navegar_propiedad(valor, parte);
        }

        let (metodo, args_str) = self.extraer_metodo_argumentos(parte);
        self.ejecutar_metodo_interpolacion(valor, metodo, args_str)
            .await
    }

    fn extraer_metodo_argumentos<'a>(&self, parte: &'a str) -> (&'a str, &'a str) {
        let metodo_fin = parte.find('(').unwrap_or(parte.len());
        let metodo = &parte[..metodo_fin];

        let args_str = if parte.contains('(') && parte.contains(')') {
            let inicio = parte.find('(').unwrap() + 1;
            let fin = parte.rfind(')').unwrap();
            &parte[inicio..fin]
        } else {
            ""
        };

        (metodo, args_str)
    }

    #[async_recursion]
    async fn ejecutar_metodo_interpolacion(
        &mut self,
        valor: Valor,
        metodo: &str,
        args_str: &str,
    ) -> Valor {
        match valor {
            Valor::Lista(ref items) => self.ejecutar_metodo_lista(items, metodo, args_str),
            Valor::Objeto(ref instancia) => {
                let argumentos = self.parsear_argumentos_interpolacion(args_str);
                self.ejecutar_metodo_objeto(instancia, metodo, argumentos)
                    .await
            }
            _ => Valor::Nulo,
        }
    }

    fn ejecutar_metodo_lista(&mut self, items: &[Valor], metodo: &str, args_str: &str) -> Valor {
        match metodo {
            "len" => Valor::Entero(items.len() as i64),
            "push" => self.ejecutar_push_lista(items, args_str),
            "pop" => self.ejecutar_pop_lista(items),
            _ => Valor::Nulo,
        }
    }

    fn ejecutar_push_lista(&mut self, items: &[Valor], args_str: &str) -> Valor {
        if args_str.is_empty() {
            return Valor::Lista(items.to_vec());
        }

        let mut nueva_lista = items.to_vec();
        let arg_valor = self.parsear_argumento_simple(args_str);
        nueva_lista.push(arg_valor);
        Valor::Lista(nueva_lista)
    }

    fn ejecutar_pop_lista(&self, items: &[Valor]) -> Valor {
        if items.is_empty() {
            return Valor::Lista(vec![]);
        }

        let mut nueva_lista = items.to_vec();
        nueva_lista.pop();
        Valor::Lista(nueva_lista)
    }

    fn parsear_argumentos_interpolacion(&mut self, args_str: &str) -> Vec<Valor> {
        if args_str.is_empty() {
            return vec![];
        }

        args_str
            .split(',')
            .map(|s| self.parsear_argumento_simple(s.trim()))
            .collect()
    }

    #[async_recursion]
    async fn ejecutar_metodo_objeto(
        &mut self,
        instancia: &crate::runtime::valores::Instancia,
        metodo: &str,
        args: Vec<Valor>,
    ) -> Valor {
        let clase = match self.gestor_clases.obtener_clase(&instancia.clase) {
            Some(c) => c,
            None => return Valor::Nulo,
        };

        let metodo_def = match clase.obtener_metodo(metodo) {
            Some(m) => m.clone(),
            None => return Valor::Nulo,
        };

        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));

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
            if let Some(valor) = self.ejecutar_sentencia(sentencia).await {
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

    fn parsear_argumento_simple(&self, arg: &str) -> Valor {
        let arg_limpio = arg.trim();

        if let Ok(n) = arg_limpio.parse::<i64>() {
            return Valor::Entero(n);
        }

        if let Ok(f) = arg_limpio.parse::<f64>() {
            return Valor::Flotante(f);
        }

        if (arg_limpio.starts_with('"') && arg_limpio.ends_with('"'))
            || (arg_limpio.starts_with('\'') && arg_limpio.ends_with('\''))
        {
            return Valor::Texto(arg_limpio[1..arg_limpio.len() - 1].to_string());
        }

        if arg_limpio == "true" {
            return Valor::Booleano(true);
        }
        if arg_limpio == "false" {
            return Valor::Booleano(false);
        }

        self.entorno_actual
            .obtener(arg_limpio)
            .unwrap_or(Valor::Nulo)
    }

    async fn resolver_acceso_encadenado(&mut self, expr: &str) -> Valor {
        let partes = self.tokenizar_encadenamiento(expr);
        if partes.is_empty() {
            return Valor::Nulo;
        }

        let primer_elemento = if partes[0] == "th" {
            "__this__"
        } else {
            partes[0].as_str()
        };

        let Some(mut valor_actual) = self.entorno_actual.obtener(primer_elemento) else {
            return Valor::Nulo;
        };

        for parte in &partes[1..] {
            if parte.starts_with('[') && parte.ends_with(']') {
                let indice_str = &parte[1..parte.len() - 1];
                let indice_valor = self.parsear_argumento_simple(indice_str);

                if let Valor::Lista(items) = valor_actual {
                    if let Valor::Entero(idx) = indice_valor {
                        valor_actual = self.acceder_elemento_lista(items, idx);
                    } else {
                        return Valor::Nulo;
                    }
                } else {
                    return Valor::Nulo;
                }
            } else {
                valor_actual = self.navegar_propiedad(valor_actual, parte);
            }

            if matches!(valor_actual, Valor::Nulo) {
                break;
            }
        }

        valor_actual
    }

    fn tokenizar_encadenamiento(&self, expr: &str) -> Vec<String> {
        let mut partes = Vec::new();
        let mut parte_actual = String::new();
        let mut chars = expr.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '.' {
                if !parte_actual.is_empty() {
                    partes.push(parte_actual.clone());
                    parte_actual.clear();
                }
                continue;
            }

            if c == '[' {
                if !parte_actual.is_empty() {
                    partes.push(parte_actual.clone());
                    parte_actual.clear();
                }

                let mut indice = String::new();
                indice.push('[');
                while let Some(&ic) = chars.peek() {
                    if ic == ']' {
                        chars.next();
                        indice.push(']');
                        break;
                    }
                    indice.push(chars.next().unwrap());
                }
                partes.push(indice);
                continue;
            }

            parte_actual.push(c);
        }

        if !parte_actual.is_empty() {
            partes.push(parte_actual);
        }

        partes
    }

    fn navegar_propiedad(&self, valor: Valor, propiedad: &str) -> Valor {
        match valor {
            Valor::Objeto(ref inst) => {
                if let Ok(props) = inst.propiedades.lock() {
                    props.get(propiedad).cloned().unwrap_or(Valor::Nulo)
                } else {
                    Valor::Nulo
                }
            }
            Valor::Diccionario(ref mapa) => mapa.get(propiedad).cloned().unwrap_or(Valor::Nulo),
            Valor::Lista(ref items) if propiedad == "length" => Valor::Entero(items.len() as i64),
            _ => Valor::Nulo,
        }
    }
}
