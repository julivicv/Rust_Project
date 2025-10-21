use std::io::{Write, Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct Produto {
    pub product_id: i64,
    pub category_alias: String,
    pub price: f64,
    pub material: String,
    pub stone: String,
}

impl Produto {
    pub const TAMANHO_REGISTRO: usize = 87;

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::TAMANHO_REGISTRO);
        bytes.extend_from_slice(&self.product_id.to_le_bytes());
        let cat = format!("{:<30}", self.category_alias);
        bytes.extend_from_slice(&cat.as_bytes()[..30]);
        bytes.extend_from_slice(&self.price.to_le_bytes());
        let mat = format!("{:<20}", self.material);
        bytes.extend_from_slice(&mat.as_bytes()[..20]);
        let st = format!("{:<20}", self.stone);
        bytes.extend_from_slice(&st.as_bytes()[..20]);
        bytes.push(b'\n');
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let product_id = i64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let category_alias = String::from_utf8_lossy(&bytes[8..38]).trim().to_string();
        let price = f64::from_le_bytes(bytes[38..46].try_into().unwrap());
        let material = String::from_utf8_lossy(&bytes[46..66]).trim().to_string();
        let stone = String::from_utf8_lossy(&bytes[66..86]).trim().to_string();
        Produto { product_id, category_alias, price, material, stone }
    }
}

// Funções relacionadas a inserção, busca, mostrar e consulta via índice parcial
use crate::indice::{IndiceParcial};

pub fn inserir_produtos_ordenados(produtos: &mut Vec<Produto>, caminho: &str) -> std::io::Result<()> {
    produtos.sort_by_key(|p| p.product_id);
    let mut arquivo = std::fs::File::create(caminho)?;
    for produto in produtos {
        let bytes = produto.to_bytes();
        arquivo.write_all(&bytes)?;
    }
    Ok(())
}

pub fn mostrar_produtos(caminho: &str, limite: usize) -> std::io::Result<Vec<Produto>> {
    let mut arquivo = std::fs::File::open(caminho)?;
    let mut produtos = Vec::new();
    let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];
    for _ in 0..limite {
        match arquivo.read_exact(&mut buffer) {
            Ok(_) => produtos.push(Produto::from_bytes(&buffer)),
            Err(_) => break,
        }
    }
    Ok(produtos)
}

pub fn busca_binaria_arquivo(caminho: &str, chave: i64) -> std::io::Result<Option<Produto>> {
    let mut arquivo = std::fs::File::open(caminho)?;
    let tamanho = arquivo.metadata()?.len();
    let num_registros = tamanho / Produto::TAMANHO_REGISTRO as u64;
    let mut esq = 0i64;
    let mut dir = num_registros as i64 - 1;
    let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];
    while esq <= dir {
        let meio = (esq + dir) / 2;
        let posicao = meio as u64 * Produto::TAMANHO_REGISTRO as u64;
        arquivo.seek(SeekFrom::Start(posicao))?;
        arquivo.read_exact(&mut buffer)?;
        let produto = Produto::from_bytes(&buffer);
        if produto.product_id < chave {
            esq = meio + 1;
        } else if produto.product_id > chave {
            dir = meio - 1;
        } else {
            return Ok(Some(produto));
        }
    }
    Ok(None)
}

pub fn remover_produto(caminho_arquivo: &str, chave: i64) -> std::io::Result<bool> {
    let mut arquivo = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(caminho_arquivo)?;

    let tamanho = arquivo.metadata()?.len();
    let num_registros = tamanho / Produto::TAMANHO_REGISTRO as u64;
    let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];

    for i in 0..num_registros {
        let pos = i * Produto::TAMANHO_REGISTRO as u64;
        arquivo.seek(SeekFrom::Start(pos))?;
        arquivo.read_exact(&mut buffer)?;

        let produto = Produto::from_bytes(&buffer);
        if produto.product_id == chave {
            // Marca como removido: sobrescreve product_id para -1
            arquivo.seek(SeekFrom::Start(pos))?;
            arquivo.write_all(&(-1i64).to_le_bytes())?;
            return Ok(true);
        }
    }
    Ok(false) // produto não encontrado
}


pub fn consultar_com_indice(caminho_arquivo: &str, indice: &IndiceParcial, chave: i64) -> std::io::Result<Option<Produto>>  {
    if let Some((idx, posicao_inicial)) = indice.buscar_posicao(chave) {
        let mut arquivo = std::fs::File::open(caminho_arquivo)?;
        let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];
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
                    let produto = Produto::from_bytes(&buffer);
                    if produto.product_id == chave {
                        return Ok(Some(produto));
                    }
                    if produto.product_id > chave {
                        break;
                    }
                    pos_atual += Produto::TAMANHO_REGISTRO as u64;
                }
                Err(_) => break,
            }
        }
    }
    Ok(None)
}


// Função para buscar produto considerando overflow
pub fn buscar_produto_com_overflow(caminho_principal: &str, caminho_overflow: &str, chave: i64) -> std::io::Result<Option<Produto>> {
    // Primeiro busca no arquivo principal
    if let Some(produto) = busca_binaria_arquivo(caminho_principal, chave)? {
        return Ok(Some(produto));
    }
    
    // Se não encontrou no principal, busca no overflow
    if let Some(produto) = buscar_no_overflow(caminho_overflow, chave)? {
        return Ok(Some(produto));
    }
    
    Ok(None)
}

