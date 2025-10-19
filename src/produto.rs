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


pub fn consultar_com_indice(caminho_arquivo: &str, indice: &IndiceParcial, chave: i64)
    -> std::io::Result<Option<Produto>> 
{
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
