use crate::runtime::clases::{Clase, GestorClases};
use crate::runtime::entorno::Entorno;
use crate::runtime::enums::GestorEnums;
use crate::runtime::funciones::GestorFunciones;
use crate::runtime::interfaces::{GestorInterfaces, Interfaz};
use crate::runtime::stdlib::http;
use crate::runtime::stdlib::json;
use crate::runtime::stdlib::net;
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
        let tipo_esperado = decl.tipo.as_ref().map(|t| t.nombre.as_str());

        if !self.validar_tipo_valor(&decl.nombre, &valor, tipo_esperado) {
            return None;
        }
        if let Some(tipo) = &decl.tipo {
            self.entorno_actual.registrar_tipo(&decl.nombre, &tipo.nombre);
        }

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
        let tipo_esperado = decl.tipo.as_ref().map(|t| t.nombre.as_str());

        if !self.validar_tipo_valor(&decl.nombre, &valor, tipo_esperado) {
            return None;
        }
        if let Some(tipo) = &decl.tipo {
            self.entorno_actual.registrar_tipo(&decl.nombre, &tipo.nombre);
        }

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
                let tipo_actual = self.entorno_actual.obtener_tipo(&nombre);
                if !self.validar_tipo_valor(&nombre, &valor, tipo_actual.as_deref()) {
                    return None;
                }
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

    fn validar_tipo_valor(&self, nombre: &str, valor: &Valor, tipo_esperado: Option<&str>) -> bool {
        let Some(tipo_esperado) = tipo_esperado else {
            return true;
        };
        if !valor.es_tipo_compatible(tipo_esperado) {
            eprintln!(
                "Error: Tipo incompatible para '{}': se esperaba '{}' pero se recibió {}.",
                nombre,
                tipo_esperado,
                valor.nombre_tipo()
            );
            return false;
        }
        true
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

    fn dividir_parametros(
        parametros: &[Parametro],
    ) -> (Vec<String>, Option<crate::runtime::valores::ParametroRest>) {
        let fijos = parametros
            .iter()
            .filter(|p| !p.es_rest)
            .map(|p| p.nombre.clone())
            .collect();
        let resto = parametros
            .iter()
            .find(|p| p.es_rest)
            .map(|p| crate::runtime::valores::ParametroRest {
                nombre: p.nombre.clone(),
                tipo: p.tipo.as_ref().map(|t| t.nombre.clone()),
            });
        (fijos, resto)
    }

    fn vincular_resto(&mut self, contexto: &str, parametros: &[Parametro], args: &[Valor]) {
        let resto_def = parametros.iter().find(|p| p.es_rest);
        let Some(def) = resto_def else { return };
        let fijos = parametros.iter().filter(|p| !p.es_rest).count();
        let resto: Vec<Valor> = args.iter().skip(fijos).cloned().collect();
        def.tipo.as_ref().map(|t| {
            crate::runtime::funciones::validar_elementos_rest(
                contexto,
                &crate::runtime::valores::ParametroRest {
                    nombre: def.nombre.clone(),
                    tipo: Some(t.nombre.clone()),
                },
                &resto,
            )
        });
        self.entorno_actual
            .definir_variable(def.nombre.clone(), Valor::Lista(resto));
    }

    fn registrar_funcion(&mut self, func: DeclaracionFuncion) -> Option<Valor> {
        let (parametros, parametro_rest) = Self::dividir_parametros(&func.parametros);
        let funcion = Funcion::con_rest(
            func.nombre.clone(),
            parametros,
            parametro_rest,
            func.cuerpo,
            func.es_async,
            func.doc.clone(),
        );
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

    fn separar_prefijo(base: &str) -> (String, &str) {
        let Some(resto) = base.strip_prefix("[]") else {
            return (String::new(), base);
        };
        let (prefijo, nucleo) = Self::separar_prefijo(resto);
        (format!("[]{}", prefijo), nucleo)
    }

    fn sustituir_tipo(tipo: &Tipo, mapa: &HashMap<String, String>) -> String {
        let (prefijo, nucleo) = Self::separar_prefijo(&tipo.base);
        if tipo.args.is_empty() {
            let res = mapa.get(nucleo).map(|s| s.as_str()).unwrap_or(nucleo);
            return format!("{}{}", prefijo, res);
        }
        let args = tipo
            .args
            .iter()
            .map(|a| Self::sustituir_tipo(a, mapa))
            .collect::<Vec<_>>()
            .join(", ");
        let base_res = mapa.get(nucleo).map(|s| s.as_str()).unwrap_or(nucleo);
        format!("{}{}<{}>", prefijo, base_res, args)
    }

    fn validar_implementaciones(&self, clase: &DeclaracionClase) {
        for referencia in &clase.implementaciones {
            self.validar_implementacion_interfaz(clase, referencia);
        }
    }

    fn validar_implementacion_interfaz(&self, clase: &DeclaracionClase, referencia: &Tipo) {
        let base = crate::runtime::valores::base_de_tipo(&referencia.base);
        let interfaz_opt = self.gestor_interfaces.obtener(base);

        if interfaz_opt.is_none() {
            eprintln!("Error: La interfaz '{}' no está definida.", referencia.nombre);
            return;
        }

        let interfaz = interfaz_opt.unwrap().clone();
        if interfaz.parametros_tipo.len() != referencia.args.len() {
            eprintln!(
                "Error: La interfaz '{}' espera {} argumento(s) genérico(s) pero '{}' le pasa {} en la clase '{}'.",
                interfaz.nombre,
                interfaz.parametros_tipo.len(),
                referencia.nombre,
                referencia.args.len(),
                clase.nombre
            );
            return;
        }

        let mapa: HashMap<String, String> = interfaz
            .parametros_tipo
            .iter()
            .cloned()
            .zip(referencia.args.iter().map(|t| t.nombre.clone()))
            .collect();

        for (nombre_metodo, metodo_interfaz) in &interfaz.metodos {
            self.validar_metodo_interfaz(clase, &referencia.nombre, nombre_metodo, metodo_interfaz, &mapa);
        }
        for (nombre_prop, prop_interfaz) in &interfaz.propiedades {
            self.validar_propiedad_interfaz(clase, &referencia.nombre, nombre_prop, prop_interfaz, &mapa);
        }
    }

    fn validar_metodo_interfaz(
        &self,
        clase: &DeclaracionClase,
        nombre_interfaz: &str,
        nombre_metodo: &str,
        metodo_interfaz: &Metodo,
        mapa: &HashMap<String, String>,
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
        let arity_ok = Self::misma_aridad(metodo_interfaz, metodo_clase);
        if !arity_ok {
            eprintln!(
                "Error: La clase '{}' implementa incorrectamente el método '{}' de la interfaz '{}'. Diferente número de parámetros.",
                clase.nombre, nombre_metodo, nombre_interfaz
            );
            return;
        }

        self.validar_fijos(metodo_clase, metodo_interfaz, &clase.nombre, nombre_interfaz, nombre_metodo, mapa);
        self.validar_retorno(metodo_clase, metodo_interfaz, &clase.nombre, nombre_interfaz, nombre_metodo, mapa);
    }

    fn misma_aridad(interfaz: &Metodo, clase: &Metodo) -> bool {
        let fijos = |m: &Metodo| m.parametros.iter().filter(|p| !p.es_rest).count();
        let tiene_rest = |m: &Metodo| m.parametros.iter().any(|p| p.es_rest);
        fijos(interfaz) == fijos(clase) && tiene_rest(interfaz) == tiene_rest(clase)
    }

    fn validar_fijos(
        &self,
        clase: &Metodo,
        interfaz: &Metodo,
        nombre_clase: &str,
        nombre_interfaz: &str,
        nombre_metodo: &str,
        mapa: &HashMap<String, String>,
    ) {
        for (pi, pc) in interfaz.parametros.iter().zip(clase.parametros.iter()) {
            let nombre = Self::nombre_parametro(pi);
            self.validar_tipo_parametro(
                nombre_clase,
                nombre_interfaz,
                nombre_metodo,
                &nombre,
                pi.tipo.as_ref().map(|t| Self::sustituir_tipo(t, mapa)),
                pc.tipo.as_ref().map(|t| t.nombre.clone()),
            );
        }
    }

    fn nombre_parametro(p: &Parametro) -> String {
        match p.es_rest {
            true => format!("...{}", p.nombre),
            false => p.nombre.clone(),
        }
    }

    fn validar_retorno(
        &self,
        clase: &Metodo,
        interfaz: &Metodo,
        nombre_clase: &str,
        nombre_interfaz: &str,
        nombre_metodo: &str,
        mapa: &HashMap<String, String>,
    ) {
        let tipos = (
            interfaz.tipo_retorno.as_ref().map(|t| Self::sustituir_tipo(t, mapa)),
            clase.tipo_retorno.as_ref().map(|t| t.nombre.clone()),
        );
        let (Some(esperado), Some(recibido)) = tipos else {
            return;
        };
        if esperado != recibido {
            eprintln!(
                "Error: La clase '{}' implementa incorrectamente el método '{}' de la interfaz '{}'. Retorno esperado '{}' pero se declaró '{}'.",
                nombre_clase, nombre_metodo, nombre_interfaz, esperado, recibido
            );
        }
    }

    fn validar_tipo_parametro(
        &self,
        clase: &str,
        interfaz: &str,
        metodo: &str,
        param: &str,
        esperado: Option<String>,
        recibido: Option<String>,
    ) {
        if let (Some(esperado), Some(recibido)) = (esperado, recibido) {
            if esperado != recibido {
                eprintln!(
                    "Error: La clase '{}' implementa incorrectamente el parámetro '{}' del método '{}' de la interfaz '{}'. Se esperaba '{}' pero se declaró '{}'.",
                    clase, param, metodo, interfaz, esperado, recibido
                );
            }
        }
    }

    fn validar_propiedad_interfaz(
        &self,
        clase: &DeclaracionClase,
        nombre_interfaz: &str,
        nombre_prop: &str,
        prop_interfaz: &umbral_parser::ast::Propiedad,
        mapa: &HashMap<String, String>,
    ) {
        let prop_clase_opt = clase.propiedades.iter().find(|p| p.nombre == *nombre_prop);

        if prop_clase_opt.is_none() {
            eprintln!(
                "Error: La clase '{}' no implementa la propiedad '{}' de la interfaz '{}'.",
                clase.nombre, nombre_prop, nombre_interfaz
            );
            return;
        }

        let prop_clase = prop_clase_opt.unwrap();
        let esperado =
            prop_interfaz.tipo.as_ref().map(|t| Self::sustituir_tipo(t, mapa));
        let recibido = prop_clase.tipo.as_ref().map(|t| t.nombre.clone());
        if let (Some(esperado), Some(recibido)) = (esperado, recibido) {
            if esperado != recibido {
                eprintln!(
                    "Error: La clase '{}' implementa incorrectamente la propiedad '{}' de la interfaz '{}'. Se esperaba '{}' pero se declaró '{}'.",
                    clase.nombre, nombre_prop, nombre_interfaz, esperado, recibido
                );
            }
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

    /// Captura los globales de un módulo ya ejecutado.
    fn captura_de_modulo(modulo: &Interpretador) -> HashMap<String, Valor> {
        modulo
            .entorno_actual
            .variables
            .iter()
            .chain(modulo.entorno_actual.constantes.iter())
            .map(|(nombre, valor)| (nombre.clone(), valor.clone()))
            .collect()
    }

    /// Adjunta la captura a funciones sueltas o dentro de diccionarios.
    fn adjuntar_captura(valor: Valor, captura: &HashMap<String, Valor>) -> Valor {
        match valor {
            Valor::Funcion(f) => Self::funcion_con_captura(f, captura),
            Valor::Diccionario(mapa) => Valor::Diccionario(Self::mapa_con_captura(mapa, captura)),
            otro => otro,
        }
    }

    fn funcion_con_captura(
        funcion: crate::runtime::valores::Funcion,
        captura: &HashMap<String, Valor>,
    ) -> Valor {
        let mut combinada: HashMap<String, Valor> = captura
            .iter()
            .filter(|(nombre, _)| *nombre != &funcion.nombre)
            .map(|(nombre, valor)| (nombre.clone(), valor.clone()))
            .collect();
        // Preserva ligaduras previas (ej. `__this__` de un método extraído
        // como callback): la instancia ligada gana sobre los globales.
        if let Some(previa) = funcion.entorno_capturado.clone() {
            previa.into_iter().for_each(|(nombre, valor)| {
                combinada.insert(nombre, valor);
            });
        }
        Valor::Funcion(funcion.con_captura(combinada))
    }

    fn mapa_con_captura(
        mapa: HashMap<String, Valor>,
        captura: &HashMap<String, Valor>,
    ) -> HashMap<String, Valor> {
        mapa.into_iter()
            .map(|(clave, valor)| (clave, Self::adjuntar_captura(valor, captura)))
            .collect()
    }

    fn es_exportado(modulo: &Interpretador, nombre: &str) -> bool {
        modulo.exportaciones.get(nombre).copied().unwrap_or(false)
    }

    fn inyectar_captura(entorno: &mut Entorno, captura: Option<HashMap<String, Valor>>) {
        let Some(mapa) = captura else {
            return;
        };
        mapa.into_iter()
            .filter(|(nombre, _)| nombre != "__this__")
            .for_each(|(nombre, valor)| {
                entorno.definir_variable(nombre, valor);
            });
    }

    fn importar_clases_transitivas(&mut self, modulo: &Interpretador) {
        let captura = Self::captura_de_modulo(modulo);
        modulo
            .gestor_clases
            .clases
            .iter()
            .filter(|(nombre, _)| !self.gestor_clases.clases.contains_key(*nombre))
            .map(|(nombre, clase)| (nombre.clone(), clase.clone().con_captura(captura.clone())))
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|(nombre, copia)| {
                self.gestor_clases.clases.insert(nombre, copia);
            });
    }

    fn importar_modulo_como_objeto(&mut self, nombre_var: String, modulo: &Interpretador) {
        let captura = Self::captura_de_modulo(modulo);
        let tabla = Self::tabla_exportada(modulo, &captura);
        self.registrar_clases_objeto(modulo, &nombre_var, &captura);
        let clases = Self::clases_exportadas(modulo, &nombre_var);
        let tabla_final = Self::unir_tablas(tabla, clases);
        self.importar_clases_transitivas(modulo);
        self.entorno_actual
            .definir_variable(nombre_var, Valor::Diccionario(tabla_final));
    }

    fn tabla_exportada(
        modulo: &Interpretador,
        captura: &HashMap<String, Valor>,
    ) -> HashMap<String, Valor> {
        modulo
            .entorno_actual
            .variables
            .iter()
            .chain(modulo.entorno_actual.constantes.iter())
            .filter(|(nombre, _)| Self::es_exportado(modulo, nombre))
            .map(|(nombre, valor)| {
                (
                    nombre.clone(),
                    Self::adjuntar_captura(valor.clone(), captura),
                )
            })
            .collect()
    }

    fn clases_exportadas(
        modulo: &Interpretador,
        nombre_var: &str,
    ) -> HashMap<String, Valor> {
        modulo
            .gestor_clases
            .clases
            .iter()
            .filter(|(nombre, _)| Self::es_exportado(modulo, nombre))
            .map(|(nombre, _)| {
                (
                    nombre.clone(),
                    Valor::Clase(format!("__modulo_{}_{}", nombre_var, nombre)),
                )
            })
            .collect()
    }

    fn unir_tablas(
        base: HashMap<String, Valor>,
        extra: HashMap<String, Valor>,
    ) -> HashMap<String, Valor> {
        base.into_iter().chain(extra).collect()
    }

    fn registrar_clases_objeto(
        &mut self,
        modulo: &Interpretador,
        nombre_var: &str,
        captura: &HashMap<String, Valor>,
    ) {
        modulo
            .gestor_clases
            .clases
            .iter()
            .filter(|(nombre, _)| Self::es_exportado(modulo, nombre))
            .map(|(nombre, clase)| {
                let unico = format!("__modulo_{}_{}", nombre_var, nombre);
                let copia = clase
                    .clone()
                    .con_nombre(unico.clone())
                    .con_captura(captura.clone());
                (unico, copia)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|(unico, copia)| {
                self.gestor_clases.clases.insert(unico, copia);
            });
    }

    fn importar_todo(&mut self, alias: Option<String>, modulo: &Interpretador) {
        let alias_nombre = alias.unwrap_or_else(|| "mod".to_string());

        self.importar_variables_exportadas(&alias_nombre, modulo);
        self.importar_clases_exportadas(&alias_nombre, modulo);
    }

    fn importar_variables_exportadas(&mut self, alias: &str, modulo: &Interpretador) {
        let captura = Self::captura_de_modulo(modulo);
        modulo
            .entorno_actual
            .variables
            .iter()
            .chain(modulo.entorno_actual.constantes.iter())
            .filter(|(nombre, _)| Self::es_exportado(modulo, nombre))
            .map(|(nombre, valor)| {
                (
                    format!("{}_{}", alias, nombre),
                    Self::adjuntar_captura(valor.clone(), &captura),
                )
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|(destino, valor)| {
                self.entorno_actual.definir_variable(destino, valor);
            });
        self.importar_clases_transitivas(modulo);
    }

    fn importar_clases_exportadas(&mut self, alias: &str, modulo: &Interpretador) {
        let captura = Self::captura_de_modulo(modulo);
        modulo
            .gestor_clases
            .clases
            .iter()
            .filter(|(nombre, _)| Self::es_exportado(modulo, nombre))
            .map(|(nombre, clase)| {
                (
                    format!("{}_{}", alias, nombre),
                    clase.clone().con_captura(captura.clone()),
                )
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|(destino, copia)| {
                self.gestor_clases.clases.insert(destino, copia);
            });
        self.importar_clases_transitivas(modulo);
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
        let Some(valor) = modulo.entorno_actual.obtener(nombre) else {
            return false;
        };
        let captura = Self::captura_de_modulo(modulo);
        let adaptado = Self::adjuntar_captura(valor, &captura);
        self.entorno_actual
            .definir_variable(nombre_final.to_string(), adaptado);
        self.importar_clases_transitivas(modulo);
        true
    }

    fn intentar_importar_clase(
        &mut self,
        nombre: &str,
        nombre_final: &str,
        modulo: &Interpretador,
    ) -> bool {
        let Some(clase) = modulo.gestor_clases.clases.get(nombre) else {
            return false;
        };
        let captura = Self::captura_de_modulo(modulo);
        let copia = clase.clone().con_captura(captura);
        self.gestor_clases
            .clases
            .insert(nombre_final.to_string(), copia);
        self.importar_clases_transitivas(modulo);
        true
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
                self.evaluar_llamada_funcion_expresion(nombre, argumentos)
                    .await
            }
            Expresion::Await(expr) => self.evaluar_await_expresion(*expr).await,
        }
    }

    #[async_recursion]
    async fn evaluar_literal_cadena(&mut self, s: String) -> Valor {
        let contenido = s.trim_matches('"').to_string();
        let interpolado = self.procesar_interpolaciones(contenido).await;
        Valor::Texto(interpolado)
    }

    #[async_recursion]
    async fn evaluar_llamada_funcion_expresion(
        &mut self,
        nombre: String,
        argumentos: Vec<Expresion>,
    ) -> Valor {
        let mut args = Vec::new();
        for arg in argumentos {
            args.extend(self.expandir_argumento(arg).await);
        }

        if self.es_funcion_builtin(&nombre) {
            return self.ejecutar_funcion_builtin(&nombre, args).await;
        }

        self.ejecutar_funcion_usuario(&nombre, args).await
    }

    #[async_recursion]
    async fn evaluar_await_expresion(&mut self, expr: Expresion) -> Valor {
        let valor = self.evaluar_expresion(expr).await;

        let Valor::Promesa(SharedPromesa(arc_mutex)) = valor else {
            return valor;
        };

        let handle_opt = {
            let mut guard = arc_mutex.lock().unwrap();
            guard.take()
        };

        let Some(handle) = handle_opt else {
            return Valor::Nulo;
        };

        match handle.await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error en tarea asincrona: {:?}", e);
                Valor::Nulo
            }
        }
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
            _ => {
                eprintln!("Operador binario desconocido: {}", op);
                Valor::Nulo
            }
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

    #[async_recursion]
    async fn expandir_argumento(&mut self, arg: Expresion) -> Vec<Valor> {
        match arg {
            Expresion::Spread(expr) => self.expandir_spread(*expr).await,
            otro => vec![self.evaluar_expresion(otro).await],
        }
    }

    #[async_recursion]
    async fn expandir_spread(&mut self, expr: Expresion) -> Vec<Valor> {
        match self.evaluar_expresion(expr).await {
            Valor::Lista(elementos) => elementos,
            otro => vec![otro],
        }
    }

    async fn evaluar_llamado_funcion(&mut self, llamado: &LlamadoFuncion) -> Valor {
        let mut argumentos = Vec::new();
        for arg in &llamado.argumentos {
            argumentos.extend(self.expandir_argumento(arg.clone()).await);
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
        nombre == "tprint" || nombre == "help" || nombre == "doc"
    }

    async fn ejecutar_funcion_builtin(&mut self, nombre: &str, argumentos: Vec<Valor>) -> Valor {
        match nombre {
            "tprint" => {
                for arg in argumentos {
                    self.tprint(arg).await;
                }
                Valor::Nulo
            }
            "help" | "doc" => {
                let texto = self.texto_ayuda_de_valores(&argumentos);
                println!("{}", texto);
                Valor::Texto(texto)
            }
            _ => Valor::Nulo,
        }
    }

    fn texto_ayuda_de_valores(&self, valores: &[Valor]) -> String {
        if valores.is_empty() {
            return self.listar_ayuda_disponible();
        }
        valores
            .iter()
            .map(|v| self.texto_ayuda_de_valor(v))
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn texto_ayuda_de_valor(&self, valor: &Valor) -> String {
        match valor {
            Valor::Funcion(f) => f.texto_ayuda(),
            Valor::FuncionNativa(n, _) => format!("{} (función nativa, sin umdocs)", n),
            Valor::Clase(nombre) => self
                .gestor_clases
                .obtener_clase(nombre)
                .map(|c| c.texto_ayuda())
                .unwrap_or_else(|| format!("Clase '{}' no encontrada", nombre)),
            Valor::Objeto(inst) => self
                .gestor_clases
                .obtener_clase(&inst.clase)
                .map(|c| c.texto_ayuda())
                .unwrap_or_else(|| format!("{} (sin documentación umdocs)", inst.clase)),
            Valor::Texto(nombre) => self.texto_ayuda_por_nombre(nombre),
            Valor::Diccionario(mapa) if http::es_respuesta_pulse(mapa) => {
                "Respuesta pulse (sin umdocs)".to_string()
            }
            otro => format!("{} (sin documentación umdocs)", otro.nombre_tipo()),
        }
    }

    fn texto_ayuda_por_nombre(&self, nombre: &str) -> String {
        let clave = nombre.trim();
        if let Some(texto) = self.buscar_en_entorno(clave) {
            return texto;
        }
        if let Some(texto) = self.buscar_en_clases(clave) {
            return texto;
        }
        if let Some(texto) = self.buscar_metodo_compuesto(clave) {
            return texto;
        }
        format!("'{}' no encontrado (sin documentación umdocs)", clave)
    }

    fn buscar_en_entorno(&self, clave: &str) -> Option<String> {
        let valor = self.entorno_actual.obtener(clave)?;
        Some(self.texto_ayuda_de_valor(&valor))
    }

    fn buscar_en_clases(&self, clave: &str) -> Option<String> {
        let clase = self.gestor_clases.obtener_clase(clave)?;
        Some(clase.texto_ayuda())
    }

    fn buscar_metodo_compuesto(&self, clave: &str) -> Option<String> {
        let (clase_n, metodo_n) = clave.split_once('.')?;
        let clase = self.gestor_clases.obtener_clase(clase_n.trim())?;
        let metodo = clase.obtener_metodo(metodo_n.trim());
        if let Some(m) = metodo {
            return Some(texto_ayuda_metodo(clase_n.trim(), m));
        }
        Some(format!(
            "Método '{}' no encontrado en clase '{}'",
            metodo_n.trim(),
            clase_n.trim()
        ))
    }

    fn listar_ayuda_disponible(&self) -> String {
        let nombres = self.nombres_ordenados();
        let docs = self.funciones_documentadas(&nombres);
        self.formatear_lista_ayuda(docs)
    }

    fn nombres_ordenados(&self) -> Vec<String> {
        let mut lista: Vec<String> = self
            .entorno_actual
            .variables
            .keys()
            .chain(self.entorno_actual.constantes.keys())
            .cloned()
            .collect();
        lista.sort();
        lista.dedup();
        lista
    }

    fn funciones_documentadas(&self, nombres: &[String]) -> Vec<String> {
        nombres
            .iter()
            .filter(|n| self.es_funcion_documentada(n))
            .cloned()
            .collect()
    }

    fn es_funcion_documentada(&self, nombre: &str) -> bool {
        let Some(v) = self.entorno_actual.obtener(nombre) else {
            return false;
        };
        matches!(v, Valor::Funcion(f) if f.doc.is_some())
    }

    fn formatear_lista_ayuda(&self, docs: Vec<String>) -> String {
        let mut lineas = vec!["Ayuda umdocs. Uso: help(funcion) | doc(funcion) | Std.doc(funcion)".to_string()];
        if docs.is_empty() {
            lineas.push("  (no hay funciones con umdocs en este ámbito)".to_string());
            return lineas.join("\n");
        }
        lineas.push("  Funciones documentadas:".to_string());
        lineas.extend(docs.iter().map(|n| format!("    - {}", n)));
        lineas.join("\n")
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
            valores.extend(self.expandir_argumento(arg).await);
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
        let lista = self.obtener_constructores(tipo);
        if lista.is_empty() {
            return;
        }
        let Some(elegido) = Self::elegir_constructor(&lista, args.len()) else {
            self.reportar_constructor(tipo, args.len(), &lista);
            return;
        };

        self.ejecutar_constructor(elegido, args, instancia)
            .await;
    }

    fn obtener_constructores(&self, tipo: &str) -> Vec<umbral_parser::ast::Metodo> {
        self.gestor_clases
            .obtener_clase(tipo)
            .map(|actual| actual.constructores.clone())
            .unwrap_or_default()
    }

    /// Elige el constructor compatible con la cantidad de argumentos.
    fn elegir_constructor(
        lista: &[umbral_parser::ast::Metodo],
        total: usize,
    ) -> Option<umbral_parser::ast::Metodo> {
        let compatibles: Vec<umbral_parser::ast::Metodo> = lista
            .iter()
            .filter(|actual| Self::es_compatible(actual, total))
            .cloned()
            .collect();
        let Some(exacto) = compatibles
            .iter()
            .find(|actual| !Self::tiene_resto(actual))
            .cloned()
        else {
            return compatibles.into_iter().next();
        };
        Some(exacto)
    }

    fn es_compatible(metodo: &umbral_parser::ast::Metodo, total: usize) -> bool {
        let fijos = Self::contar_fijos(metodo);
        Self::tiene_resto(metodo) && total >= fijos || !Self::tiene_resto(metodo) && total == fijos
    }

    fn contar_fijos(metodo: &umbral_parser::ast::Metodo) -> usize {
        metodo
            .parametros
            .iter()
            .filter(|actual| !actual.es_rest)
            .count()
    }

    fn tiene_resto(metodo: &umbral_parser::ast::Metodo) -> bool {
        metodo.parametros.iter().any(|actual| actual.es_rest)
    }

    fn reportar_constructor(
        &self,
        tipo: &str,
        total: usize,
        lista: &[umbral_parser::ast::Metodo],
    ) {
        let esperadas: Vec<String> = lista
            .iter()
            .map(|actual| Self::contar_fijos(actual).to_string())
            .collect();
        eprintln!(
            "Error: La clase '{}' no tiene constructor con {} argumento(s). Esperados: {}.",
            tipo,
            total,
            esperadas.join(", ")
        );
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

    fn captura_de_clase(&self, nombre_clase: &str) -> Option<HashMap<String, Valor>> {
        self.gestor_clases
            .obtener_clase(nombre_clase)
            .and_then(|actual| actual.entorno_capturado.clone())
    }

    fn crear_entorno_constructor(&mut self, instancia: &crate::runtime::valores::Instancia) {
        let captura = self.captura_de_clase(&instancia.clase);
        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));
        Self::inyectar_captura(&mut self.entorno_actual, captura);
        self.entorno_actual
            .definir_variable("__this__".to_string(), Valor::Objeto(instancia.clone()));
    }

    fn restaurar_entorno(&mut self) {
        if let Some(parent) = self.entorno_actual.parent.take() {
            self.entorno_actual = *parent;
        }
    }

    fn vincular_parametros_constructor(
        &mut self,
        parametros: &[umbral_parser::ast::Parametro],
        args: &[Valor],
    ) {
        let (fijos, _) = Self::dividir_parametros(parametros);
        args.iter()
            .zip(fijos.iter())
            .for_each(|(valor, nombre)| {
                self.entorno_actual
                    .definir_variable(nombre.clone(), valor.clone());
            });
        self.vincular_resto("constructor", parametros, args);
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
        let Some(valor) = Self::propiedad_de_instancia(instancia, propiedad) else {
            return self
                .buscar_metodo_como_funcion(instancia, propiedad)
                .unwrap_or_else(|| self.error_propiedad_no_encontrada(propiedad));
        };
        valor
    }

    fn propiedad_de_instancia(
        instancia: &crate::runtime::valores::Instancia,
        propiedad: &str,
    ) -> Option<Valor> {
        instancia
            .propiedades
            .lock()
            .ok()?
            .get(propiedad)
            .cloned()
    }

    fn buscar_metodo_como_funcion(
        &self,
        instancia: &crate::runtime::valores::Instancia,
        propiedad: &str,
    ) -> Option<Valor> {
        let clase = self.gestor_clases.obtener_clase(&instancia.clase)?;
        let metodo = clase.obtener_metodo(propiedad)?;

        let (parametros, parametro_rest) = Self::dividir_parametros(&metodo.parametros);
        let base = Funcion::con_rest(
            propiedad.to_string(),
            parametros,
            parametro_rest,
            metodo.cuerpo.clone(),
            metodo.es_async,
            metodo.doc.clone(),
        );
        // Liga la instancia (`th`): al extraer `ctrl.metodo` como callback,
        // la función resultante conserva su `__this__` vía captura, que
        // `GestorFunciones` inyecta al invocarla. Sin esto el contexto
        // de la clase (ej. dependencias inyectadas) se pierde.
        let mut captura = clase.entorno_capturado.clone().unwrap_or_default();
        captura.insert(
            "__this__".to_string(),
            Valor::Objeto(instancia.clone()),
        );
        Some(Valor::Funcion(base.con_captura(captura)))
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

        if es_propiedad_doc(propiedad) {
            if let Some(texto) = self.texto_doc_propiedad(&obj_valor) {
                return Valor::Texto(texto);
            }
        }

        match obj_valor {
            Valor::Objeto(ref instancia) => self.acceder_propiedad_objeto(instancia, propiedad),
            Valor::Diccionario(mapa) => self.acceder_clave_diccionario(mapa, propiedad),
            Valor::Lista(ref items) if propiedad == "length" => Valor::Entero(items.len() as i64),
            _ => self.error_acceso_propiedad_invalido(propiedad, &obj_valor),
        }
    }

    fn texto_doc_propiedad(&self, valor: &Valor) -> Option<String> {
        match valor {
            Valor::Funcion(f) => Some(f.texto_ayuda()),
            Valor::FuncionNativa(n, _) => {
                Some(format!("{} (función nativa, sin umdocs)", n))
            }
            Valor::Clase(nombre) => self
                .gestor_clases
                .obtener_clase(nombre)
                .map(|c| c.texto_ayuda()),
            Valor::Objeto(inst) => self
                .gestor_clases
                .obtener_clase(&inst.clase)
                .map(|c| c.texto_ayuda()),
            _ => None,
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
                "json" if argumentos.is_empty() => {
                    return http::valor_a_json_texto(&Valor::Lista(items.clone()));
                }
                "text" | "string" if argumentos.is_empty() => {
                    return Valor::Texto(Valor::Lista(items.clone()).to_string());
                }
                "parse" if argumentos.is_empty() => {
                    return Valor::Lista(items.clone());
                }
                _ => {
                    eprintln!("Método '{}' no existe para arreglos", metodo);
                    return Valor::Nulo;
                }
            }
        }

        if let Valor::Diccionario(mapa) = obj_valor {
            if http::es_respuesta_pulse(&mapa) && argumentos.is_empty() {
                match metodo {
                    "parse" => return http::respuesta_parse(&mapa),
                    "json" => return http::respuesta_json(&mapa),
                    "text" | "string" => return http::respuesta_texto(&mapa),
                    _ => {}
                }
            }

            if let Some(funcion_val) = mapa.get(metodo) {
                let args = self.evaluar_argumentos(argumentos).await;

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
            }

            if argumentos.is_empty() && metodo == "json" {
                return http::valor_a_json_texto(&Valor::Diccionario(mapa));
            }

            if argumentos.is_empty() && (metodo == "text" || metodo == "string") {
                return Valor::Texto(Valor::Diccionario(mapa).to_string());
            }

            if argumentos.is_empty() && metodo == "parse" {
                return Valor::Diccionario(mapa);
            }

            eprintln!("Método '{}' no encontrado en el diccionario", metodo);
            return Valor::Nulo;
        }

        if let Valor::Texto(texto) = &obj_valor {
            if metodo == "parse" && argumentos.is_empty() {
                return http::texto_a_valor_umbral(texto);
            }
            if metodo == "json" && argumentos.is_empty() {
                return http::texto_a_json_canonico(texto);
            }
            if (metodo == "text" || metodo == "string") && argumentos.is_empty() {
                return Valor::Texto(texto.clone());
            }
            eprintln!("Método '{}' no existe para texto", metodo);
            return Valor::Nulo;
        }

        if let Valor::Enchufe(manejador) = obj_valor {
            let args = self.evaluar_argumentos(argumentos).await;
            return net::invocar(&manejador, metodo, args);
        }

        if argumentos.is_empty()
            && !matches!(obj_valor, Valor::Objeto(_))
            && (metodo == "text" || metodo == "string" || metodo == "json")
        {
            match metodo {
                "text" | "string" => return Valor::Texto(obj_valor.to_string()),
                "json" => return http::valor_a_json_texto(&obj_valor),
                _ => {}
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

        if argumentos.is_empty() {
            if let Some(valor) = self.json_por_defecto(&instancia, metodo) {
                return valor;
            }
        }

        self.ejecutar_metodo_instancia_impl(instancia, metodo, argumentos)
            .await
    }

    /// Conversiones por defecto de instancias:
    /// `.json()` devuelve el JSON tradicional (Texto),
    /// `.text()` / `.string()` devuelven la forma Umbral tal cual
    /// (`User(["prop" => valor])`) como Texto,
    /// `.parse()` devuelve el diccionario `["prop" => valor]`.
    /// Retorna `None` si la clase define su propio método
    /// (se respeta el override) o si no es un método de conversión.
    fn json_por_defecto(
        &self,
        instancia: &crate::runtime::valores::Instancia,
        metodo: &str,
    ) -> Option<Valor> {
        let es_conversion = matches!(metodo, "json" | "parse" | "text" | "string");
        if !es_conversion {
            return None;
        }
        let tiene_override = self
            .gestor_clases
            .obtener_clase(&instancia.clase)
            .and_then(|c| c.obtener_metodo(metodo))
            .is_some();
        if tiene_override {
            return None;
        }
        let valor = match metodo {
            "json" => http::valor_a_json_texto(&Valor::Objeto(instancia.clone())),
            "text" | "string" => {
                Valor::Texto(Valor::Objeto(instancia.clone()).to_string())
            }
            _ => json::objeto_a_diccionario(instancia),
        };
        Some(valor)
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
        let captura = self.captura_de_clase(&instancia.clase);
        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));
        Self::inyectar_captura(&mut self.entorno_actual, captura);
        self.entorno_actual
            .definir_variable("__this__".to_string(), Valor::Objeto(instancia.clone()));
        Self::vincular_fijos_metodo(&mut self.entorno_actual, parametros, args);
        self.vincular_resto("método", parametros, args);
        self.valor_retorno = None;
    }

    fn vincular_fijos_metodo(
        entorno: &mut Entorno,
        parametros: &[umbral_parser::ast::Parametro],
        args: &[Valor],
    ) {
        let (fijos, _) = Self::dividir_parametros(parametros);
        args.iter()
            .zip(fijos.iter())
            .for_each(|(valor, nombre)| {
                entorno.definir_variable(nombre.clone(), valor.clone());
            });
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
            Valor::Objeto(o) => self.objeto_a_texto(o).await,
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
            pares.push(format!("\"{}\" => {}", k, val_str));
        }
        format!("[{}]", pares.join(", "))
    }

    #[async_recursion]
    async fn objeto_a_texto(
        &mut self,
        instancia: crate::runtime::valores::Instancia,
    ) -> String {
        let pares = match instancia.propiedades.lock() {
            Ok(props) => props.clone(),
            Err(_) => return format!("{}([error])", instancia.clase),
        };
        let mut partes = Vec::new();
        for (k, v) in pares {
            let val_str = self.convertir_a_texto(v).await;
            partes.push(format!("\"{}\" => {}", k, val_str));
        }
        format!("{}([{}])", instancia.clase, partes.join(", "))
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
            Valor::Diccionario(ref mapa)
                if http::es_respuesta_pulse(mapa) && args_str.trim().is_empty() =>
            {
                match metodo {
                    "parse" => http::respuesta_parse(mapa),
                    "json" => http::respuesta_json(mapa),
                    "text" | "string" => http::respuesta_texto(mapa),
                    _ => Valor::Nulo,
                }
            }
            Valor::Diccionario(ref mapa)
                if metodo == "json" && args_str.trim().is_empty() =>
            {
                http::valor_a_json_texto(&Valor::Diccionario(mapa.clone()))
            }
            Valor::Diccionario(ref mapa)
                if (metodo == "text" || metodo == "string") && args_str.trim().is_empty() =>
            {
                Valor::Texto(Valor::Diccionario(mapa.clone()).to_string())
            }
            Valor::Diccionario(ref mapa)
                if metodo == "parse" && args_str.trim().is_empty() =>
            {
                Valor::Diccionario(mapa.clone())
            }
            Valor::Texto(ref texto) if metodo == "parse" && args_str.trim().is_empty() => {
                http::texto_a_valor_umbral(texto)
            }
            Valor::Texto(ref texto) if metodo == "json" && args_str.trim().is_empty() => {
                http::texto_a_json_canonico(texto)
            }
            Valor::Texto(ref texto)
                if (metodo == "text" || metodo == "string") && args_str.trim().is_empty() =>
            {
                Valor::Texto(texto.clone())
            }
            otro if (metodo == "text" || metodo == "string") && args_str.trim().is_empty() => {
                Valor::Texto(otro.to_string())
            }
            otro if metodo == "json" && args_str.trim().is_empty() => {
                http::valor_a_json_texto(&otro)
            }
            _ => Valor::Nulo,
        }
    }

    fn ejecutar_metodo_lista(&mut self, items: &[Valor], metodo: &str, args_str: &str) -> Valor {
        match metodo {
            "len" => Valor::Entero(items.len() as i64),
            "push" => self.ejecutar_push_lista(items, args_str),
            "pop" => self.ejecutar_pop_lista(items),
            "json" if args_str.trim().is_empty() => {
                http::valor_a_json_texto(&Valor::Lista(items.to_vec()))
            }
            "text" | "string" if args_str.trim().is_empty() => {
                Valor::Texto(Valor::Lista(items.to_vec()).to_string())
            }
            "parse" if args_str.trim().is_empty() => Valor::Lista(items.to_vec()),
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
        let Some(datos) = self.datos_metodo(&instancia.clase, metodo) else {
            if args.is_empty() {
                if let Some(valor) = self.json_por_defecto(instancia, metodo) {
                    return valor;
                }
            }
            return Valor::Nulo;
        };
        self.preparar_objeto(instancia, &datos, &args);
        self.correr_objeto(datos.cuerpo).await
    }

    fn datos_metodo(
        &self,
        nombre_clase: &str,
        metodo: &str,
    ) -> Option<umbral_parser::ast::Metodo> {
        self.gestor_clases
            .obtener_clase(nombre_clase)?
            .obtener_metodo(metodo)
            .cloned()
    }

    fn preparar_objeto(
        &mut self,
        instancia: &crate::runtime::valores::Instancia,
        datos: &umbral_parser::ast::Metodo,
        args: &[Valor],
    ) {
        let captura = self.captura_de_clase(&instancia.clase);
        let anterior = std::mem::replace(&mut self.entorno_actual, Entorno::nuevo(None));
        self.entorno_actual = Entorno::nuevo(Some(anterior));
        Self::inyectar_captura(&mut self.entorno_actual, captura);
        self.entorno_actual
            .definir_variable("__this__".to_string(), Valor::Objeto(instancia.clone()));
        datos
            .parametros
            .iter()
            .zip(args.iter())
            .for_each(|(param, valor)| {
                self.entorno_actual
                    .definir_variable(param.nombre.clone(), valor.clone());
            });
        self.valor_retorno = None;
    }

    async fn correr_objeto(&mut self, cuerpo: Vec<umbral_parser::ast::Sentencia>) -> Valor {
        for sentencia in cuerpo {
            if let Some(valor) = self.ejecutar_sentencia(sentencia).await {
                self.restaurar_entorno();
                self.valor_retorno = None;
                return valor;
            }
        }
        self.restaurar_entorno();
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
            valor_actual = if parte.starts_with('[') && parte.ends_with(']') {
                self.evaluar_acceso_array_interpolacion(valor_actual, parte)
            } else {
                self.navegar_propiedad(valor_actual, parte)
            };

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

    fn evaluar_acceso_array_interpolacion(&self, valor_actual: Valor, parte: &str) -> Valor {
        let indice_str = &parte[1..parte.len() - 1];
        let indice_valor = self.parsear_argumento_simple(indice_str);

        let Valor::Lista(items) = valor_actual else {
            return Valor::Nulo;
        };

        if let Valor::Entero(idx) = indice_valor {
            return self.acceder_elemento_lista(items, idx);
        }

        Valor::Nulo
    }

    fn navegar_propiedad(&self, valor: Valor, propiedad: &str) -> Valor {
        match valor {
            Valor::Objeto(ref inst) => propiedad_de_objeto(inst, propiedad),
            Valor::Diccionario(ref mapa) => mapa.get(propiedad).cloned().unwrap_or(Valor::Nulo),
            Valor::Lista(ref items) if propiedad == "length" => Valor::Entero(items.len() as i64),
            _ => Valor::Nulo,
        }
    }
}

fn propiedad_de_objeto(inst: &crate::runtime::valores::Instancia, propiedad: &str) -> Valor {
    let Ok(props) = inst.propiedades.lock() else {
        return Valor::Nulo;
    };
    props.get(propiedad).cloned().unwrap_or(Valor::Nulo)
}

fn es_propiedad_doc(nombre: &str) -> bool {
    nombre == "doc" || nombre == "help"
}

fn texto_ayuda_metodo(nombre_clase: &str, metodo: &umbral_parser::ast::Metodo) -> String {
    let firma = firma_metodo(metodo);
    let nombre = format!("{}.{}", nombre_clase, metodo.nombre);
    match &metodo.doc {
        Some(d) => d.formatear(&nombre, &firma),
        None => format!("{} {}\n  (sin documentación umdocs)", nombre, firma),
    }
}

fn firma_metodo(metodo: &umbral_parser::ast::Metodo) -> String {
    let params: Vec<String> = metodo.parametros.iter().map(formatear_parametro).collect();
    format!("({})", params.join(", "))
}

fn formatear_parametro(p: &umbral_parser::ast::Parametro) -> String {
    match (p.es_rest, &p.tipo) {
        (true, Some(t)) => format!("...{}->{}", p.nombre, t.nombre),
        (true, None) => format!("...{}", p.nombre),
        (false, _) => p.nombre.clone(),
    }
}
