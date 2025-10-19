use std::io::{Write, Read, Seek, SeekFrom};
use std::fs::OpenOptions;
use std::convert::TryInto;
use crate::indice::IndiceParcial;

#[derive(Debug, Clone)]
pub struct Pedido {
    pub order_id: i64,
    pub user_id: i64,
    pub event_time: String, // 30 bytes, sempre fixo!
    pub product_id: i64,
    pub price: f64,
}

impl Pedido {
    pub const TAMANHO_REGISTRO: usize = 62; // 8+8+30+8+8 = 62

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::TAMANHO_REGISTRO);
        bytes.extend_from_slice(&self.order_id.to_le_bytes());
        bytes.extend_from_slice(&self.user_id.to_le_bytes());
        let t = format!("{:<30}", self.event_time);
        bytes.extend_from_slice(&t.as_bytes()[..30]);
        bytes.extend_from_slice(&self.product_id.to_le_bytes());
        bytes.extend_from_slice(&self.price.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let order_id = i64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let user_id = i64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let event_time = String::from_utf8_lossy(&bytes[16..46]).trim().to_string();
        let product_id = i64::from_le_bytes(bytes[46..54].try_into().unwrap());
        let price = f64::from_le_bytes(bytes[54..62].try_into().unwrap());
        Pedido { order_id, user_id, event_time, product_id, price }
    }
}

pub fn inserir_pedidos_ordenados(pedidos: &mut Vec<Pedido>, caminho: &str) -> std::io::Result<()> {
    pedidos.sort_by_key(|p| p.order_id);
    let mut arquivo = std::fs::File::create(caminho)?;
    for pedido in pedidos {
        let bytes = pedido.to_bytes();
        arquivo.write_all(&bytes)?;
    }
    Ok(())
}

pub fn mostrar_pedidos(caminho: &str, limite: usize) -> std::io::Result<Vec<Pedido>> {
    let mut arquivo = std::fs::File::open(caminho)?;
    let mut pedidos = Vec::new();
    let mut buffer = vec![0u8; Pedido::TAMANHO_REGISTRO];
    for _ in 0..limite {
        match arquivo.read_exact(&mut buffer) {
            Ok(_) => pedidos.push(Pedido::from_bytes(&buffer)),
            Err(_) => break,
        }
    }
    Ok(pedidos)
}

pub fn busca_binaria_arquivo_pedido(caminho: &str, chave: i64) -> std::io::Result<Option<Pedido>> {
    let mut arquivo = std::fs::File::open(caminho)?;
    let tamanho = arquivo.metadata()?.len();
    let num_registros = tamanho / Pedido::TAMANHO_REGISTRO as u64;
    let mut esq = 0i64;
    let mut dir = num_registros as i64 - 1;
    let mut buffer = vec![0u8; Pedido::TAMANHO_REGISTRO];
    while esq <= dir {
        let meio = (esq + dir) / 2;
        let posicao = meio as u64 * Pedido::TAMANHO_REGISTRO as u64;
        arquivo.seek(SeekFrom::Start(posicao))?;
        arquivo.read_exact(&mut buffer)?;
        let pedido = Pedido::from_bytes(&buffer);
        if pedido.order_id < chave {
            esq = meio + 1;
        } else if pedido.order_id > chave {
            dir = meio - 1;
        } else {
            return Ok(Some(pedido));
        }
    }
    Ok(None)
}

pub fn consultar_com_indice_pedido(
    caminho_arquivo: &str,
    indice: &IndiceParcial,
    chave: i64,
) -> std::io::Result<Option<Pedido>> {
    if let Some((idx, posicao_inicial)) = indice.buscar_posicao(chave) {
        let mut arquivo = std::fs::File::open(caminho_arquivo)?;
        let mut buffer = vec![0u8; Pedido::TAMANHO_REGISTRO];
        arquivo.seek(SeekFrom::Start(posicao_inicial))?;
        let posicao_final = if idx + 1 < indice.entradas.len() {
            indice.entradas[idx + 1].posicao
        } else {
            arquivo.metadata()?.len()
        };
        let mut pos_atual = posicao_inicial;
        while pos_atual < posicao_final {
            match arquivo.read_exact(&mut buffer) {
                Ok(_) => {
                    let pedido = Pedido::from_bytes(&buffer);
                    if pedido.order_id == chave {
                        return Ok(Some(pedido));
                    }
                    if pedido.order_id > chave {
                        break;
                    }
                    pos_atual += Pedido::TAMANHO_REGISTRO as u64;
                }
                Err(_) => break,
            }
        }
    }
    Ok(None)
}

