use crate::runtime::valores::{ClaseEnchufe, InteriorEnchufe, ManejadorEnchufe, Valor};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

const MAX_LECTURA: i64 = 65536;
const MAX_ESCUCHA: i64 = 1024;

pub fn crear_modulo() -> Valor {
    Valor::Diccionario(HashMap::from([
        ("tcp".to_string(), Valor::FuncionNativa("tcp".to_string(), crear_tcp)),
        ("udp".to_string(), Valor::FuncionNativa("udp".to_string(), crear_udp)),
        ("Address".to_string(), crear_direcciones()),
    ]))
}

fn crear_direcciones() -> Valor {
    Valor::Diccionario(HashMap::from([(
        "create".to_string(),
        Valor::FuncionNativa("create".to_string(), crear_direccion),
    )]))
}

fn crear_tcp(_argumentos: Vec<Valor>) -> Valor {
    nuevo_enchufe(ClaseEnchufe::Tcp)
}

fn crear_udp(_argumentos: Vec<Valor>) -> Valor {
    nuevo_enchufe(ClaseEnchufe::Udp)
}

fn nuevo_enchufe(clase: ClaseEnchufe) -> Valor {
    Valor::Enchufe(ManejadorEnchufe(Arc::new(Mutex::new(InteriorEnchufe {
        clase,
        pendiente: None,
        oyente: None,
        flujo: None,
        datagrama: None,
        lectura_ms: None,
        escritura_ms: None,
        reuso: false,
        cerrado: false,
    }))))
}

fn nuevo_flujo(flujo: TcpStream) -> Valor {
    Valor::Enchufe(ManejadorEnchufe(Arc::new(Mutex::new(InteriorEnchufe {
        clase: ClaseEnchufe::Tcp,
        pendiente: None,
        oyente: None,
        flujo: Some(flujo),
        datagrama: None,
        lectura_ms: None,
        escritura_ms: None,
        reuso: false,
        cerrado: false,
    }))))
}

fn fallar(mensaje: &str) -> Valor {
    eprintln!("Error de red: {}", mensaje);
    Valor::Nulo
}

fn texto_en(argumentos: &[Valor], indice: usize) -> Option<String> {
    match argumentos.get(indice) {
        Some(Valor::Texto(actual)) => Some(actual.clone()),
        _ => None,
    }
}

fn entero_en(argumentos: &[Valor], indice: usize) -> Option<i64> {
    match argumentos.get(indice) {
        Some(Valor::Entero(actual)) => Some(*actual),
        _ => None,
    }
}

fn puerto_valido(puerto: i64, servidor: String) -> Option<(String, u16)> {
    match (0..=65535).contains(&puerto) {
        true => Some((servidor, puerto as u16)),
        false => None,
    }
}

fn puerto_en(argumentos: &[Valor], indice: usize, servidor: String) -> Option<(String, u16)> {
    match entero_en(argumentos, indice) {
        Some(puerto) => puerto_valido(puerto, servidor),
        None => None,
    }
}

fn extraer_direccion(argumentos: &[Valor]) -> Option<(String, u16)> {
    match texto_en(argumentos, 0) {
        Some(servidor) => puerto_en(argumentos, 1, servidor),
        None => None,
    }
}

fn dic_direccion(servidor: &str, puerto: u16) -> Valor {
    Valor::Diccionario(HashMap::from([
        ("host".to_string(), Valor::Texto(servidor.to_string())),
        ("port".to_string(), Valor::Entero(puerto as i64)),
    ]))
}

fn crear_direccion(argumentos: Vec<Valor>) -> Valor {
    match extraer_direccion(&argumentos) {
        Some((servidor, puerto)) => dic_direccion(&servidor, puerto),
        None => fallar("Address.create requiere servidor Str y puerto Int"),
    }
}

pub fn invocar(manejador: &ManejadorEnchufe, metodo: &str, argumentos: Vec<Valor>) -> Valor {
    match manejador.0.lock() {
        Ok(estado) => repartir(estado, metodo, argumentos),
        Err(_) => fallar("enchufe bloqueado"),
    }
}

