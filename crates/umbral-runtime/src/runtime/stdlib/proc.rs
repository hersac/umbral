use crate::runtime::valores::Valor;
use std::collections::HashMap;
use std::process::Command;

pub fn crear_modulo() -> Valor {
    let mut mapa = HashMap::new();

    mapa.insert(
        "exec".to_string(),
        Valor::FuncionNativa("exec".to_string(), exec),
    );

    Valor::Diccionario(mapa)
}

fn exec(args: Vec<Valor>) -> Valor {
    if let Some(Valor::Texto(cmd)) = args.get(0) {
        let cmd_args: Vec<String> = args
            .iter()
            .skip(1)
            .filter_map(|v| {
                if let Valor::Texto(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect();

        match Command::new(cmd).args(&cmd_args).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                let mut result = HashMap::new();
                result.insert("stdout".to_string(), Valor::Texto(stdout));
                result.insert("stderr".to_string(), Valor::Texto(stderr));
                result.insert(
                    "code".to_string(),
                    Valor::Entero(output.status.code().unwrap_or(-1) as i64),
                );

                Valor::Diccionario(result)
            }
            Err(_) => Valor::Nulo,
        }
    } else {
        Valor::Nulo
    }
}