// Função para buscar no arquivo de overflow
pub fn buscar_no_overflow(caminho_overflow: &str, chave: i64) -> std::io::Result<Option<Produto>> {
    if !std::path::Path::new(caminho_overflow).exists() {
        return Ok(None);
    }
    
    let mut arquivo = std::fs::File::open(caminho_overflow)?;
    let tamanho = arquivo.metadata()?.len();
    let num_registros = tamanho / Produto::TAMANHO_REGISTRO as u64;
    let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];
    
    for i in 0..num_registros {
        let pos = i * Produto::TAMANHO_REGISTRO as u64;
        arquivo.seek(SeekFrom::Start(pos))?;
        match arquivo.read_exact(&mut buffer) {
            Ok(_) => {
                let produto = Produto::from_bytes(&buffer);
                if produto.product_id == chave {
                    return Ok(Some(produto));
                }
            }
            Err(_) => break,
        }
    }
    Ok(None)
}

// Função para remover produto considerando overflow
pub fn remover_produto_com_overflow(caminho_principal: &str, caminho_overflow: &str, chave: i64) -> std::io::Result<bool> {
    // Primeiro tenta remover do arquivo principal
    if remover_produto(caminho_principal, chave)? {
        return Ok(true);
    }
    
    // Se não encontrou no principal, tenta remover do overflow
    if remover_do_overflow(caminho_overflow, chave)? {
        return Ok(true);
    }
    
    Ok(false)
}

// Função para remover do arquivo de overflow
pub fn remover_do_overflow(caminho_overflow: &str, chave: i64) -> std::io::Result<bool> {
    if !std::path::Path::new(caminho_overflow).exists() {
        return Ok(false);
    }
    
    let mut arquivo = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(caminho_overflow)?;

    let tamanho = arquivo.metadata()?.len();
    let num_registros = tamanho / Produto::TAMANHO_REGISTRO as u64;
    let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];

    for i in 0..num_registros {
        let pos = i * Produto::TAMANHO_REGISTRO as u64;
        arquivo.seek(SeekFrom::Start(pos))?;
        arquivo.read_exact(&mut buffer)?;

        let produto = Produto::from_bytes(&buffer);
        if produto.product_id == chave {
            // Marca como removido: sobrescreve product_id para -1
            arquivo.seek(SeekFrom::Start(pos))?;
            arquivo.write_all(&(-1i64).to_le_bytes())?;
            return Ok(true);
        }
    }
    Ok(false)
}

// Função para consultar com índice considerando overflow
pub fn consultar_com_indice_e_overflow(caminho_principal: &str, caminho_overflow: &str, indice: &IndiceParcial, chave: i64) -> std::io::Result<Option<Produto>> {
    // Primeiro busca usando o índice no arquivo principal
    if let Some(produto) = consultar_com_indice(caminho_principal, indice, chave)? {
        return Ok(Some(produto));
    }
    
    // Se não encontrou no principal, busca no overflow
    if let Some(produto) = buscar_no_overflow(caminho_overflow, chave)? {
        return Ok(Some(produto));
    }
    
    Ok(None)
}

// Função para consultar com índice e overflow (com debug)
pub fn consultar_com_indice_e_overflow_debug(caminho_principal: &str, caminho_overflow: &str, indice: &IndiceParcial, chave: i64) -> std::io::Result<Option<Produto>> {
    println!("\n🔍 === DEBUG: CONSULTA COM ÍNDICE E OVERFLOW ===");
    println!("Chave buscada: {}", chave);
    println!("Total de entradas no indice: {}", indice.entradas.len());
    println!("Fator de esparsidade: {}", indice.fator_esparsidade);
    println!();
    
    // Passo 1: Busca usando índice no arquivo principal
    println!(" PASSO 1: Busca usando índice no arquivo principal");
    if let Some(produto) = consultar_com_indice(caminho_principal, indice, chave)? {
        println!(" Produto encontrado no arquivo principal via índice!");
        println!(" Produto: {:?}", produto);
        return Ok(Some(produto));
    }
    println!(" Produto não encontrado no arquivo principal via índice");
    
    // Passo 2: Busca no arquivo de overflow
    println!();
    println!(" PASSO 2: Busca no arquivo de overflow");
    if !std::path::Path::new(caminho_overflow).exists() {
        println!("  Arquivo de overflow não existe");
        return Ok(None);
    }
    
    let mut arquivo = std::fs::File::open(caminho_overflow)?;
    let tamanho = arquivo.metadata()?.len();
    let num_registros = tamanho / Produto::TAMANHO_REGISTRO as u64;
    let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];
    
    println!(" Total de registros no overflow: {}", num_registros);
    println!("🔍 Iniciando busca sequencial no overflow...");
    
    for i in 0..num_registros {
        let pos = i * Produto::TAMANHO_REGISTRO as u64;
        arquivo.seek(SeekFrom::Start(pos))?;
        match arquivo.read_exact(&mut buffer) {
            Ok(_) => {
                let produto = Produto::from_bytes(&buffer);
                println!("    Registro {}: ID={}, Posição={}", i + 1, produto.product_id, pos);
                
                if produto.product_id == chave {
                    println!("    SUCESSO! Produto encontrado no overflow!");
                    println!("    Produto: {:?}", produto);
                    return Ok(Some(produto));
                }
            }
            Err(_) => {
                println!("    Erro ao ler registro na posição {}", pos);
                break;
            }
        }
    }
    
    println!("    Total de registros verificados no overflow: {}", num_registros);
    println!("    Produto não encontrado no overflow");
    
    println!("\n === FIM DO DEBUG ===");
    Ok(None)
}