fn repartir(estado: MutexGuard<InteriorEnchufe>, metodo: &str, argumentos: Vec<Valor>) -> Valor {
    match metodo {
        "bind" => vincular(estado, argumentos),
        "listen" => escuchar(estado, argumentos),
        "accept" => aceptar(estado, argumentos),
        "connect" => conectar(estado, argumentos),
        "read" => leer(estado, argumentos),
        "write" => escribir(estado, argumentos),
        "send" => enviar(estado, argumentos),
        "receive" => recibir(estado, argumentos),
        "close" => cerrar(estado, argumentos),
        "set_timeout" => definir_tiempo(estado, argumentos),
        "set_read_timeout" => definir_tiempo_lectura(estado, argumentos),
        "set_write_timeout" => definir_tiempo_escritura(estado, argumentos),
        "set_reuse_address" => definir_reuso(estado, argumentos),
        "local_address" => direccion_local(estado, argumentos),
        "remote_address" => direccion_remota(estado, argumentos),
        _ => fallar("método de enchufe desconocido"),
    }
}

fn en_uso(estado: &InteriorEnchufe) -> bool {
    estado.pendiente.is_some()
        || estado.oyente.is_some()
        || estado.flujo.is_some()
        || estado.datagrama.is_some()
        || estado.cerrado
}

fn primera_ip(servidor: &str) -> Option<IpAddr> {
    match (servidor, 0).to_socket_addrs() {
        Ok(direcciones) => direcciones.map(|actual| actual.ip()).next(),
        Err(_) => None,
    }
}

fn resolver_ip(servidor: &str) -> Option<IpAddr> {
    match servidor.parse::<IpAddr>() {
        Ok(ip) => Some(ip),
        Err(_) => primera_ip(servidor),
    }
}

fn dominio(destino: SocketAddr) -> socket2::Domain {
    match destino {
        SocketAddr::V4(_) => socket2::Domain::IPV4,
        SocketAddr::V6(_) => socket2::Domain::IPV6,
    }
}

fn fijar_destino(enlace: socket2::Socket, destino: SocketAddr) -> Option<socket2::Socket> {
    match enlace.bind(&destino.into()) {
        Ok(_) => Some(enlace),
        Err(_) => None,
    }
}

fn fijar_enlace(enlace: socket2::Socket, destino: SocketAddr, reuso: bool) -> Option<socket2::Socket> {
    match enlace.set_reuse_address(reuso) {
        Ok(_) => fijar_destino(enlace, destino),
        Err(_) => None,
    }
}

fn abrir_enlace(destino: SocketAddr, tipo: socket2::Type, reuso: bool) -> Option<socket2::Socket> {
    match socket2::Socket::new(dominio(destino), tipo, None) {
        Ok(enlace) => fijar_enlace(enlace, destino, reuso),
        Err(_) => None,
    }
}

fn guardar_pendiente(estado: &mut InteriorEnchufe, pendiente: socket2::Socket) -> Valor {
    estado.pendiente = Some(pendiente);
    Valor::Booleano(true)
}

fn guardar_datagrama(estado: &mut InteriorEnchufe, enlace: socket2::Socket) -> Valor {
    let datagrama: UdpSocket = enlace.into();
    aplicar_tiempos_datagrama(&datagrama, estado.lectura_ms, estado.escritura_ms);
    estado.datagrama = Some(datagrama);
    Valor::Booleano(true)
}

fn enlazar_tcp(estado: &mut InteriorEnchufe, destino: SocketAddr) -> Valor {
    match abrir_enlace(destino, socket2::Type::STREAM, estado.reuso) {
        Some(pendiente) => guardar_pendiente(estado, pendiente),
        None => fallar("no se pudo vincular el puerto tcp"),
    }
}

fn enlazar_udp(estado: &mut InteriorEnchufe, destino: SocketAddr) -> Valor {
    match abrir_enlace(destino, socket2::Type::DGRAM, estado.reuso) {
        Some(enlace) => guardar_datagrama(estado, enlace),
        None => fallar("no se pudo vincular el puerto udp"),
    }
}

fn enlazar_ip(estado: &mut InteriorEnchufe, servidor: &str, puerto: u16) -> Valor {
    match resolver_ip(servidor) {
        Some(ip) => enlazar_clase(estado, SocketAddr::new(ip, puerto)),
        None => fallar("servidor no resoluble para bind"),
    }
}

fn enlazar_clase(estado: &mut InteriorEnchufe, destino: SocketAddr) -> Valor {
    match estado.clase {
        ClaseEnchufe::Tcp => enlazar_tcp(estado, destino),
        ClaseEnchufe::Udp => enlazar_udp(estado, destino),
    }
}