pub fn remover_pedido(caminho_arquivo: &str, chave: i64) -> std::io::Result<bool> {
    let mut arquivo = OpenOptions::new()
        .read(true)
        .write(true)
        .open(caminho_arquivo)?;
    let tamanho = arquivo.metadata()?.len();
    let num_registros = tamanho / Pedido::TAMANHO_REGISTRO as u64;
    let mut buffer = vec![0u8; Pedido::TAMANHO_REGISTRO];
    for i in 0..num_registros {
        let pos = i * Pedido::TAMANHO_REGISTRO as u64;
        arquivo.seek(SeekFrom::Start(pos))?;
        arquivo.read_exact(&mut buffer)?;
        let pedido = Pedido::from_bytes(&buffer);
        if pedido.order_id == chave {
            arquivo.seek(SeekFrom::Start(pos))?;
            arquivo.write_all(&(-1i64).to_le_bytes())?;
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn inserir_novo_pedido(
    caminho_arquivo: &str,
    caminho_overflow: &str,
    pedido: Pedido,
    indice: &mut IndiceParcial,
) -> std::io::Result<()> {
    let mut arquivo_overflow = OpenOptions::new()
        .create(true)
        .append(true)
        .open(caminho_overflow)?;
    let bytes = pedido.to_bytes();
    arquivo_overflow.write_all(&bytes)?;

    // Critério: reconstruir quando overflow > 10% do principal
    let tam_principal = std::fs::File::open(caminho_arquivo)?.metadata()?.len();
    let tam_overflow = arquivo_overflow.metadata()?.len();
    if tam_overflow as f64 > tam_principal as f64 * 0.1 {
        reconstruir_arquivo_e_indice_pedido(caminho_arquivo, caminho_overflow, indice)?;
    }
    Ok(())
}

pub fn reconstruir_arquivo_e_indice_pedido(
    caminho_principal: &str,
    caminho_overflow: &str,
    indice: &mut IndiceParcial,
) -> std::io::Result<()> {
    let mut todos_pedidos: Vec<Pedido> = Vec::new();

    {
        let mut arquivo = std::fs::File::open(caminho_principal)?;
        let mut buffer = vec![0u8; Pedido::TAMANHO_REGISTRO];
        loop {
            match arquivo.read_exact(&mut buffer) {
                Ok(_) => {
                    let pedido = Pedido::from_bytes(&buffer);
                    if pedido.order_id != -1 { // ignorar removidos
                        todos_pedidos.push(pedido);
                    }
                }
                Err(_) => break,
            }
        }
    }
    {
        let mut arquivo = std::fs::File::open(caminho_overflow)?;
        let mut buffer = vec![0u8; Pedido::TAMANHO_REGISTRO];
        loop {
            match arquivo.read_exact(&mut buffer) {
                Ok(_) => {
                    let pedido = Pedido::from_bytes(&buffer);
                    if pedido.order_id != -1 {
                        todos_pedidos.push(pedido);
                    }
                }
                Err(_) => break,
            }
        }
    }
    inserir_pedidos_ordenados(&mut todos_pedidos, caminho_principal)?;
    *indice = crate::indice::construir_indice_parcial(caminho_principal, indice.fator_esparsidade)?;
    std::fs::write(caminho_overflow, "")?;

    Ok(())
}

pub fn consultar_com_indice_pedido_debug(
    caminho_arquivo: &str,
    indice: &IndiceParcial,
    chave: i64,
) -> std::io::Result<Option<Pedido>> {
    println!("\n=== DEBUG: CONSULTA POR INDICE PEDIDO ===");
    println!("Chave buscada: {}", chave);
    println!("Total de entradas no indice: {}", indice.entradas.len());
    println!("Fator de esparsidade: {}", indice.fator_esparsidade);
    println!();
    
    // Passo 1: Busca binária no índice
    println!(" PASSO 1: Busca binária no índice");
    println!("   Procurando entrada no índice que contenha a chave {}...", chave);
    
    if let Some((idx, posicao_inicial)) = indice.buscar_posicao(chave) {
        println!("    Entrada encontrada no índice!");
        println!("   📍 Índice da entrada: {}", idx);
        println!("   🔑 Chave da entrada: {}", indice.entradas[idx].chave);
        println!("   📍 Posição no arquivo: {}", posicao_inicial);
        
        // Mostra contexto das entradas próximas
        println!();
        println!(" Contexto das entradas do índice:");
        let inicio = if idx >= 2 { idx - 2 } else { 0 };
        let fim = if idx + 3 < indice.entradas.len() { idx + 3 } else { indice.entradas.len() };
        
        for i in inicio..fim {
            let marcador = if i == idx { "👉" } else { "  " };
            println!("   {} [{}] Chave: {}, Posição: {}", 
                marcador, i, indice.entradas[i].chave, indice.entradas[i].posicao);
        }
        
        // Passo 2: Determinar intervalo de busca
        println!();
        println!(" PASSO 2: Determinar intervalo de busca");
        let posicao_final = if idx + 1 < indice.entradas.len() {
            indice.entradas[idx + 1].posicao
        } else {
            std::fs::File::open(caminho_arquivo)?.metadata()?.len()
        };
        
        println!("   📍 Posição inicial: {}", posicao_inicial);
        println!("   📍 Posição final: {}", posicao_final);
        println!("   📏 Tamanho do intervalo: {} bytes", posicao_final - posicao_inicial);
        println!("    Número de registros no intervalo: {}", 
                (posicao_final - posicao_inicial) / Pedido::TAMANHO_REGISTRO as u64);
        
        // Passo 3: Busca sequencial no intervalo
        println!();
        println!(" PASSO 3: Busca sequencial no intervalo");
        let mut arquivo = std::fs::File::open(caminho_arquivo)?;
        let mut buffer = vec![0u8; Pedido::TAMANHO_REGISTRO];
        arquivo.seek(SeekFrom::Start(posicao_inicial))?;
        
        let mut pos_atual = posicao_inicial;
        let mut contador_registros = 0;
        
        println!("   🔍 Iniciando busca sequencial...");
        
        while pos_atual < posicao_final {
            match arquivo.read_exact(&mut buffer) {
                Ok(_) => {
                    let pedido = Pedido::from_bytes(&buffer);
                    contador_registros += 1;
                    
                    println!("    Registro {}: ID={}, Posição={}", 
                            contador_registros, pedido.order_id, pos_atual);
                    
                    if pedido.order_id == chave {
                        println!("    SUCESSO! Pedido encontrado!");
                        println!("    Pedido: {:?}", pedido);
                        return Ok(Some(pedido));
                    }
                    
                    if pedido.order_id > chave {
                        println!("     Chave {} maior que a buscada {}, parando busca", 
                                pedido.order_id, chave);
                        break;
                    }
                    
                    pos_atual += Pedido::TAMANHO_REGISTRO as u64;
                }
                Err(_) => {
                    println!("    Erro ao ler registro na posição {}", pos_atual);
                    break;
                }
            }
        }
        
        println!("    Total de registros verificados: {}", contador_registros);
        println!("    Pedido não encontrado no intervalo");
        
    } else {
        println!("    Nenhuma entrada encontrada no índice para a chave {}", chave);
    }
    
    println!("\n === FIM DO DEBUG ===");
    Ok(None)
}

pub fn buscar_pedido_com_overflow(caminho_principal: &str, caminho_overflow: &str, chave: i64) -> std::io::Result<Option<Pedido>> {
    // Primeiro busca no arquivo principal
    if let Some(pedido) = busca_binaria_arquivo_pedido(caminho_principal, chave)? {
        return Ok(Some(pedido));
    }
    
    // Se não encontrou no principal, busca no overflow
    if let Some(pedido) = buscar_pedido_no_overflow(caminho_overflow, chave)? {
        return Ok(Some(pedido));
    }
    
    Ok(None)
}

pub fn buscar_pedido_no_overflow(caminho_overflow: &str, chave: i64) -> std::io::Result<Option<Pedido>> {
    if !std::path::Path::new(caminho_overflow).exists() {
        return Ok(None);
    }
    
    let mut arquivo = std::fs::File::open(caminho_overflow)?;
    let tamanho = arquivo.metadata()?.len();
    let num_registros = tamanho / Pedido::TAMANHO_REGISTRO as u64;
    let mut buffer = vec![0u8; Pedido::TAMANHO_REGISTRO];
    
    for i in 0..num_registros {
        let pos = i * Pedido::TAMANHO_REGISTRO as u64;
        arquivo.seek(SeekFrom::Start(pos))?;
        match arquivo.read_exact(&mut buffer) {
            Ok(_) => {
                let pedido = Pedido::from_bytes(&buffer);
                if pedido.order_id == chave {
                    return Ok(Some(pedido));
                }
            }
            Err(_) => break,
        }
    }
    Ok(None)
}

pub fn remover_pedido_com_overflow(caminho_principal: &str, caminho_overflow: &str, chave: i64) -> std::io::Result<bool> {
    // Primeiro tenta remover do arquivo principal
    if remover_pedido(caminho_principal, chave)? {
        return Ok(true);
    }
    
    // Se não encontrou no principal, tenta remover do overflow
    if remover_pedido_do_overflow(caminho_overflow, chave)? {
        return Ok(true);
    }
    
    Ok(false)
}

pub fn remover_pedido_do_overflow(caminho_overflow: &str, chave: i64) -> std::io::Result<bool> {
    if !std::path::Path::new(caminho_overflow).exists() {
        return Ok(false);
    }
    
    let mut arquivo = OpenOptions::new()
        .read(true)
        .write(true)
        .open(caminho_overflow)?;

    let tamanho = arquivo.metadata()?.len();
    let num_registros = tamanho / Pedido::TAMANHO_REGISTRO as u64;
    let mut buffer = vec![0u8; Pedido::TAMANHO_REGISTRO];

    for i in 0..num_registros {
        let pos = i * Pedido::TAMANHO_REGISTRO as u64;
        arquivo.seek(SeekFrom::Start(pos))?;
        arquivo.read_exact(&mut buffer)?;

        let pedido = Pedido::from_bytes(&buffer);
        if pedido.order_id == chave {
            // Marca como removido: sobrescreve order_id para -1
            arquivo.seek(SeekFrom::Start(pos))?;
            arquivo.write_all(&(-1i64).to_le_bytes())?;
            return Ok(true);
        }
    }
    Ok(false)
}