use serde::{Serialize, Deserialize};
use std::io::{Read, Write};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndiceEntry {
    pub chave: i64,
    pub posicao: u64,
}

impl IndiceEntry {
    pub const TAMANHO_ENTRADA: usize = 16; // 8 bytes para i64 + 8 bytes para u64
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::TAMANHO_ENTRADA);
        bytes.extend_from_slice(&self.chave.to_le_bytes());
        bytes.extend_from_slice(&self.posicao.to_le_bytes());
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let chave = i64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let posicao = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        IndiceEntry { chave, posicao }
    }
}

#[derive(Debug, Clone)]
pub struct IndiceParcial {
    pub entradas: Vec<IndiceEntry>,
    pub fator_esparsidade: usize,
}

impl IndiceParcial {
    pub fn novo(fator_esparsidade: usize) -> Self {
        IndiceParcial {
            entradas: Vec::new(),
            fator_esparsidade,
        }
    }

    pub fn adicionar_entrada(&mut self, chave: i64, posicao: u64) {
        self.entradas.push(IndiceEntry { chave, posicao });
    }


    pub fn salvar_binario(&self, caminho: &str) -> std::io::Result<()> {
        let mut arquivo = std::fs::File::create(caminho)?;
        
        // Escreve o fator de esparsidade no início (4 bytes)
        arquivo.write_all(&(self.fator_esparsidade as u32).to_le_bytes())?;
        
        // Escreve o número de entradas (4 bytes)
        arquivo.write_all(&(self.entradas.len() as u32).to_le_bytes())?;
        
        // Escreve cada entrada
        for entrada in &self.entradas {
            arquivo.write_all(&entrada.to_bytes())?;
        }
        
        Ok(())
    }


    pub fn carregar_binario(caminho: &str) -> std::io::Result<Self> {
        let mut arquivo = std::fs::File::open(caminho)?;
        
        // Lê o fator de esparsidade (4 bytes)
        let mut fator_bytes = [0u8; 4];
        arquivo.read_exact(&mut fator_bytes)?;
        let fator_esparsidade = u32::from_le_bytes(fator_bytes) as usize;
        
        // Lê o número de entradas (4 bytes)
        let mut num_entradas_bytes = [0u8; 4];
        arquivo.read_exact(&mut num_entradas_bytes)?;
        let num_entradas = u32::from_le_bytes(num_entradas_bytes) as usize;
        
        // Lê as entradas
        let mut entradas = Vec::with_capacity(num_entradas);
        let mut buffer = vec![0u8; IndiceEntry::TAMANHO_ENTRADA];
        
        for _ in 0..num_entradas {
            arquivo.read_exact(&mut buffer)?;
            entradas.push(IndiceEntry::from_bytes(&buffer));
        }
        
        Ok(IndiceParcial {
            entradas,
            fator_esparsidade,
        })
    }

    pub fn buscar_posicao(&self, chave: i64) -> Option<(usize, u64)> {
        let mut esq = 0;
        let mut dir = self.entradas.len();
        while esq < dir {
            let meio = (esq + dir) / 2;
            if self.entradas[meio].chave < chave {
                esq = meio + 1;
            } else if self.entradas[meio].chave > chave {
                dir = meio;
            } else {
                return Some((meio, self.entradas[meio].posicao));
            }
        }
        if esq > 0 {
            Some((esq - 1, self.entradas[esq - 1].posicao))
        } else {
            Some((0, 0))
        }
    }
}

pub fn construir_indice_parcial(caminho_arquivo: &str, fator: usize) 
    -> std::io::Result<IndiceParcial> 
{
    use crate::produto::Produto;
    let mut indice = IndiceParcial::novo(fator);
    let mut arquivo = std::fs::File::open(caminho_arquivo)?;
    let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];
    let mut contador = 0;
    let mut posicao = 0u64;
    loop {
        match arquivo.read_exact(&mut buffer) {
            Ok(_) => {
                if contador % fator == 0 {
                    let produto = Produto::from_bytes(&buffer);
                    indice.adicionar_entrada(produto.product_id, posicao);
                }
                contador += 1;
                posicao += Produto::TAMANHO_REGISTRO as u64;
            }
            Err(_) => break,
        }
    }
    Ok(indice)
}