fn vincular_clase(estado: &mut InteriorEnchufe, servidor: &str, puerto: u16) -> Valor {
    match en_uso(estado) {
        true => fallar("el enchufe ya está vinculado o cerrado"),
        false => enlazar_ip(estado, servidor, puerto),
    }
}

fn vincular(mut estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match extraer_direccion(&argumentos) {
        Some((servidor, puerto)) => vincular_clase(&mut estado, &servidor, puerto),
        None => fallar("bind requiere servidor Str y puerto Int"),
    }
}

fn guardar_oyente(estado: &mut InteriorEnchufe, enlace: socket2::Socket) -> Valor {
    let oyente: TcpListener = enlace.into();
    estado.oyente = Some(oyente);
    Valor::Booleano(true)
}

fn escuchar_enlace(estado: &mut InteriorEnchufe, enlace: socket2::Socket, espera: i64) -> Valor {
    match enlace.listen(espera as i32) {
        Ok(_) => guardar_oyente(estado, enlace),
        Err(_) => fallar("no se pudo iniciar la escucha"),
    }
}

fn escuchar_pendiente(estado: &mut InteriorEnchufe, espera: i64) -> Valor {
    match estado.pendiente.take() {
        Some(enlace) => escuchar_enlace(estado, enlace, espera),
        None => fallar("ejecuta bind antes de listen"),
    }
}

fn escuchar_oyente(estado: &mut InteriorEnchufe, espera: i64) -> Valor {
    match estado.oyente.is_some() {
        true => fallar("el enchufe ya está en escucha"),
        false => escuchar_pendiente(estado, espera),
    }
}

fn escuchar_clase(estado: &mut InteriorEnchufe, espera: i64) -> Valor {
    match estado.clase {
        ClaseEnchufe::Tcp => escuchar_oyente(estado, espera),
        ClaseEnchufe::Udp => fallar("listen solo aplica a enchufes tcp"),
    }
}

fn escuchar(mut estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match entero_en(&argumentos, 0) {
        Some(espera) => escuchar_limite(&mut estado, espera),
        None => fallar("listen requiere espera Int"),
    }
}

fn escuchar_limite(estado: &mut InteriorEnchufe, espera: i64) -> Valor {
    match (1..=MAX_ESCUCHA).contains(&espera) {
        true => escuchar_clase(estado, espera),
        false => fallar("listen requiere espera entre 1 y 1024"),
    }
}

fn envolver_flujo(estado: &InteriorEnchufe, flujo: TcpStream) -> Valor {
    aplicar_tiempos_flujo(&flujo, estado.lectura_ms, estado.escritura_ms);
    nuevo_flujo(flujo)
}

fn aceptar_conexion(estado: &InteriorEnchufe, oyente: &TcpListener) -> Valor {
    match oyente.accept() {
        Ok((flujo, _)) => envolver_flujo(estado, flujo),
        Err(_) => fallar("no se pudo aceptar la conexión"),
    }
}

fn aceptar(estado: MutexGuard<InteriorEnchufe>, _argumentos: Vec<Valor>) -> Valor {
    match estado.oyente.as_ref() {
        Some(oyente) => aceptar_conexion(&estado, oyente),
        None => fallar("el enchufe no está en escucha"),
    }
}

fn guardar_flujo(estado: &mut InteriorEnchufe, flujo: TcpStream) -> Valor {
    aplicar_tiempos_flujo(&flujo, estado.lectura_ms, estado.escritura_ms);
    estado.flujo = Some(flujo);
    Valor::Booleano(true)
}

fn conectar_tcp(estado: &mut InteriorEnchufe, servidor: &str, puerto: u16) -> Valor {
    match TcpStream::connect((servidor, puerto)) {
        Ok(flujo) => guardar_flujo(estado, flujo),
        Err(_) => fallar("no se pudo conectar al servidor"),
    }
}

fn conectar_datagrama(datagrama: &UdpSocket, servidor: &str, puerto: u16) -> Valor {
    match datagrama.connect((servidor, puerto)) {
        Ok(_) => Valor::Booleano(true),
        Err(_) => fallar("no se pudo conectar el udp"),
    }
}

fn conectar_udp(estado: &InteriorEnchufe, servidor: &str, puerto: u16) -> Valor {
    match estado.datagrama.as_ref() {
        Some(datagrama) => conectar_datagrama(datagrama, servidor, puerto),
        None => fallar("udp requiere bind antes de connect"),
    }
}

fn conectar_clase(estado: &mut InteriorEnchufe, servidor: &str, puerto: u16) -> Valor {
    match estado.clase {
        ClaseEnchufe::Tcp => conectar_tcp(estado, servidor, puerto),
        ClaseEnchufe::Udp => conectar_udp(estado, servidor, puerto),
    }
}

fn conectar_nuevo(estado: &mut InteriorEnchufe, servidor: &str, puerto: u16) -> Valor {
    match en_uso(estado) {
        true => fallar("el enchufe ya está en uso o cerrado"),
        false => conectar_clase(estado, servidor, puerto),
    }
}

fn conectar_udp_vinculado(estado: &mut InteriorEnchufe, servidor: &str, puerto: u16) -> Valor {
    match estado.clase {
        ClaseEnchufe::Tcp => conectar_nuevo(estado, servidor, puerto),
        ClaseEnchufe::Udp => conectar_udp(estado, servidor, puerto),
    }
}

fn conectar(mut estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match extraer_direccion(&argumentos) {
        Some((servidor, puerto)) => conectar_udp_vinculado(&mut estado, &servidor, puerto),
        None => fallar("connect requiere servidor Str y puerto Int"),
    }
}

fn texto_bufer(bufer: &[u8], total: usize) -> Valor {
    Valor::Texto(String::from_utf8_lossy(&bufer[..total]).to_string())
}

fn leer_flujo(mut flujo: &TcpStream, limite: usize) -> Valor {
    let mut bufer = vec![0u8; limite];
    match flujo.read(&mut bufer) {
        Ok(total) => texto_bufer(&bufer, total),
        Err(_) => fallar("error al leer del enchufe"),
    }
}

fn leer_limite(estado: &InteriorEnchufe, limite: i64) -> Valor {
    match (1..=MAX_LECTURA).contains(&limite) {
        true => leer_conectado(estado, limite as usize),
        false => fallar("read requiere tamaño entre 1 y 65536"),
    }
}

fn leer_conectado(estado: &InteriorEnchufe, limite: usize) -> Valor {
    match estado.flujo.as_ref() {
        Some(flujo) => leer_flujo(flujo, limite),
        None => fallar("el enchufe no está conectado"),
    }
}

fn leer(estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match entero_en(&argumentos, 0) {
        Some(limite) => leer_limite(&estado, limite),
        None => fallar("read requiere tamaño Int"),
    }
}

fn escribir_flujo(mut flujo: &TcpStream, datos: &str) -> Valor {
    match flujo.write_all(datos.as_bytes()) {
        Ok(_) => Valor::Entero(datos.len() as i64),
        Err(_) => fallar("error al escribir en el enchufe"),
    }
}

fn escribir_conectado(estado: &InteriorEnchufe, datos: &str) -> Valor {
    match estado.flujo.as_ref() {
        Some(flujo) => escribir_flujo(flujo, datos),
        None => fallar("el enchufe no está conectado"),
    }
}

fn escribir(estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match texto_en(&argumentos, 0) {
        Some(datos) => escribir_conectado(&estado, &datos),
        None => fallar("write requiere datos Str"),
    }
}

fn enviar_conectado(estado: &InteriorEnchufe, datos: &str) -> Valor {
    match estado.datagrama.as_ref() {
        Some(actual) => enviar_directo(actual, datos, None),
        None => fallar("udp requiere bind antes de send"),
    }
}

fn enviar_directo(datagrama: &UdpSocket, datos: &str, destino: Option<SocketAddr>) -> Valor {
    match enviar_destino(datagrama, datos, destino) {
        Some(total) => Valor::Entero(total as i64),
        None => fallar("error al enviar el datagrama"),
    }
}

fn enviar_destino(datagrama: &UdpSocket, datos: &str, destino: Option<SocketAddr>) -> Option<usize> {
    match destino {
        Some(direccion) => datagrama.send_to(datos.as_bytes(), direccion).ok(),
        None => datagrama.send(datos.as_bytes()).ok(),
    }
}

fn destino_ip(servidor: &str, puerto: u16) -> Option<SocketAddr> {
    match resolver_ip(servidor) {
        Some(ip) => Some(SocketAddr::new(ip, puerto)),
        None => None,
    }
}

fn enviar_remoto(estado: &InteriorEnchufe, datos: &str, servidor: &str, puerto: u16) -> Valor {
    match destino_ip(servidor, puerto) {
        Some(direccion) => enviar_a(estado, datos, direccion),
        None => fallar("destino no resoluble para send"),
    }
}

fn enviar_a(estado: &InteriorEnchufe, datos: &str, direccion: SocketAddr) -> Valor {
    match estado.datagrama.as_ref() {
        Some(actual) => enviar_directo(actual, datos, Some(direccion)),
        None => fallar("udp requiere bind antes de send"),
    }
}

fn enviar_destino_txt(estado: &InteriorEnchufe, datos: &str, argumentos: &[Valor]) -> Valor {
    match extraer_direccion(&argumentos[1..]) {
        Some((servidor, puerto)) => enviar_remoto(estado, datos, &servidor, puerto),
        None => fallar("send requiere destino Str e puerto Int"),
    }
}

fn enviar_datagrama(estado: &InteriorEnchufe, datos: &str, argumentos: &[Valor]) -> Valor {
    match argumentos.len() {
        1 => enviar_conectado(estado, datos),
        _ => enviar_destino_txt(estado, datos, argumentos),
    }
}

fn enviar(estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match texto_en(&argumentos, 0) {
        Some(datos) => enviar_datagrama(&estado, &datos, &argumentos),
        None => fallar("send requiere datos Str"),
    }
}

fn recibir_en(datagrama: &UdpSocket, limite: usize) -> Valor {
    let mut bufer = vec![0u8; limite];
    match datagrama.recv_from(&mut bufer) {
        Ok((total, _)) => texto_bufer(&bufer, total),
        Err(_) => fallar("error al recibir el datagrama"),
    }
}

fn recibir_limite(estado: &InteriorEnchufe, limite: i64) -> Valor {
    match (1..=MAX_LECTURA).contains(&limite) {
        true => recibir_vinculado(estado, limite as usize),
        false => fallar("receive requiere tamaño entre 1 y 65536"),
    }
}

fn recibir_vinculado(estado: &InteriorEnchufe, limite: usize) -> Valor {
    match estado.datagrama.as_ref() {
        Some(actual) => recibir_en(actual, limite),
        None => fallar("udp requiere bind antes de receive"),
    }
}

fn recibir(estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match entero_en(&argumentos, 0) {
        Some(limite) => recibir_limite(&estado, limite),
        None => fallar("receive requiere tamaño Int"),
    }
}

fn silenciar_flujo(flujo: Option<&TcpStream>) {
    match flujo {
        Some(actual) => apagar(actual),
        None => {},
    }
}

fn apagar(flujo: &TcpStream) {
    let _ = flujo.shutdown(Shutdown::Both);
}

fn cerrar(mut estado: MutexGuard<InteriorEnchufe>, _argumentos: Vec<Valor>) -> Valor {
    silenciar_flujo(estado.flujo.as_ref());
    estado.pendiente = None;
    estado.oyente = None;
    estado.flujo = None;
    estado.datagrama = None;
    estado.cerrado = true;
    Valor::Booleano(true)
}

fn milis(ms: i64) -> Option<u64> {
    match ms > 0 {
        true => Some(ms as u64),
        false => None,
    }
}

fn aplicar_tiempos_flujo(flujo: &TcpStream, lectura: Option<u64>, escritura: Option<u64>) {
    let _ = flujo.set_read_timeout(lectura.map(Duration::from_millis));
    let _ = flujo.set_write_timeout(escritura.map(Duration::from_millis));
}

fn aplicar_tiempos_datagrama(datagrama: &UdpSocket, lectura: Option<u64>, escritura: Option<u64>) {
    let _ = datagrama.set_read_timeout(lectura.map(Duration::from_millis));
    let _ = datagrama.set_write_timeout(escritura.map(Duration::from_millis));
}

fn aplicar_tiempos(estado: &InteriorEnchufe) {
    match estado.flujo.as_ref() {
        Some(flujo) => aplicar_tiempos_flujo(flujo, estado.lectura_ms, estado.escritura_ms),
        None => aplicar_tiempos_udp(estado),
    }
}

fn aplicar_tiempos_udp(estado: &InteriorEnchufe) {
    match estado.datagrama.as_ref() {
        Some(actual) => aplicar_tiempos_datagrama(actual, estado.lectura_ms, estado.escritura_ms),
        None => {},
    }
}

fn guardar_tiempos(estado: &mut InteriorEnchufe, lectura: Option<u64>, escritura: Option<u64>) -> Valor {
    estado.lectura_ms = lectura;
    estado.escritura_ms = escritura;
    aplicar_tiempos(estado);
    Valor::Booleano(true)
}

fn definir_tiempo(mut estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match entero_en(&argumentos, 0) {
        Some(ms) => guardar_tiempos(&mut estado, milis(ms), milis(ms)),
        None => fallar("set_timeout requiere milisegundos Int"),
    }
}

fn definir_tiempo_lectura(mut estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match entero_en(&argumentos, 0) {
        Some(ms) => guardar_lectura(&mut estado, ms),
        None => fallar("set_read_timeout requiere milisegundos Int"),
    }
}

fn guardar_lectura(estado: &mut InteriorEnchufe, ms: i64) -> Valor {
    let escritura = estado.escritura_ms;
    guardar_tiempos(estado, milis(ms), escritura)
}

fn definir_tiempo_escritura(mut estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match entero_en(&argumentos, 0) {
        Some(ms) => guardar_escritura(&mut estado, ms),
        None => fallar("set_write_timeout requiere milisegundos Int"),
    }
}

fn guardar_escritura(estado: &mut InteriorEnchufe, ms: i64) -> Valor {
    let lectura = estado.lectura_ms;
    guardar_tiempos(estado, lectura, milis(ms))
}

fn definir_reuso_valor(estado: &mut InteriorEnchufe, valor: bool) -> Valor {
    match en_uso(estado) {
        true => fallar("set_reuse_address debe llamarse antes de bind"),
        false => activar_reuso(estado, valor),
    }
}

fn activar_reuso(estado: &mut InteriorEnchufe, valor: bool) -> Valor {
    estado.reuso = valor;
    Valor::Booleano(true)
}

fn definir_reuso(mut estado: MutexGuard<InteriorEnchufe>, argumentos: Vec<Valor>) -> Valor {
    match argumentos.get(0) {
        Some(Valor::Booleano(valor)) => definir_reuso_valor(&mut estado, *valor),
        _ => fallar("set_reuse_address requiere Bool"),
    }
}

fn direccion_de(resultado: std::io::Result<SocketAddr>, mensaje: &str) -> Valor {
    match resultado {
        Ok(actual) => dic_direccion(&actual.ip().to_string(), actual.port()),
        Err(_) => fallar(mensaje),
    }
}

fn local_tcp(estado: &InteriorEnchufe) -> Valor {
    match estado.oyente.as_ref() {
        Some(oyente) => direccion_de(oyente.local_addr(), "sin dirección local"),
        None => local_tcp_flujo(estado),
    }
}

fn local_tcp_flujo(estado: &InteriorEnchufe) -> Valor {
    match estado.flujo.as_ref() {
        Some(flujo) => direccion_de(flujo.local_addr(), "sin dirección local"),
        None => local_tcp_pendiente(estado),
    }
}

fn local_tcp_pendiente(estado: &InteriorEnchufe) -> Valor {
    match estado.pendiente.as_ref() {
        Some(pendiente) => direccion_pendiente(pendiente),
        None => fallar("sin dirección local"),
    }
}

fn direccion_pendiente(pendiente: &socket2::Socket) -> Valor {
    match pendiente.local_addr() {
        Ok(direccion) => direccion_sock(direccion),
        Err(_) => fallar("sin dirección local"),
    }
}

fn direccion_sock(direccion: socket2::SockAddr) -> Valor {
    match direccion.as_socket() {
        Some(actual) => dic_direccion(&actual.ip().to_string(), actual.port()),
        None => fallar("sin dirección local"),
    }
}

fn local_udp(estado: &InteriorEnchufe) -> Valor {
    match estado.datagrama.as_ref() {
        Some(actual) => direccion_de(actual.local_addr(), "sin dirección local"),
        None => fallar("sin dirección local"),
    }
}

fn direccion_local(estado: MutexGuard<InteriorEnchufe>, _argumentos: Vec<Valor>) -> Valor {
    match estado.clase {
        ClaseEnchufe::Tcp => local_tcp(&estado),
        ClaseEnchufe::Udp => local_udp(&estado),
    }
}

fn remota_tcp(estado: &InteriorEnchufe) -> Valor {
    match estado.flujo.as_ref() {
        Some(flujo) => direccion_de(flujo.peer_addr(), "sin dirección remota"),
        None => fallar("sin dirección remota"),
    }
}

fn remota_udp(estado: &InteriorEnchufe) -> Valor {
    match estado.datagrama.as_ref() {
        Some(actual) => direccion_de(actual.peer_addr(), "sin dirección remota"),
        None => fallar("sin dirección remota"),
    }
}

fn direccion_remota(estado: MutexGuard<InteriorEnchufe>, _argumentos: Vec<Valor>) -> Valor {
    match estado.clase {
        ClaseEnchufe::Tcp => remota_tcp(&estado),
        ClaseEnchufe::Udp => remota_udp(&estado),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn direccion_libre() -> (String, u16) {
        ("127.0.0.1".to_string(), 0)
    }

    #[test]
    fn crea_direccion_valida() {
        match crear_direccion(vec![Valor::Texto("127.0.0.1".to_string()), Valor::Entero(8080)]) {
            Valor::Diccionario(mapa) => {
                assert!(matches!(mapa.get("host"), Some(Valor::Texto(actual)) if actual == "127.0.0.1"));
                assert!(matches!(mapa.get("port"), Some(Valor::Entero(8080))));
            }
            _ => panic!("se esperaba diccionario"),
        }
    }

    #[test]
    fn rechaza_direccion_invalida() {
        assert!(matches!(crear_direccion(vec![]), Valor::Nulo));
        assert!(matches!(
            crear_direccion(vec![Valor::Texto("x".to_string()), Valor::Entero(99999)]),
            Valor::Nulo
        ));
    }

    #[test]
    fn tcp_vincula_puerto_libre() {
        let (servidor, puerto) = direccion_libre();
        match crear_tcp(vec![]) {
            Valor::Enchufe(manejador) => {
                assert!(matches!(
                    invocar(&manejador, "bind", vec![Valor::Texto(servidor), Valor::Entero(puerto as i64)]),
                    Valor::Booleano(true)
                ));
                assert!(matches!(invocar(&manejador, "listen", vec![Valor::Entero(5)]), Valor::Booleano(true)));
                match invocar(&manejador, "local_address", vec![]) {
                    Valor::Diccionario(mapa) => assert!(matches!(mapa.get("port"), Some(Valor::Entero(p)) if *p > 0)),
                    _ => panic!("se esperaba dirección local"),
                }
                assert!(matches!(invocar(&manejador, "close", vec![]), Valor::Booleano(true)));
            }
            _ => panic!("se esperaba enchufe"),
        }
    }

    #[test]
    fn udp_bucle_local() {
        match (crear_udp(vec![]), crear_udp(vec![])) {
            (Valor::Enchufe(receptor), Valor::Enchufe(origen)) => {
                assert!(matches!(
                    invocar(&receptor, "bind", vec![Valor::Texto("127.0.0.1".to_string()), Valor::Entero(0)]),
                    Valor::Booleano(true)
                ));
                assert!(matches!(
                    invocar(&origen, "bind", vec![Valor::Texto("127.0.0.1".to_string()), Valor::Entero(0)]),
                    Valor::Booleano(true)
                ));
                let destino = puerto_enchufe(&receptor);
                assert!(matches!(
                    invocar(
                        &origen,
                        "send",
                        vec![
                            Valor::Texto("hola".to_string()),
                            Valor::Texto("127.0.0.1".to_string()),
                            Valor::Entero(destino),
                        ]
                    ),
                    Valor::Entero(4)
                ));
                assert!(matches!(
                    invocar(&receptor, "set_timeout", vec![Valor::Entero(2000)]),
                    Valor::Booleano(true)
                ));
                match invocar(&receptor, "receive", vec![Valor::Entero(64)]) {
                    Valor::Texto(actual) => assert_eq!(actual, "hola"),
                    _ => panic!("se esperaba el datagrama"),
                }
            }
            _ => panic!("se esperaba enchufe"),
        }
    }

    fn puerto_enchufe(receptor: &ManejadorEnchufe) -> i64 {
        match invocar(receptor, "local_address", vec![]) {
            Valor::Diccionario(mapa) => extraer_puerto(&mapa),
            _ => panic!("sin dirección local"),
        }
    }

    fn extraer_puerto(mapa: &HashMap<String, Valor>) -> i64 {
        match mapa.get("port") {
            Some(Valor::Entero(puerto)) => *puerto,
            _ => panic!("sin puerto"),
        }
    }
}
